mod identity;
#[cfg(any(test, debug_assertions))]
pub mod testkit;
mod text;
mod webhook;

pub use identity::identifier_hash;
pub use text::{SupportMessageDisplayContent, SupportMessageLink, parse_support_message_display_content, support_message_notification_text};
pub use webhook::ChatwootWebhookVerifier;
