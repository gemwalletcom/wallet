use std::collections::BTreeMap;
use std::time::Duration;

use crate::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use rig::tool::ToolDyn;
use rig::tool::rmcp::McpTool;
use rmcp::ServiceExt;
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientTransportConfig};

use gem_tracing::tracing::{info, warn};

use crate::DispatchSource;
use crate::config::McpServerDef;
use crate::tools::{GatedTool, ToolPolicy};

const LIST_TOOLS_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn connect_servers(defs: &BTreeMap<String, McpServerDef>, selections: &BTreeMap<String, Vec<DispatchSource>>) -> Result<Vec<Box<dyn ToolDyn>>> {
    let mut tools = Vec::new();
    for (name, allow_sources) in selections {
        let Some(def) = defs.get(name) else {
            warn!(mcp = %name, "mcp not defined in config; agent will run without its tools");
            continue;
        };
        match connect_one(name, def, allow_sources).await {
            Ok(new) => {
                info!(mcp = %name, url = %def.url, tools = new.len(), "mcp connected");
                tools.extend(new);
            }
            Err(e) => warn!(
                mcp = %name,
                url = %def.url,
                error = %e,
                "mcp unavailable; agent will run without its tools",
            ),
        }
    }
    Ok(tools)
}

async fn connect_one(name: &str, def: &McpServerDef, allow_sources: &[DispatchSource]) -> Result<Vec<Box<dyn ToolDyn>>> {
    let http = reqwest::Client::builder().default_headers(auth_headers(name, def)?).build()?;
    let transport = StreamableHttpClientTransport::with_client(http, StreamableHttpClientTransportConfig::with_uri(def.url.clone()));
    let service = ClientInfo::new(ClientCapabilities::default(), Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
        .serve(transport)
        .await
        .map_err(|e| format!("handshake: {e}"))?;

    let listed = tokio::time::timeout(LIST_TOOLS_TIMEOUT, service.list_tools(Default::default()))
        .await
        .map_err(|e| format!("tools/list timed out: {e}"))??;

    let peer = service.peer().clone();
    Box::leak(Box::new(service));

    let policy = ToolPolicy {
        allow_sources: allow_sources.to_vec(),
    };
    Ok(listed
        .tools
        .into_iter()
        .map(|tool| {
            Box::new(GatedTool {
                inner: Box::new(McpTool::from_mcp_server(tool, peer.clone())),
                policy: policy.clone(),
            }) as Box<dyn ToolDyn>
        })
        .collect())
}

fn auth_headers(name: &str, def: &McpServerDef) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let Some(var) = &def.token else {
        return Ok(headers);
    };
    let token = std::env::var(var).map_err(|_| format!("${var} must be set (referenced by mcp `{name}`)"))?;
    headers.insert(reqwest::header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {token}"))?);
    Ok(headers)
}
