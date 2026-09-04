use std::error::Error;

use gem_evm::ethereum_address_checksum;
use primitives::name::NameRecord;
use primitives::{Chain, EVMChain};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct NameConfig {
    pub max_name_length: usize,
}

pub struct NameClient {
    providers: Vec<Box<dyn NameResolver>>,
    config: NameConfig,
}

impl NameClient {
    pub fn new(providers: Vec<Box<dyn NameResolver>>, config: NameConfig) -> Self {
        Self { providers, config }
    }

    pub async fn resolve(&self, name: &str, chain: Chain) -> Result<Option<NameRecord>, Box<dyn Error + Send + Sync>> {
        let query = NameQuery::new(name);
        if query.name.len() > self.config.max_name_length {
            return Err(format!("name '{}' exceeds maximum length of {}", query.name, self.config.max_name_length).into());
        }

        let provider = self.matched_provider(name, chain)?;
        let Some(address) = provider.resolve(&query, chain).await? else {
            return Ok(None);
        };
        let address = match EVMChain::from_chain(chain) {
            Some(_) => ethereum_address_checksum(&address)?,
            None => address,
        };

        Ok(Some(NameRecord {
            provider: provider.provider(),
            address,
            name: name.to_string(),
            chain,
        }))
    }

    fn matched_provider(&self, name: &str, chain: Chain) -> Result<&dyn NameResolver, Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .enumerate()
            .filter(|(_, provider)| provider.chains().contains(&chain))
            .filter_map(|(index, provider)| {
                provider
                    .domains()
                    .iter()
                    .filter_map(|domain| domain_match_len(name, domain))
                    .max()
                    .map(|match_len| (match_len, index, provider.as_ref()))
            })
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)))
            .map(|(_, _, provider)| provider)
            .ok_or_else(|| format!("No provider found for name: {name}").into())
    }
}

fn domain_match_len(name: &str, domain: &str) -> Option<usize> {
    if domain == "*" {
        return Some(0);
    }

    if name.eq_ignore_ascii_case(domain) {
        Some(domain.len())
    } else if name.len() > domain.len() {
        let offset = name.len() - domain.len();
        let has_dot = name.as_bytes().get(offset - 1).is_some_and(|byte| *byte == b'.');
        let is_suffix_match = name.get(offset..).is_some_and(|suffix| suffix.eq_ignore_ascii_case(domain));

        if has_dot && is_suffix_match { Some(domain.len()) } else { None }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, NameProvider};

    use super::{NameClient, NameConfig};
    use crate::testkit::MockNameResolver;

    #[tokio::test]
    async fn test_resolve_checksums_evm_address() {
        let client = NameClient::new(
            vec![
                Box::new(MockNameResolver::new(
                    NameProvider::Ud,
                    vec!["crypto"],
                    vec![Chain::Ethereum],
                    Ok("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"),
                )),
                Box::new(MockNameResolver::new(
                    NameProvider::Sns,
                    vec!["sol"],
                    vec![Chain::Solana],
                    Ok("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
                )),
            ],
            NameConfig { max_name_length: 20 },
        );

        let ethereum = client.resolve("example.crypto", Chain::Ethereum).await.unwrap().unwrap();
        let solana = client.resolve("example.sol", Chain::Solana).await.unwrap().unwrap();

        assert_eq!(ethereum.address, "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a");
        assert_eq!(solana.address, "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2");
    }

    #[tokio::test]
    async fn test_resolve_prefers_longer_domain_match() {
        let client = NameClient::new(
            vec![
                Box::new(MockNameResolver::new(
                    NameProvider::Ens,
                    vec!["*"],
                    vec![Chain::Base],
                    Ok("0x0000000000000000000000000000000000000001"),
                )),
                Box::new(MockNameResolver::new(
                    NameProvider::Basenames,
                    vec!["base.eth"],
                    vec![Chain::Base],
                    Ok("0x0000000000000000000000000000000000000002"),
                )),
            ],
            NameConfig { max_name_length: 20 },
        );

        let record = client.resolve("alice.base.eth", Chain::Base).await.unwrap().unwrap();

        assert_eq!(record.provider, NameProvider::Basenames);
        assert_eq!(record.address, "0x0000000000000000000000000000000000000002");
    }

    #[tokio::test]
    async fn test_resolve_rejects_long_name() {
        let client = NameClient::new(
            vec![Box::new(MockNameResolver::new(
                NameProvider::Injective,
                vec!["inj"],
                vec![Chain::Injective],
                Ok("inj14apqz6u2nprsly3j0mqa6jwpxnmnphq3pp0q9g"),
            ))],
            NameConfig { max_name_length: 20 },
        );

        let result = client.resolve("inj1kly3z4r8pzgfhh9cx5x69xjw0j4evlepq6ccgw.inj", Chain::Injective).await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "name 'inj1kly3z4r8pzgfhh9cx5x69xjw0j4evlepq6ccgw' exceeds maximum length of 20"
        );
    }

    #[tokio::test]
    async fn test_resolve_returns_more_specific_provider_error() {
        let client = NameClient::new(
            vec![
                Box::new(MockNameResolver::new(
                    NameProvider::Ens,
                    vec!["*"],
                    vec![Chain::Base],
                    Ok("0x0000000000000000000000000000000000000003"),
                )),
                Box::new(MockNameResolver::new(NameProvider::Basenames, vec!["base.eth"], vec![Chain::Base], Err("failed"))),
            ],
            NameConfig { max_name_length: 20 },
        );

        let error = client.resolve("alice.base.eth", Chain::Base).await.err().unwrap();

        assert_eq!(error.to_string(), "failed");
    }
}
