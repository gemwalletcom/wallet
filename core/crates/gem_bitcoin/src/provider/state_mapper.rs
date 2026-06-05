use crate::models::block::BitcoinNodeInfo;
use primitives::{Asset, BitcoinChain, NodeSyncStatus};

pub fn map_chain_id(chain: BitcoinChain, node_info: &BitcoinNodeInfo) -> Result<String, &'static str> {
    let chain = chain.get_chain();
    let asset = Asset::from_chain(chain);

    if node_info.blockbook.network.as_deref() == Some(asset.symbol.as_str()) {
        Ok(chain.network_id().to_string())
    } else {
        Err("Invalid Bitcoin network")
    }
}

pub fn map_node_status(node_info: &BitcoinNodeInfo) -> NodeSyncStatus {
    let latest_block_number = node_info.backend.blocks;
    let current_block_number = Some(node_info.blockbook.best_height);

    NodeSyncStatus::new(node_info.blockbook.in_sync, latest_block_number, current_block_number)
}

pub fn map_latest_block_number(node_info: &BitcoinNodeInfo) -> u64 {
    node_info.blockbook.best_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::block::{BitcoinBackend, BitcoinBlockbook, BitcoinNodeInfo};
    use primitives::{Asset, Chain};

    #[test]
    fn test_map_chain_id() {
        let bitcoin = node_info(Some(Asset::from_chain(Chain::Bitcoin).symbol), true, 1, Some(1));
        assert_eq!(map_chain_id(BitcoinChain::Bitcoin, &bitcoin).unwrap(), Chain::Bitcoin.network_id());

        let doge = node_info(Some(Asset::from_chain(Chain::Doge).symbol), true, 1, Some(1));
        assert_eq!(map_chain_id(BitcoinChain::Doge, &doge).unwrap(), Chain::Doge.network_id());
        assert_eq!(map_chain_id(BitcoinChain::Bitcoin, &doge), Err("Invalid Bitcoin network"));
    }

    #[test]
    fn test_map_node_status_returns_flag_and_block_numbers() {
        let node_info = node_info(Some(Asset::from_chain(Chain::Bitcoin).symbol), false, 123, Some(456));

        let status = map_node_status(&node_info);

        assert!(!status.in_sync);
        assert_eq!(status.latest_block_number, Some(456));
        assert_eq!(status.current_block_number, Some(123));
    }

    #[test]
    fn test_map_latest_block_number_returns_best_height() {
        let node_info = node_info(Some(Asset::from_chain(Chain::Bitcoin).symbol), true, 1_000, Some(2_000));
        assert_eq!(map_latest_block_number(&node_info), 1_000);
    }

    fn node_info(network: Option<String>, in_sync: bool, best_height: u64, blocks: Option<u64>) -> BitcoinNodeInfo {
        BitcoinNodeInfo {
            blockbook: BitcoinBlockbook { network, in_sync, best_height },
            backend: BitcoinBackend { blocks, consensus: None },
        }
    }
}
