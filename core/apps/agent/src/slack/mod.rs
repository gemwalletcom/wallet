pub mod client;
pub mod dispatch;
pub mod mrkdwn;
pub mod socket;

pub use client::SlackClient;

pub fn channel_allowed(channel: &str, allow: &[String]) -> bool {
    let needle = channel.trim().trim_start_matches('#');
    allow.iter().any(|a| a.trim().trim_start_matches('#') == needle)
}

#[cfg(test)]
mod tests {
    use super::channel_allowed;

    #[test]
    fn matches_channels_with_and_without_hash() {
        let allow = vec!["#support".to_string()];
        assert!(channel_allowed("#support", &allow));
        assert!(channel_allowed("support", &allow));
        assert!(!channel_allowed("#general", &allow));
        assert!(!channel_allowed("supports", &allow));
    }
}
