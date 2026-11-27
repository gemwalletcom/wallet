use std::collections::HashMap;

use gem_client::{Client, ClientError, ClientExt, Target};

use super::model::PartnerTokenResponse;

pub const PARTNER_URL: &str = "https://nd1.breez.technology";

struct PartnerTokenTarget<'a> {
    key: &'a str,
}

impl Target for PartnerTokenTarget<'_> {
    fn path(&self) -> String {
        "/api/jwt".to_string()
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("Authorization".to_string(), format!("Bearer {}", self.key))])
    }
}

#[derive(Debug)]
pub(super) struct PartnerClient<C: Client> {
    client: C,
    key: String,
}

impl<C: Client> PartnerClient<C> {
    pub fn new(client: C, key: String) -> Self {
        Self { client, key }
    }

    pub async fn get_token(&self) -> Result<PartnerTokenResponse, ClientError> {
        self.client.get(PartnerTokenTarget { key: &self.key }).await
    }
}
