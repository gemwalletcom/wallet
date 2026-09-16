use crate::SupportMessageLink;

impl SupportMessageLink {
    pub fn mock(title: &str, url: &str, subtitle: Option<&str>) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            subtitle: subtitle.map(str::to_string),
        }
    }
}
