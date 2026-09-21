use crate::models::block::{BitcoinBackend, BitcoinBlockbook, BitcoinNodeInfo};

impl BitcoinNodeInfo {
    pub fn mock(coin: &str, in_sync: bool, best_height: u64, blocks: Option<u64>) -> Self {
        Self {
            blockbook: BitcoinBlockbook { coin: coin.into(), in_sync, best_height },
            backend: BitcoinBackend { blocks, consensus: None },
        }
    }
}
