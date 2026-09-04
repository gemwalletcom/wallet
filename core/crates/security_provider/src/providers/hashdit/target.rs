use gem_client::Target;

#[derive(Clone, Debug)]
pub enum HashDitTarget {
    AddressSecurity,
    TokenSecurity,
}

impl Target for HashDitTarget {
    fn path(&self) -> String {
        match self {
            Self::AddressSecurity => "/v2/hashdit/address-security-v2".to_string(),
            Self::TokenSecurity => "/v2/hashdit/token-security".to_string(),
        }
    }
}
