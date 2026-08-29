use primitives::{AssetId, PerpetualDirection, Transaction, TransactionDirection, TransactionPerpetualMetadata, TransactionState, TransactionType};

use super::model::GemTransactionTitle;
use crate::services::collections::unique;

pub fn transaction_asset_ids(transactions: &[Transaction]) -> Vec<AssetId> {
    unique(transactions.iter().flat_map(|transaction| transaction.associated_asset_ids()))
}

pub fn transaction_title(transaction: &Transaction) -> GemTransactionTitle {
    match transaction.transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT => transfer_title(transaction),
        TransactionType::SmartContractCall => GemTransactionTitle::SmartContract,
        TransactionType::Swap => GemTransactionTitle::Swap,
        TransactionType::TokenApproval => GemTransactionTitle::Approve,
        TransactionType::StakeDelegate => GemTransactionTitle::Stake,
        TransactionType::StakeUndelegate => GemTransactionTitle::Unstake,
        TransactionType::StakeRedelegate => GemTransactionTitle::Redelegate,
        TransactionType::StakeRewards => GemTransactionTitle::Rewards,
        TransactionType::StakeWithdraw | TransactionType::EarnWithdraw => GemTransactionTitle::Withdraw,
        TransactionType::AssetActivation => GemTransactionTitle::ActivateAsset,
        TransactionType::StakeFreeze => GemTransactionTitle::Freeze,
        TransactionType::StakeUnfreeze => GemTransactionTitle::Unfreeze,
        TransactionType::EarnDeposit => GemTransactionTitle::Earn,
        TransactionType::PerpetualOpenPosition => GemTransactionTitle::PerpetualOpen {
            direction: perpetual_direction(transaction),
        },
        TransactionType::PerpetualClosePosition => GemTransactionTitle::PerpetualClose {
            direction: perpetual_direction(transaction),
        },
        TransactionType::PerpetualModifyPosition => GemTransactionTitle::PerpetualModify,
    }
}

fn transfer_title(transaction: &Transaction) -> GemTransactionTitle {
    if transaction.state != TransactionState::Confirmed {
        return GemTransactionTitle::Transfer;
    }
    match transaction.direction {
        TransactionDirection::Incoming => GemTransactionTitle::Received,
        TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => GemTransactionTitle::Sent,
    }
}

fn perpetual_direction(transaction: &Transaction) -> Option<PerpetualDirection> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionPerpetualMetadata>(metadata).ok().map(|metadata| metadata.direction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use primitives::Chain;

    fn transaction(asset_id: AssetId, fee_asset_id: AssetId) -> Transaction {
        Transaction::new(
            "hash".into(),
            asset_id,
            "from".into(),
            "to".into(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "1".into(),
            fee_asset_id,
            "1".into(),
            None,
            None,
            Utc::now(),
        )
    }

    fn typed(transaction_type: TransactionType, state: TransactionState, direction: TransactionDirection) -> Transaction {
        let mut transaction = transaction(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum));
        transaction.transaction_type = transaction_type;
        transaction.state = state;
        transaction.direction = direction;
        transaction
    }

    #[test]
    fn test_transaction_title_reads_a_transfer_from_its_state_and_direction() {
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};
        use TransactionState::{Confirmed, Failed, InTransit, Pending};

        assert_eq!(transaction_title(&typed(TransactionType::Transfer, Confirmed, Incoming)), GemTransactionTitle::Received);
        assert_eq!(transaction_title(&typed(TransactionType::Transfer, Confirmed, Outgoing)), GemTransactionTitle::Sent);
        assert_eq!(transaction_title(&typed(TransactionType::Transfer, Confirmed, SelfTransfer)), GemTransactionTitle::Sent);
        assert_eq!(transaction_title(&typed(TransactionType::TransferNFT, Confirmed, Incoming)), GemTransactionTitle::Received);

        for state in [Pending, Failed, InTransit] {
            assert_eq!(transaction_title(&typed(TransactionType::Transfer, state, Incoming)), GemTransactionTitle::Transfer);
        }
    }

    #[test]
    fn test_transaction_title_separates_earn_from_stake() {
        let confirmed = |transaction_type| typed(transaction_type, TransactionState::Confirmed, TransactionDirection::Outgoing);

        assert_eq!(transaction_title(&confirmed(TransactionType::StakeDelegate)), GemTransactionTitle::Stake);
        assert_eq!(transaction_title(&confirmed(TransactionType::EarnDeposit)), GemTransactionTitle::Earn);
        assert_eq!(transaction_title(&confirmed(TransactionType::EarnWithdraw)), GemTransactionTitle::Withdraw);
        assert_eq!(transaction_title(&confirmed(TransactionType::StakeWithdraw)), GemTransactionTitle::Withdraw);
    }

    #[test]
    fn test_transaction_title_carries_the_perpetual_direction_when_the_metadata_has_one() {
        let mut open = typed(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_title(&open), GemTransactionTitle::PerpetualOpen { direction: None });

        open.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                pnl: 0.0,
                price: 1.0,
                direction: PerpetualDirection::Short,
                is_liquidation: None,
                provider: None,
            })
            .unwrap(),
        );
        assert_eq!(
            transaction_title(&open),
            GemTransactionTitle::PerpetualOpen {
                direction: Some(PerpetualDirection::Short)
            }
        );

        let mut close = open.clone();
        close.transaction_type = TransactionType::PerpetualClosePosition;
        assert_eq!(
            transaction_title(&close),
            GemTransactionTitle::PerpetualClose {
                direction: Some(PerpetualDirection::Short)
            }
        );
    }

    #[test]
    fn test_transaction_asset_ids_includes_fee_assets_once() {
        let solana = AssetId::from_chain(Chain::Solana);
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let usdc = AssetId::from_token(Chain::Solana, "usdc");

        let mut asset_ids = transaction_asset_ids(&[transaction(usdc.clone(), solana.clone()), transaction(ethereum.clone(), ethereum.clone())]);
        asset_ids.sort_by_key(|asset_id| asset_id.to_string());
        let mut expected = vec![usdc, solana, ethereum];
        expected.sort_by_key(|asset_id| asset_id.to_string());

        assert_eq!(asset_ids, expected);
    }
}
