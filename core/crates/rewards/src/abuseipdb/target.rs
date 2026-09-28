use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum AbuseIpDbTarget {
    Check { ip_address: String },
}

impl Target for AbuseIpDbTarget {
    fn path(&self) -> String {
        match self {
            Self::Check { ip_address } => build_path_with_query("/api/v2/check", &[("ipAddress", ip_address)]),
        }
    }
}
