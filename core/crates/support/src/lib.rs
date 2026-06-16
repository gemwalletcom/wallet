mod chatwoot;
mod client;
mod constants;
mod model;
mod text;

pub use chatwoot::ChatwootClient;
pub use client::{SupportClient, SupportWebhookResult};
pub use model::*;
pub use text::markdown_plain_text;
