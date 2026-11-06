mod identity;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
mod text;
mod webhook;

pub use identity::identifier_hash;
pub use text::{SupportMessageDisplayContent, SupportMessageLink, markdown_plain_text, parse_support_message_display_content};
pub use webhook::ChatwootWebhookVerifier;
