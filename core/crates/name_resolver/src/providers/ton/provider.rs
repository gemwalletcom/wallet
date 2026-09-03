use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_ton::address::hex_to_base64_address;
use gem_ton::models::DnsRecordsResponse;
use gem_ton::rpc::client::TonClient;
use primitives::{Chain, NameProvider};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct TonProvider {
    client: TonClient<ReqwestClient>,
}

impl TonProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: TonClient::new(client) }
    }

    fn map_address(response: DnsRecordsResponse) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let Some(wallet) = response.records.first().and_then(|record| record.dns_wallet.as_deref()) else {
            return Ok(None);
        };
        Ok(Some(hex_to_base64_address(wallet).ok_or_else(|| format!("invalid TON DNS wallet: {wallet}"))?))
    }
}

#[async_trait]
impl NameResolver for TonProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Ton
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["ton"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ton]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Self::map_address(self.client.get_dns_records(&query.domain).await?)
    }
}

#[cfg(test)]
mod tests {
    use gem_ton::models::DnsRecordsResponse;

    use super::TonProvider;

    #[test]
    fn test_map_address() {
        let response: DnsRecordsResponse = serde_json::from_str(include_str!("../../../testdata/ton_dns_records_response.json")).unwrap();

        assert_eq!(
            TonProvider::map_address(response).unwrap().as_deref(),
            Some("EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ")
        );
    }
}
