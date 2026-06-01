use std::time::Duration;

use super::ToolFailure;
use reqwest::redirect::Policy;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct FetchTool {
    pub client: reqwest::Client,
    pub allow: Vec<String>,
    pub timeout_secs: u64,
    pub max_bytes: usize,
}

impl FetchTool {
    pub fn new(allow: Vec<String>, timeout_secs: u64, max_bytes: usize) -> Self {
        // Custom redirect policy: every hop's host must also pass the allowlist.
        // Closes the SSRF-via-open-redirect bypass — an attacker who finds an
        // open-redirect endpoint on an allowed host can't pivot to internal /
        // off-list hosts.
        let allow_for_policy = allow.clone();
        let client = reqwest::Client::builder()
            .redirect(Policy::custom(move |attempt| {
                let host = attempt.url().host_str().unwrap_or("");
                if host_allowed(host, &allow_for_policy) { attempt.follow() } else { attempt.stop() }
            }))
            .build()
            .expect("reqwest::Client::builder failed (rustls + redirect policy)");
        Self {
            client,
            allow,
            timeout_secs,
            max_bytes,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchArgs {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FetchOutput {
    pub status: u16,
    pub body: String,
    pub truncated: bool,
}

impl Tool for FetchTool {
    const NAME: &'static str = "fetch";
    type Error = ToolFailure;
    type Args = FetchArgs;
    type Output = FetchOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        let allow = self.allow.join(", ");
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "HTTP GET against a strict allowlist of hosts. Use this for any external \
                read — Gem Wallet docs/site, GitHub API/raw, etc. The URL's host must \
                exactly match (or be a subdomain of) one of: {allow}. Anything else \
                fails. Returns the response body as text, truncated. No POST, no headers, \
                no redirects to non-allowed hosts."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "Absolute URL to GET. Must use http or https."
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let url = reqwest::Url::parse(&args.url).map_err(|e| ToolFailure::other(format!("invalid url: {e}")))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ToolFailure::not_allowed(format!("scheme `{}` not allowed", url.scheme())));
        }
        let host = url.host_str().ok_or_else(|| ToolFailure::other("url has no host"))?;
        if !host_allowed(host, &self.allow) {
            return Err(ToolFailure::other(format!("host `{host}` not in fetch.allow allowlist")));
        }
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, "gemmy-support-bot")
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| ToolFailure::other(format!("request failed: {e}")))?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await.map_err(|e| ToolFailure::other(format!("reading body: {e}")))?;
        let truncated = bytes.len() > self.max_bytes;
        let slice = if truncated { &bytes[..self.max_bytes] } else { &bytes[..] };
        let body = String::from_utf8_lossy(slice).into_owned();
        Ok(FetchOutput { status, body, truncated })
    }
}

fn host_allowed(host: &str, allow: &[String]) -> bool {
    allow.iter().any(|a| {
        let a = a.trim().trim_start_matches("*.").to_lowercase();
        let host = host.to_lowercase();
        host == a || host.ends_with(&format!(".{a}"))
    })
}

#[cfg(test)]
mod tests {
    use super::host_allowed;

    #[test]
    fn allows_exact_and_subdomains() {
        let allow = vec!["docs.gemwallet.com".to_string(), "gemwallet.com".to_string(), "github.com".to_string()];
        assert!(host_allowed("docs.gemwallet.com", &allow));
        assert!(host_allowed("gemwallet.com", &allow));
        assert!(host_allowed("api.github.com", &allow));
        assert!(host_allowed("raw.githubusercontent.com", &["githubusercontent.com".to_string()]));
    }

    #[test]
    fn rejects_anything_off_list() {
        let allow = vec!["docs.gemwallet.com".to_string()];
        assert!(!host_allowed("evil.com", &allow));
        assert!(!host_allowed("docs.gemwallet.com.evil.com", &allow));
        assert!(!host_allowed("gemwallet.com", &allow));
    }

    #[test]
    fn star_prefix_is_normalised() {
        let allow = vec!["*.gemwallet.com".to_string()];
        assert!(host_allowed("docs.gemwallet.com", &allow));
        assert!(host_allowed("gemwallet.com", &allow));
        assert!(!host_allowed("evil.com", &allow));
    }
}
