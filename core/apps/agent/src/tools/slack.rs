use super::ToolFailure;
use crate::slack::{SlackClient, channel_allowed};

pub async fn resolve_allowed_channel(client: &SlackClient, requested: &str, allow_channels: &[String], error_field: &str) -> Result<String, ToolFailure> {
    let requested = requested.trim();

    if channel_allowed(requested, allow_channels) {
        return client
            .public_channel_id_by_name(requested)
            .await
            .map_err(|e| ToolFailure::other(e.to_string()))?
            .ok_or_else(|| ToolFailure::other(format!("slack channel `{requested}` not found")));
    }

    if let Ok(Some(name)) = client.conversation_name(requested).await
        && channel_allowed(&name, allow_channels)
    {
        return Ok(requested.to_string());
    }

    Err(ToolFailure::not_allowed(format!("channel `{requested}` not in {error_field} allow-list")))
}
