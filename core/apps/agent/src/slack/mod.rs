pub mod client;
pub mod dispatch;
pub mod mrkdwn;
pub mod socket;

pub use client::SlackClient;

pub fn channel_allowed(channel: &str, allow: &[String]) -> bool {
    let needle = channel.trim().trim_start_matches('#');
    allow.iter().any(|a| a.trim().trim_start_matches('#') == needle)
}

pub fn is_user_id(channel: &str) -> bool {
    channel.trim().starts_with('U')
}

#[cfg(test)]
mod tests {
    use super::{channel_allowed, is_user_id};

    #[test]
    fn matches_channels_with_and_without_hash() {
        let allow = vec!["#support".to_string()];
        assert!(channel_allowed("#support", &allow));
        assert!(channel_allowed("support", &allow));
        assert!(!channel_allowed("#general", &allow));
        assert!(!channel_allowed("supports", &allow));
    }

    #[test]
    fn recognizes_user_ids() {
        assert!(is_user_id("U0123456789"));
        assert!(!is_user_id("D0123456789"));
    }
}
