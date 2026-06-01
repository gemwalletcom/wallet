use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::store::MemoryStore;
use crate::tools::ToolFailure;

#[derive(Clone)]
pub struct SearchMemoryTool {
    pub store: Arc<MemoryStore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMemoryArgs {
    pub query: String,
    #[serde(default)]
    pub top_n: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchMemoryHit {
    pub source: String,
    pub content: String,
}

impl Tool for SearchMemoryTool {
    const NAME: &'static str = "search_memory";
    type Error = ToolFailure;
    type Args = SearchMemoryArgs;
    type Output = Vec<SearchMemoryHit>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Semantic search over the long-term memory store \
                (indexed `context/*.md` + `agents/<name>/*.md` + `agents/<name>/memory/*.md` \
                plus runtime-saved entries). Returns the top-N most relevant docs \
                (source path + full content). Use when a question hints at internal Gem \
                knowledge but you don't know which exact file holds it. For known files, \
                `cat` is faster."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "What you're trying to find out." },
                    "top_n": { "type": "integer", "default": 3, "description": "How many hits to return." }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let n = args.top_n.unwrap_or(3);
        let hits = self.store.search(&args.query, n).await?;
        Ok(hits
            .into_iter()
            .map(|m| SearchMemoryHit {
                source: m.source,
                content: m.content,
            })
            .collect())
    }
}

#[derive(Clone)]
pub struct SaveMemoryTool {
    pub store: Arc<MemoryStore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveMemoryArgs {
    pub id: String,
    pub content: String,
}

impl Tool for SaveMemoryTool {
    const NAME: &'static str = "save_memory";
    type Error = ToolFailure;
    type Args = SaveMemoryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Persist a durable note into long-term memory (sqlite vector store). \
                Pick a stable kebab-case `id`; reusing an `id` overwrites the previous entry. \
                `content` is the text to remember, formatted as Markdown."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "kebab-case slug" },
                    "content": { "type": "string", "description": "the fact, 1-3 sentences" }
                },
                "required": ["id", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let id = args.id.clone();
        self.store.save(args.id, "runtime".to_string(), args.content).await?;
        Ok(format!("saved memo `{}` to long-term memory", id))
    }
}
