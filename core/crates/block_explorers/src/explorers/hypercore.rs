use primitives::AssetId;

use crate::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct HyperliquidExplorer;
pub struct HypurrScan;
pub struct FlowScan;

impl HyperliquidExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        HyperCoreExplorer::boxed(Metadata::with_token("Hyperliquid", "https://app.hyperliquid.xyz/explorer"))
    }
}

impl HypurrScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        HyperCoreExplorer::boxed(Metadata::new("HypurrScan", "https://hypurrscan.io"))
    }
}

impl FlowScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        HyperCoreExplorer::boxed(Metadata::new("FlowScan", "https://www.flowscan.xyz"))
    }
}

struct HyperCoreExplorer {
    explorer: Box<dyn BlockExplorer>,
}

impl HyperCoreExplorer {
    fn boxed(metadata: Metadata) -> Box<dyn BlockExplorer> {
        Box::new(Self { explorer: Explorer::boxed(metadata) })
    }
}

impl BlockExplorer for HyperCoreExplorer {
    fn name(&self) -> String {
        self.explorer.name()
    }

    fn get_tx_url(&self, transaction_id: &str) -> String {
        let hash = transaction_id.split_once(':').map_or(transaction_id, |(hash, _)| hash);
        self.explorer.get_tx_url(hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        self.explorer.get_address_url(address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        match AssetId::decode_token_id(token).as_slice() {
            [_, token_id, _] => self.explorer.get_token_url(token_id),
            _ => None,
        }
    }

    fn get_nft_url(&self, contract: &str, token_id: &str) -> Option<String> {
        self.explorer.get_nft_url(contract, token_id)
    }
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::{HYPERCORE_PERPETUAL_USDC_TOKEN_ID, HYPERCORE_SPOT_HYPE_TOKEN_ID};

    use super::*;

    #[test]
    fn test_hyperliquid_explorer_tx_url() {
        let explorer = HyperliquidExplorer::boxed();
        let tx_hash = "0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86";
        let result = explorer.get_tx_url(tx_hash);

        assert_eq!(result, "https://app.hyperliquid.xyz/explorer/tx/0x144bb14b70b1ea80c06a0427e862140000ea2b7bf051872ce50dd920fd547b86");
        assert_eq!(explorer.get_tx_url(&format!("{tx_hash}:327271591086")), result);
    }

    #[test]
    fn test_hyperliquid_explorer_address_url() {
        let explorer = HyperliquidExplorer::boxed();
        let address = "0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce";
        let result = explorer.get_address_url(address);

        assert_eq!(result, "https://app.hyperliquid.xyz/explorer/address/0x953cb34f310cdef2ec0351e8c20e87bd53bd3bce");
    }

    #[test]
    fn test_hyperliquid_explorer_token_url() {
        let explorer = HyperliquidExplorer::boxed();

        assert_eq!(
            explorer.get_token_url(HYPERCORE_SPOT_HYPE_TOKEN_ID),
            Some("https://app.hyperliquid.xyz/explorer/token/0x0d01dc56dcaaca66ad901c959b4011ec".to_string())
        );
        assert_eq!(explorer.get_token_url(HYPERCORE_PERPETUAL_USDC_TOKEN_ID), None);
    }

    #[test]
    fn test_hypurrscan_urls() {
        let explorer = HypurrScan::boxed();
        let tx_hash = "0x90effdc0864193549269042ff91b3702038900a62144b22634b8a91345456d3f";
        let address = "0xE4bfadD038B5ec2cab0e5F0354F2249cCF5d38eE";

        assert_eq!(explorer.get_tx_url(tx_hash), "https://hypurrscan.io/tx/0x90effdc0864193549269042ff91b3702038900a62144b22634b8a91345456d3f");
        assert_eq!(explorer.get_tx_url(&format!("{tx_hash}:327271591086")), explorer.get_tx_url(tx_hash));
        assert_eq!(explorer.get_address_url(address), "https://hypurrscan.io/address/0xE4bfadD038B5ec2cab0e5F0354F2249cCF5d38eE");
    }

    #[test]
    fn test_flowscan_urls() {
        let explorer = FlowScan::boxed();
        let tx_hash = "0x09f4a204b1230fbd0b6e043023ef7200002fb9ea4c262e8fadbd4d577026e9a7";
        let address = "0xE4bfadD038B5ec2cab0e5F0354F2249cCF5d38eE";

        assert_eq!(explorer.get_tx_url(tx_hash), "https://www.flowscan.xyz/tx/0x09f4a204b1230fbd0b6e043023ef7200002fb9ea4c262e8fadbd4d577026e9a7");
        assert_eq!(explorer.get_tx_url(&format!("{tx_hash}:327271591086")), explorer.get_tx_url(tx_hash));
        assert_eq!(explorer.get_address_url(address), "https://www.flowscan.xyz/address/0xE4bfadD038B5ec2cab0e5F0354F2249cCF5d38eE");
    }
}
