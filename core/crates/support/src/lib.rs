#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
mod text;

pub use text::{SupportMessageDisplayContent, SupportMessageLink, markdown_plain_text, parse_support_message_display_content};
