use std::error::Error;

use gem_evm::ethereum_address_checksum;
use primitives::name::{NameRecord, is_name};
use primitives::{Chain, EVMChain};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct NameClient {
    providers: Vec<Box<dyn NameResolver>>,
}

impl NameClient {
    pub fn new(providers: Vec<Box<dyn NameResolver>>) -> Self {
        Self { providers }
    }

    pub async fn resolve(&self, name: &str, chain: Chain) -> Result<Option<NameRecord>, Box<dyn Error + Send + Sync>> {
        if !is_name(name) {
            return Ok(None);
        }
        let query = NameQuery::new(name);
        let Some(provider) = self.matched_provider(name, chain) else {
            return Ok(None);
        };

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

    fn matched_provider(&self, name: &str, chain: Chain) -> Option<&dyn NameResolver> {
        self.providers
            .iter()
            .enumerate()
            .filter(|(_, provider)| provider.chains().contains(&chain))
            .filter_map(|(index, provider)| provider.domains().iter().filter_map(|domain| domain_match_len(name, domain)).max().map(|match_len| (match_len, index, provider.as_ref())))
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)))
            .map(|(_, _, provider)| provider)
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
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use primitives::{Chain, NameProvider, name::NameRecord};

    use super::NameClient;
    use crate::providers::sns::SnsProvider;
    use crate::testkit::MockNameResolver;

    #[tokio::test]
    async fn test_resolve_checksums_evm_address() {
        let client = NameClient::new(vec![
            Box::new(MockNameResolver::new(NameProvider::Ud, vec!["crypto"], vec![Chain::Ethereum], Ok("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"))),
            Box::new(MockNameResolver::new(NameProvider::Sns, vec!["sns"], vec![Chain::Solana], Ok("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"))),
        ]);

        let ethereum = client.resolve("example.crypto", Chain::Ethereum).await.unwrap().unwrap();
        let solana = client.resolve("example.sns", Chain::Solana).await.unwrap().unwrap();

        assert_eq!(ethereum.address, "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a");
        assert_eq!(solana.address, "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2");
    }

    #[tokio::test]
    async fn test_resolve_prefers_longer_domain_match() {
        let client = NameClient::new(vec![
            Box::new(MockNameResolver::new(NameProvider::Ens, vec!["*"], vec![Chain::Base], Ok("0x0000000000000000000000000000000000000001"))),
            Box::new(MockNameResolver::new(NameProvider::Basenames, vec!["base.eth"], vec![Chain::Base], Ok("0x0000000000000000000000000000000000000002"))),
        ]);

        let record = client.resolve("alice.base.eth", Chain::Base).await.unwrap().unwrap();

        assert_eq!(record.provider, NameProvider::Basenames);
        assert_eq!(record.address, "0x0000000000000000000000000000000000000002");
    }

    #[tokio::test]
    async fn test_resolve_long_sns_names() {
        for name in ["fatherstretchmyhandspt2.sns", "abcdefghijklmnopqrstu.sns", "sub.fatherstretchmyhandspt2.sns"] {
            let transport = MockClient::new().with_get(move |path| {
                assert_eq!(path, format!("/resolve/{name}"));
                Ok(br#"{"s":"ok","result":"GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"}"#.to_vec())
            });
            let client = NameClient::new(vec![Box::new(SnsProvider::new(transport))]);

            assert_eq!(
                client.resolve(name, Chain::Solana).await.unwrap(),
                Some(NameRecord {
                    provider: NameProvider::Sns,
                    address: "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2".to_string(),
                    name: name.to_string(),
                    chain: Chain::Solana,
                })
            );
        }
    }

    #[tokio::test]
    async fn test_resolve_skips_input_that_is_not_a_name() {
        let client = NameClient::new(vec![Box::new(MockNameResolver::new(NameProvider::Ud, vec!["com"], vec![Chain::Tron], Err("must be valid domain or subdomain")))]);

        assert_eq!(client.resolve("yyy@gmail.com", Chain::Tron).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_resolve_sns_missing_domain_has_no_record() {
        let transport = MockClient::new().with_get(|_| {
            Err(ClientError::Http {
                status: 404,
                body: br#"{"s":"error","result":"Domain not found"}"#.to_vec(),
            })
        });
        let client = NameClient::new(vec![Box::new(SnsProvider::new(transport))]);

        assert_eq!(client.resolve("fatherstretchmyhandspt2.sns", Chain::Solana).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_resolve_sns_outage_is_an_error() {
        let transport = MockClient::new().with_get(|_| Err(ClientError::Http { status: 503, body: vec![] }));
        let client = NameClient::new(vec![Box::new(SnsProvider::new(transport))]);

        assert!(client.resolve("bonfida.sns", Chain::Solana).await.is_err());
    }

    #[tokio::test]
    async fn test_resolve_long_sns_name_preserves_provider_error() {
        let transport = MockClient::new().with_get(|path| {
            assert_eq!(path, "/resolve/fatherstretchmyhandspt2.sns");
            Ok(br#"{"s":"error","result":"Domain not found"}"#.to_vec())
        });
        let client = NameClient::new(vec![Box::new(SnsProvider::new(transport))]);

        let error = client.resolve("fatherstretchmyhandspt2.sns", Chain::Solana).await.unwrap_err();

        assert_eq!(error.to_string(), "SNS request failed with status: error");
    }

    #[tokio::test]
    async fn test_resolve_sns_bsc_record() {
        for name in ["fatherstretchmyhandspt2.sns", "sub.fatherstretchmyhandspt2.sns"] {
            let transport = MockClient::new().with_get(move |path| {
                let domain = name.rsplit_once('.').unwrap().0;
                assert_eq!(path, format!("/record-v2/{domain}/BSC"));
                Ok(br#"{"s":"ok","result":{"deserialized":"0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a","stale":false}}"#.to_vec())
            });
            let client = NameClient::new(vec![Box::new(SnsProvider::new(transport))]);

            assert_eq!(
                client.resolve(name, Chain::SmartChain).await.unwrap(),
                Some(NameRecord {
                    provider: NameProvider::Sns,
                    address: "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a".to_string(),
                    name: name.to_string(),
                    chain: Chain::SmartChain,
                })
            );
        }
    }

    #[tokio::test]
    async fn test_resolve_sns_rejects_paused_sol_names() {
        let client = NameClient::new(vec![Box::new(SnsProvider::new(MockClient::new()))]);

        assert_eq!(client.resolve("bonfida.sol", Chain::Solana).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_resolve_long_names() {
        for (provider, suffix, chain) in [
            (NameProvider::Ud, "crypto", Chain::Ethereum),
            (NameProvider::Ens, "eth", Chain::Ethereum),
            (NameProvider::Sns, "sns", Chain::Solana),
            (NameProvider::Ton, "ton", Chain::Ton),
            (NameProvider::Spaceid, "bnb", Chain::SmartChain),
            (NameProvider::Did, "bit", Chain::Ethereum),
            (NameProvider::Suins, "sui", Chain::Sui),
            (NameProvider::Aptos, "apt", Chain::Aptos),
            (NameProvider::Injective, "inj", Chain::Injective),
            (NameProvider::Icns, "sei", Chain::Sei),
            (NameProvider::Icns, "cosmos", Chain::Cosmos),
            (NameProvider::Icns, "osmo", Chain::Osmosis),
            (NameProvider::Icns, "celestia", Chain::Celestia),
            (NameProvider::Lens, "lens", Chain::Ethereum),
            (NameProvider::Basenames, "base.eth", Chain::Base),
            (NameProvider::Hyperliquid, "hl", Chain::Ethereum),
            (NameProvider::AllDomains, "solana", Chain::Solana),
            (NameProvider::Near, "near", Chain::Near),
        ] {
            let name = format!("abcdefghijklmnopqrstu.{suffix}");
            let address = "0x0000000000000000000000000000000000000001";
            let client = NameClient::new(vec![Box::new(MockNameResolver::new(provider.clone(), vec![suffix], vec![chain], Ok(address)))]);

            assert_eq!(
                client.resolve(&name, chain).await.unwrap(),
                Some(NameRecord {
                    provider: provider.clone(),
                    address: address.to_string(),
                    name: name.clone(),
                    chain,
                }),
                "{name}"
            );

            let client = NameClient::new(vec![Box::new(MockNameResolver::new(provider, vec![suffix], vec![chain], Err("provider error")))]);

            assert_eq!(client.resolve(&name, chain).await.unwrap_err().to_string(), "provider error", "{name}");
        }
    }

    #[tokio::test]
    async fn test_resolve_unsupported_name_has_no_record() {
        let client = NameClient::new(vec![Box::new(MockNameResolver::new(NameProvider::Injective, vec!["inj"], vec![Chain::Injective], Err("provider called")))]);

        assert_eq!(client.resolve("alice.unknown", Chain::Injective).await.unwrap(), None);
        assert_eq!(client.resolve("abcdefghijklmnopqrstu.unknown", Chain::Injective).await.unwrap(), None);
        assert_eq!(client.resolve("abcdefghijklmnopqrstu.inj", Chain::Ton).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_resolve_returns_more_specific_provider_error() {
        let client = NameClient::new(vec![
            Box::new(MockNameResolver::new(NameProvider::Ens, vec!["*"], vec![Chain::Base], Ok("0x0000000000000000000000000000000000000003"))),
            Box::new(MockNameResolver::new(NameProvider::Basenames, vec!["base.eth"], vec![Chain::Base], Err("failed"))),
        ]);

        let error = client.resolve("alice.base.eth", Chain::Base).await.err().unwrap();

        assert_eq!(error.to_string(), "failed");
    }
}
