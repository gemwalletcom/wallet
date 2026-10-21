use std::{error::Error, sync::Arc};

use cacher::SwapVaultAddressCacher;
use gem_ton::tvm::BagOfCells;
use gem_xrp::models::{TransactionMemo, memos_hash};
use primitives::SwapProvider;
use serde::Deserialize;

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
        if let Some(deposit_id) = deposit_id(&response.data) {
            self.client.cache_deposit_id(deposit_id).await;
        }
        Ok(response)
    }
}

fn deposit_id(data: &[u8]) -> Option<String> {
    let response = serde_json::from_slice::<serde_json::Value>(data).ok()?;
    let tx_data = response.pointer("/data/txData")?;
    ton_deposit_id(tx_data).or_else(|| xrp_deposit_id(tx_data))
}

fn ton_deposit_id(tx_data: &serde_json::Value) -> Option<String> {
    let payload = tx_data.get("payload")?.as_str()?;
    Some(hex::encode(BagOfCells::parse_base64_root(payload).ok()?.hash))
}

fn xrp_deposit_id(tx_data: &serde_json::Value) -> Option<String> {
    memos_hash(&Vec::<TransactionMemo>::deserialize(tx_data.get("Memos")?).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_id() {
        let ton = include_bytes!("../../../swapper/src/bridgers/testdata/swap_ton_bsc_usdt.json");
        let xrp = include_bytes!("../../../swapper/src/bridgers/testdata/swap_xrp_bsc_usdt.json");
        let rlusd = include_bytes!("../../../swapper/src/bridgers/testdata/swap_rlusd_bsc_usdt.json");
        let evm = include_bytes!("../../../swapper/src/bridgers/testdata/swap_base_usdc_bsc_usdt.json");

        assert_eq!(deposit_id(ton).as_deref(), Some("03649dcfca44d7fe76c5a22ff8c955097e5cb60d32fef4feb9c368bee9df9d15"));
        assert_eq!(deposit_id(xrp).as_deref(), Some("3fed38eafa6b3bf232aaf79bd5ce2e20a8e41769eb1759a44a7f1ede8989a995"));
        assert_eq!(deposit_id(rlusd).as_deref(), Some("6d4a249eec51b33a5dc11e60d1cc6f7653c2d85474986e7c31ac02b1ae392366"));
        assert_eq!(deposit_id(evm), None);
    }
}
