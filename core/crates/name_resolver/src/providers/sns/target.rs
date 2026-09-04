use gem_client::Target;

#[derive(Clone, Debug)]
pub enum SnsTarget {
    Resolve { domain: String },
    Record { domain: String, record: String },
}

impl Target for SnsTarget {
    fn path(&self) -> String {
        match self {
            Self::Resolve { domain } => format!("/resolve/{domain}"),
            Self::Record { domain, record } => format!("/record-v2/{domain}/{record}"),
        }
    }
}
