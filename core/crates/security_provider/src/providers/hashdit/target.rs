use gem_client::Target;

#[derive(Clone, Debug)]
pub enum HashDitTarget {
    AddressPoisoning,
    AddressSecurity,
    DomainSecurity,
    SolanaTokenSecurity,
    TokenSecurity,
}

impl Target for HashDitTarget {
    fn path(&self) -> String {
        match self {
            Self::AddressPoisoning => "/v2/hashdit/address-poisoning".to_string(),
            Self::AddressSecurity => "/v2/hashdit/address-security-v2".to_string(),
            Self::DomainSecurity => "/v2/hashdit/domain-security".to_string(),
            Self::SolanaTokenSecurity => "/v2/hashdit/solana-token-security".to_string(),
            Self::TokenSecurity => "/v2/hashdit/token-security".to_string(),
        }
    }
}
