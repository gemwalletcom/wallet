use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, Target};

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

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([(CONTENT_TYPE.to_string(), "application/json".to_string())])
    }
}
