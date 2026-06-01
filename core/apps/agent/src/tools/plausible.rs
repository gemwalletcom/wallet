use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone)]
pub struct PlausibleTool {
    pub client: reqwest::Client,
    pub base_url: String,
    pub api_key: String,
    pub sites: Vec<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct PlausibleArgs {
    pub site_id: String,
    pub metrics: Vec<String>,
    #[serde(default)]
    pub dimensions: Vec<String>,
    #[serde(default)]
    pub date_range: Option<String>,
    #[serde(default)]
    pub filters: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct PlausibleOutput(Value);

impl Tool for PlausibleTool {
    const NAME: &'static str = "plausible";
    type Error = ToolFailure;
    type Args = PlausibleArgs;
    type Output = PlausibleOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        let sites = self.sites.join(", ");
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Query the Plausible analytics Stats API. `site_id` must be one of the tracked \
                domains ({sites}); anything else is rejected at the tool boundary. `metrics` is \
                required (e.g. `visitors`, `pageviews`, `visits`, `bounce_rate`, \
                `visit_duration`, `events`). `dimensions` optionally groups results (e.g. \
                `visit:source`, `event:page`, `visit:country`). `date_range` is a string like \
                `7d`, `30d`, `month`, `12mo`, or a `[start, end]` ISO pair (default `7d`). \
                `filters` is the Plausible v2 filter array if you need to scope results."
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "site_id": { "type": "string", "description": "Plausible site id (domain)." },
                    "metrics": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Plausible metrics to aggregate."
                    },
                    "dimensions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional dimensions to group by."
                    },
                    "date_range": { "type": "string", "description": "Date range, default `7d`." },
                    "filters": { "description": "Optional Plausible v2 filter array." }
                },
                "required": ["site_id", "metrics"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if self.api_key.is_empty() {
            return Err(ToolFailure::not_allowed("plausible api key not configured"));
        }
        if !self.sites.contains(&args.site_id) {
            return Err(ToolFailure::not_allowed(format!(
                "site_id `{}` is not in the configured plausible.sites allow-list",
                args.site_id
            )));
        }
        let body = json!({
            "site_id": args.site_id,
            "metrics": args.metrics,
            "dimensions": args.dimensions,
            "date_range": args.date_range.as_deref().unwrap_or("7d"),
            "filters": args.filters.unwrap_or(Value::Array(vec![])),
        });
        let url = format!("{}/api/v2/query", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolFailure::other(format!("plausible request: {e}")))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ToolFailure::other(format!("plausible {status}: {text}")));
        }
        let value: Value = serde_json::from_str(&text).map_err(|e| ToolFailure::other(format!("plausible non-json: {e}")))?;
        Ok(PlausibleOutput(value))
    }
}
