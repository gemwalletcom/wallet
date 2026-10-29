use std::error::Error;

use alloy_primitives::{Bytes, hex};
use gem_client::{Client, ClientExt};
use serde::{Deserialize, Serialize};
use url::{Position, Url};

use super::contract::UniversalResolver::OffchainLookup;

#[derive(Serialize)]
struct CcipRequest {
    sender: String,
    data: String,
}

#[derive(Deserialize)]
struct CcipResponse {
    data: String,
}

pub async fn lookup<C: Client>(client: &C, url: &str, lookup: &OffchainLookup) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
    let origin = Url::parse(url)?.origin();
    let sender = hex::encode_prefixed(lookup.sender);
    let data = hex::encode_prefixed(&lookup.callData);
    for template in &lookup.urls {
        let Ok(url) = Url::parse(&template.replace("{sender}", &sender).replace("{data}", &data)) else {
            continue;
        };
        if url.origin() != origin {
            continue;
        }
        let path = &url[Position::BeforePath..Position::AfterQuery];
        let response: CcipResponse = if template.contains("{data}") {
            client.get(path).await?
        } else {
            client.post(path, &CcipRequest { sender, data }).await?
        };
        return Ok(hex::decode(response.data)?.into());
    }
    Err("ENS offchain lookup has no supported CCIP URL".into())
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::super::testkit::{CCIP_URL, offchain_lookup};
    use super::*;

    #[tokio::test]
    async fn test_ccip_get() {
        let mut lookup = offchain_lookup();
        lookup.urls = vec!["https://ccip-v3.ens.xyz/{sender}/{data}.json".into()];
        lookup.callData = vec![1, 2].into();
        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/0xeeeeeeee14d718c2b47d9923deab1335e144eeee/0x0102.json");
            Ok(br#"{"data":"0x0304"}"#.to_vec())
        });

        assert_eq!(super::lookup(&client, CCIP_URL, &lookup).await.unwrap(), Bytes::from(vec![3, 4]));
    }

    #[tokio::test]
    async fn test_ccip_rejects_untrusted_urls() {
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
            let mut lookup = offchain_lookup();
            lookup.urls = vec![url.into()];

            assert_eq!(super::lookup(&MockClient::new(), CCIP_URL, &lookup).await.unwrap_err().to_string(), "ENS offchain lookup has no supported CCIP URL", "{url}");
        }
    }

    #[tokio::test]
    async fn test_ccip_rejects_malformed_data() {
        let lookup = offchain_lookup();
        let client = MockClient::new().with_post(|_, _| Ok(br#"{"data":"0xgg"}"#.to_vec()));

        assert!(super::lookup(&client, CCIP_URL, &lookup).await.is_err());
    }
}
