use std::error::Error;

use alloy_primitives::{Bytes, hex};
use gem_client::{Client, ClientExt};
use url::{Origin, Position, Url};

use super::contract::UniversalResolver::OffchainLookup;
use super::model::{CcipRequest, CcipResponse};

pub struct CcipClient<C: Client> {
    client: C,
    origin: Origin,
}

impl<C: Client> CcipClient<C> {
    pub fn new(client: C, url: &str) -> Self {
        let origin = Url::parse(url).map(|url| url.origin()).unwrap_or_else(|_| Origin::new_opaque());
        Self { client, origin }
    }

    pub async fn lookup(&self, lookup: &OffchainLookup) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let sender = hex::encode_prefixed(lookup.sender);
        let data = hex::encode_prefixed(&lookup.callData);
        let (template, url) = lookup
            .urls
            .iter()
            .find_map(|template| {
                let url = Url::parse(&template.replace("{sender}", &sender).replace("{data}", &data)).ok()?;
                (url.origin() == self.origin).then_some((template, url))
            })
            .ok_or("ENS offchain lookup has no supported CCIP URL")?;
        let path = &url[Position::BeforePath..Position::AfterQuery];
        let response: CcipResponse = if template.contains("{data}") {
            self.client.get(path).await?
        } else {
            self.client.post(path, &CcipRequest { sender, data }).await?
        };
        Ok(hex::decode(response.data)?.into())
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_lookup() {
        let lookup = OffchainLookup {
            urls: vec!["https://ccip-v3.ens.xyz/{sender}/{data}.json".into()],
            callData: vec![1, 2].into(),
            ..OffchainLookup::mock()
        };
        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/0xeeeeeeee14d718c2b47d9923deab1335e144eeee/0x0102.json");
            Ok(br#"{"data":"0x0304"}"#.to_vec())
        });

        assert_eq!(CcipClient::mock(client).lookup(&lookup).await.unwrap(), Bytes::from(vec![3, 4]));
    }

    #[tokio::test]
    async fn test_lookup_rejects_untrusted_urls() {
        for url in [
            "http://ccip-v3.ens.xyz/",
            "https://localhost/",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://169.254.169.254/",
            "https://ccip-v3.ens.xyz.evil.test/",
            "https://ccip-v3.ens.xyz:8443/",
            "x-batch-gateway:true",
            "invalid",
        ] {
            let lookup = OffchainLookup {
                urls: vec![url.into()],
                ..OffchainLookup::mock()
            };

            assert_eq!(CcipClient::mock(MockClient::new()).lookup(&lookup).await.unwrap_err().to_string(), "ENS offchain lookup has no supported CCIP URL", "{url}");
        }
    }

    #[tokio::test]
    async fn test_lookup_rejects_malformed_data() {
        let client = MockClient::new().with_post(|_, _| Ok(br#"{"data":"0xgg"}"#.to_vec()));

        assert!(CcipClient::mock(client).lookup(&OffchainLookup::mock()).await.is_err());
    }
}
