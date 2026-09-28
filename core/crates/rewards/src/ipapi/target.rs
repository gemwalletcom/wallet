use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum IpApiTarget {
    Check { ip_address: String },
}

impl Target for IpApiTarget {
    fn path(&self) -> String {
        match self {
            Self::Check { ip_address } => build_path_with_query("/", &[("q", ip_address)]),
        }
    }
}
