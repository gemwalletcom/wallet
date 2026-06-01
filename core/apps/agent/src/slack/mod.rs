pub mod client;
pub mod dispatch;
pub mod mrkdwn;
pub mod socket;

pub use client::SlackClient;

pub fn channel_allowed(channel: &str, allow: &[String]) -> bool {
    let needle = channel.trim().trim_start_matches('#');
    allow.iter().any(|a| a.trim().trim_start_matches('#') == needle)
}
