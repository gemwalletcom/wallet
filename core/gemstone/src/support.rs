use support::{SupportMessageDisplayContent, SupportMessageLink};

#[uniffi::remote(Record)]
pub struct SupportMessageDisplayContent {
    pub text: String,
    pub links: Vec<SupportMessageLink>,
}

#[uniffi::remote(Record)]
pub struct SupportMessageLink {
    pub title: String,
    pub url: String,
    pub subtitle: Option<String>,
}
