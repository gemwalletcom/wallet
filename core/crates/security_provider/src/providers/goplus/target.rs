use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum GoPlusTarget {
    Token,
    AddressSecurity { address: String, chain_id: &'static str },
    TokenSecurity { chain_id: &'static str, contract_addresses: String },
}

impl Target for GoPlusTarget {
    fn path(&self) -> String {
        match self {
            Self::Token => "/api/v1/token".to_string(),
            Self::AddressSecurity { address, chain_id } => build_path_with_query(&format!("/api/v1/address_security/{address}"), &[("chain_id", chain_id)]),
            Self::TokenSecurity { chain_id, contract_addresses } => {
                build_path_with_query(&format!("/api/v1/token_security/{chain_id}"), &[("contract_addresses", contract_addresses)])
            }
        }
    }
}
