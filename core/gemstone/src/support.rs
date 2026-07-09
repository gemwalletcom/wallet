use support::{
    SupportMessageDisplayContent as CoreSupportMessageDisplayContent, SupportMessageLink as CoreSupportMessageLink,
    parse_support_message_display_content as parse_core_support_message_display_content,
};

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SupportMessageDisplayContent {
    pub text: String,
    pub links: Vec<SupportMessageLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SupportMessageLink {
    pub title: String,
    pub url: String,
    pub subtitle: Option<String>,
}

#[uniffi::export]
pub fn parse_support_message_display_content(markdown: &str) -> SupportMessageDisplayContent {
    parse_core_support_message_display_content(markdown).into()
}

impl From<CoreSupportMessageDisplayContent> for SupportMessageDisplayContent {
    fn from(value: CoreSupportMessageDisplayContent) -> Self {
        Self {
            text: value.text,
            links: value.links.into_iter().map(SupportMessageLink::from).collect(),
        }
    }
}

impl From<CoreSupportMessageLink> for SupportMessageLink {
    fn from(value: CoreSupportMessageLink) -> Self {
        Self {
            title: value.title,
            url: value.url,
            subtitle: value.subtitle,
        }
    }
}
