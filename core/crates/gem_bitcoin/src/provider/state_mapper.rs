use crate::models::block::BitcoinNodeInfo;
use primitives::{BitcoinChain, NodeSyncStatus};

trait BlockbookChain {
    fn blockbook_coin(self) -> &'static str;
}

impl BlockbookChain for BitcoinChain {
    fn blockbook_coin(self) -> &'static str {
        match self {
            BitcoinChain::Bitcoin => "Bitcoin",
            BitcoinChain::BitcoinCash => "Bcash",
            BitcoinChain::Litecoin => "Litecoin",
            BitcoinChain::Doge => "Dogecoin",
            BitcoinChain::Zcash => "Zcash",
        }
    }
}

pub fn map_chain_id(chain: BitcoinChain, node_info: &BitcoinNodeInfo) -> Result<String, &'static str> {
    if node_info.blockbook.coin == chain.blockbook_coin() {
        Ok(chain.get_chain().network_id().to_string())
    } else {
        Err("Invalid Bitcoin chain")
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
    use crate::models::block::BitcoinNodeInfo;
    use primitives::Chain;

    #[test]
    fn test_map_chain_id() {
        let bitcoin = BitcoinNodeInfo::mock("Bitcoin", true, 1, Some(1));
        assert_eq!(map_chain_id(BitcoinChain::Bitcoin, &bitcoin).unwrap(), Chain::Bitcoin.network_id());

        let bitcoin_cash = BitcoinNodeInfo::mock("Bcash", true, 1, Some(1));
        assert_eq!(map_chain_id(BitcoinChain::BitcoinCash, &bitcoin_cash).unwrap(), Chain::BitcoinCash.network_id());
        assert_eq!(map_chain_id(BitcoinChain::Bitcoin, &bitcoin_cash), Err("Invalid Bitcoin chain"));
    }

    #[test]
    fn test_map_node_status_returns_flag_and_block_numbers() {
        let node_info = BitcoinNodeInfo::mock("Bitcoin", false, 123, Some(456));

        let status = map_node_status(&node_info);

        assert!(!status.in_sync);
        assert_eq!(status.latest_block_number, Some(456));
        assert_eq!(status.current_block_number, Some(123));
    }

    #[test]
    fn test_map_latest_block_number_returns_best_height() {
        let node_info = BitcoinNodeInfo::mock("Bitcoin", true, 1_000, Some(2_000));
        assert_eq!(map_latest_block_number(&node_info), 1_000);
    }
}
