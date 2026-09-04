use primitives::NodeSyncStatus;
use std::error::Error;

pub fn map_node_status(latest_block: i64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    Ok(NodeSyncStatus::synced(latest_block as u64))
}

pub fn map_chain_id(genesis_block_id: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let chain_id: [u8; 4] = *hex::decode(genesis_block_id)?.last_chunk().ok_or("Tron genesis block id is too short")?;
    Ok(format!("0x{}", hex::encode(chain_id)))
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_map_chain_id_takes_the_last_four_bytes_of_the_genesis_block() {
        assert_eq!(
            map_chain_id("00000000000000001ebf88508a03865c71d452e25f4d51194196a1d22b6653dc").unwrap(),
            Chain::Tron.network_id()
        );
        assert!(map_chain_id("6653dc").is_err());
        assert!(map_chain_id("not-hex").is_err());
    }

    #[test]
    fn test_map_node_status() {
        let latest_block = 12345i64;
        let mapped = map_node_status(latest_block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345));
        assert_eq!(mapped.current_block_number, Some(12345));
    }
}
