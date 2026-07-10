#[cfg(feature = "backend")]
mod chatwoot;
#[cfg(feature = "backend")]
mod client;
#[cfg(feature = "backend")]
mod constants;
#[cfg(feature = "backend")]
mod model;
mod text;
#[cfg(feature = "backend")]
mod webhook;

#[cfg(feature = "backend")]
pub use chatwoot::ChatwootClient;
#[cfg(feature = "backend")]
pub use client::{SupportClient, SupportWebhookResult};
#[cfg(feature = "backend")]
pub use model::*;
pub use text::{SupportMessageDisplayContent, SupportMessageLink, markdown_plain_text, parse_support_message_display_content};
#[cfg(feature = "backend")]
pub use webhook::{ChatwootWebhookError, ChatwootWebhookVerifier};
