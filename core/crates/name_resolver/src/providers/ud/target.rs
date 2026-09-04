use gem_client::Target;

#[derive(Clone, Debug)]
pub enum UdTarget {
    Domain { domain: String },
}

impl Target for UdTarget {
    fn path(&self) -> String {
        match self {
            Self::Domain { domain } => format!("/resolve/domains/{domain}"),
        }
    }
}
