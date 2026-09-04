use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum SpaceIdTarget {
    Address { tld: String, domain: String },
}

impl Target for SpaceIdTarget {
    fn path(&self) -> String {
        match self {
            Self::Address { tld, domain } => build_path_with_query("/v1/getAddress", &[("tld", tld), ("domain", domain)]),
        }
    }
}
