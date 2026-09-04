use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_solana::Pubkey;
use primitives::contract_constants::SOLANA_ALLDOMAINS_ROOT_PUBLIC_KEY;
use primitives::{Chain, NameProvider};

use super::account::{name_account_key, nft_record_key, tld_house_key};
use super::client::AllDomainsClient;
use super::model::NameRecord;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct AllDomainsProvider {
    client: AllDomainsClient,
}

impl AllDomainsProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: AllDomainsClient::new(client),
        }
    }
}

#[async_trait]
impl NameResolver for AllDomainsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::AllDomains
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["skr", "saga", "poor", "bonk", "solana"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        if query.suffix.is_empty() || query.suffix.contains('.') {
            return Err(format!("invalid domain: {}", query.domain).into());
        }
        let tld = format!(".{}", query.suffix);
        let root = Pubkey::from_str(SOLANA_ALLDOMAINS_ROOT_PUBLIC_KEY)?;
        let tld_account = name_account_key(&tld, &root)?;
        let name_account = name_account_key(&query.name, &tld_account)?;

        let Some(record) = self.client.get_name_record(&name_account).await?.filter(NameRecord::is_active) else {
            return Ok(None);
        };
        if record.owner == nft_record_key(&name_account, &tld_house_key(&tld)?)? {
            return Err("NFT owner resolution is not supported".into());
        }
        Ok(Some(record.owner.to_string()))
    }
}
