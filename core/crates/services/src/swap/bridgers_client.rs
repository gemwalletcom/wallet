use std::{error::Error, sync::Arc};

use cacher::SwapVaultAddressCacher;
use gem_ton::tvm::BagOfCells;
use primitives::SwapProvider;

use super::proxy_client::SwapProxyClient;

pub struct BridgersProxyClient {
    client: SwapProxyClient,
}

impl BridgersProxyClient {
    pub fn new(url: String, deposit_addresses: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::Bridgers, &["/data/txData/data"]),
        }
    }

    pub async fn swap(&self, body: serde_json::Value) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        let response = self.client.post("/api/sswap/swap", &body).await?;
        if let Some(deposit_id) = ton_deposit_id(&response.data) {
            self.client.cache_deposit_id(deposit_id).await;
        }
        Ok(response)
    }
}

fn ton_deposit_id(data: &[u8]) -> Option<String> {
    let response = serde_json::from_slice::<serde_json::Value>(data).ok()?;
    let payload = response.pointer("/data/txData/payload")?.as_str()?;
    Some(hex::encode(BagOfCells::parse_base64_root(payload).ok()?.hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_deposit_id() {
        let ton = include_bytes!("../../../swapper/src/bridgers/testdata/swap_ton_bsc_usdt.json");
        let evm = include_bytes!("../../../swapper/src/bridgers/testdata/swap_base_usdc_bsc_usdt.json");

        assert_eq!(ton_deposit_id(ton).as_deref(), Some("03649dcfca44d7fe76c5a22ff8c955097e5cb60d32fef4feb9c368bee9df9d15"));
        assert_eq!(ton_deposit_id(evm), None);
    }
}
