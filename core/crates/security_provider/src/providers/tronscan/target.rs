use gem_client::Target;

pub enum TronscanTarget {
    AddressSecurity { address: String },
    TokenSecurity { address: String },
}

impl Target for TronscanTarget {
    fn path(&self) -> String {
        match self {
            Self::AddressSecurity { address } => format!("/api/security/account/data?address={address}"),
            Self::TokenSecurity { address } => format!("/api/security/token/data?address={address}"),
        }
    }
}
