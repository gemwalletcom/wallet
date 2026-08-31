use std::error::Error;

use primitives::NodeSyncStatus;

use crate::models::rpc::LedgerInfo;

pub fn map_node_status(ledger_info: &LedgerInfo) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    Ok(NodeSyncStatus::synced(ledger_info.ledger_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_node_status() {
        let ledger_info = LedgerInfo {
            ledger_index: 80123456,
            validated: true,
        };
        let mapped = map_node_status(&ledger_info).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(80123456));
        assert_eq!(mapped.current_block_number, Some(80123456));
    }
}
