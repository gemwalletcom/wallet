use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::rpc::Block;

pub fn map_transaction_status(blocks: Vec<Block>, transaction_id: &str, block_number: u64) -> TransactionUpdate {
    for block in blocks {
        for extrinsic in block.extrinsics {
            if extrinsic.hash == transaction_id {
                let state = if extrinsic.success { TransactionState::Confirmed } else { TransactionState::Failed };
                return TransactionUpdate::new_state(state);
            }
        }
    }

    TransactionUpdate::new(TransactionState::Pending, vec![TransactionChange::BlockNumber(block_number.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{Block, Extrinsic};

    #[test]
    fn test_map_transaction_status_confirmed() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![Extrinsic::mock()],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_failed() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![Extrinsic {
                success: false,
                ..Extrinsic::mock()
            }],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Failed);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let blocks = vec![Block {
            number: 100,
            extrinsics: vec![Extrinsic {
                hash: "other_hash".to_string(),
                ..Extrinsic::mock()
            }],
        }];

        let result = map_transaction_status(blocks, "hash123", 100);
        assert_eq!(result.state, TransactionState::Pending);
    }
}
