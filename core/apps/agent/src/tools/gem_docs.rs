use std::time::Duration;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::{Display, EnumIter};

use crate::tools::enum_slugs;

const BASE_URL: &str = "https://docs.gemwallet.com";

#[derive(Clone)]
pub struct GemDocsTool {
    pub client: reqwest::Client,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum GemDocsAction {
    Index,
    Page,
}

#[derive(Debug, Deserialize)]
pub struct GemDocsArgs {
    pub action: GemDocsAction,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GemDocsOutput {
    pub content: String,
}

impl Tool for GemDocsTool {
    const NAME: &'static str = "gem_docs";
    type Error = ToolFailure;
    type Args = GemDocsArgs;
    type Output = GemDocsOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read the public Gem Wallet docs at docs.gemwallet.com (code in \
                gemwalletcom/core / wallet is authoritative when docs lag — flag the gap). action: \
                index (the full page map of titles + paths, to discover what exists); \
                page (fetch one page as markdown — `path` is the page path without extension, e.g. \
                blockchains/solana or guides/gem-wallet-basics; blockchain pages are \
                blockchains/<chain> keyed off chains.md, so construct them directly without the index). \
                When citing to a customer, link the human-facing page (no extension), never the raw markdown URL."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": enum_slugs::<GemDocsAction>() },
                    "path": { "type": "string", "description": "Page path without extension. Required for action=page." }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let url = match args.action {
            GemDocsAction::Index => format!("{BASE_URL}/llms.txt"),
            GemDocsAction::Page => {
                let path = args
                    .path
                    .as_deref()
                    .ok_or_else(|| ToolFailure::missing("path", "page"))?
                    .trim()
                    .trim_matches('/')
                    .trim_end_matches(".md");
                format!("{BASE_URL}/{path}.md")
            }
        };
        let resp = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| ToolFailure::other(format!("{url}: {e}")))?;
        let status = resp.status();
        let content = resp.text().await.map_err(|e| ToolFailure::other(format!("{url}: read body: {e}")))?;
        if !status.is_success() {
            return Err(ToolFailure::other(format!(
                "{url} failed: {status} — page may not exist; check action=index for the right path"
            )));
        }
        Ok(GemDocsOutput { content })
    }
}
