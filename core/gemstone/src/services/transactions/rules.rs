use primitives::{
    AssetId, PerpetualDirection, Transaction, TransactionDirection, TransactionPerpetualMetadata, TransactionResourceTypeMetadata, TransactionState, TransactionType,
    TransactionWalletConnectMetadata, TransferDataOutputAction,
};

use super::model::{GemAmountSign, GemTransactionHeaderKind, GemTransactionParticipantRole, GemTransactionSubtitle, GemTransactionTitle, GemTransactionValue};
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

pub fn transaction_subtitle(transaction: &Transaction) -> GemTransactionSubtitle {
    match transaction.transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT | TransactionType::TokenApproval | TransactionType::SmartContractCall => match transaction.direction {
            TransactionDirection::Incoming => GemTransactionSubtitle::FromAddress {
                address: transaction.from.clone(),
            },
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => GemTransactionSubtitle::ToAddress { address: transaction.to.clone() },
        },
        TransactionType::StakeDelegate | TransactionType::StakeRedelegate | TransactionType::EarnDeposit => GemTransactionSubtitle::ToAddress { address: transaction.to.clone() },
        TransactionType::StakeUndelegate | TransactionType::EarnWithdraw => GemTransactionSubtitle::FromAddress { address: transaction.to.clone() },
        TransactionType::StakeFreeze => resource(transaction).map_or(GemTransactionSubtitle::None, |resource| GemTransactionSubtitle::ToResource { resource }),
        TransactionType::StakeUnfreeze => resource(transaction).map_or(GemTransactionSubtitle::None, |resource| GemTransactionSubtitle::FromResource { resource }),
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => {
            match perpetual_metadata(transaction).map(|metadata| metadata.price).filter(|price| *price > 0.0) {
                Some(value) => GemTransactionSubtitle::Price { value },
                None => GemTransactionSubtitle::None,
            }
        }
        TransactionType::Swap | TransactionType::StakeRewards | TransactionType::StakeWithdraw | TransactionType::AssetActivation => GemTransactionSubtitle::None,
    }
}

pub fn transaction_participant(transaction: &Transaction) -> Option<(GemTransactionParticipantRole, String)> {
    let role = match transaction.transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT => match transaction.direction {
            TransactionDirection::Incoming => GemTransactionParticipantRole::Sender,
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => GemTransactionParticipantRole::Recipient,
        },
        TransactionType::TokenApproval => GemTransactionParticipantRole::Contract,
        TransactionType::SmartContractCall => match wallet_connect_metadata(transaction).map(|metadata| metadata.output_action) {
            Some(TransferDataOutputAction::Send) => GemTransactionParticipantRole::Recipient,
            Some(TransferDataOutputAction::Sign) | None => GemTransactionParticipantRole::Contract,
        },
        TransactionType::StakeDelegate => GemTransactionParticipantRole::Validator,
        TransactionType::EarnDeposit | TransactionType::EarnWithdraw => GemTransactionParticipantRole::Provider,
        TransactionType::Swap
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRedelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze
        | TransactionType::AssetActivation
        | TransactionType::PerpetualOpenPosition
        | TransactionType::PerpetualClosePosition
        | TransactionType::PerpetualModifyPosition => return None,
    };
    let address = match transaction.direction {
        TransactionDirection::Incoming => &transaction.from,
        TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => &transaction.to,
    };
    (!address.is_empty()).then(|| (role, address.clone()))
}

pub fn transaction_value(transaction: &Transaction) -> GemTransactionValue {
    match transaction.transaction_type {
        TransactionType::Swap => GemTransactionValue::SwapReceived,
        TransactionType::TokenApproval => GemTransactionValue::AssetSymbol,
        TransactionType::PerpetualOpenPosition => GemTransactionValue::PerpetualNotional,
        TransactionType::PerpetualClosePosition => match perpetual_metadata(transaction).map(|metadata| metadata.pnl).filter(|pnl| *pnl != 0.0) {
            Some(value) => GemTransactionValue::PerpetualPnl { value },
            None => GemTransactionValue::None,
        },
        TransactionType::StakeRewards | TransactionType::StakeWithdraw => GemTransactionValue::Amount { sign: GemAmountSign::Incoming },
        TransactionType::Transfer => GemTransactionValue::Amount {
            sign: amount_sign(&transaction.direction),
        },
        TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRedelegate
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze
        | TransactionType::EarnDeposit
        | TransactionType::EarnWithdraw
        | TransactionType::AssetActivation
        | TransactionType::SmartContractCall => GemTransactionValue::Amount { sign: GemAmountSign::None },
        TransactionType::TransferNFT | TransactionType::PerpetualModifyPosition => GemTransactionValue::None,
    }
}

pub fn transaction_equivalent_value(transaction: &Transaction) -> GemTransactionValue {
    match transaction.transaction_type {
        TransactionType::Swap => GemTransactionValue::SwapSpent,
        _ => GemTransactionValue::None,
    }
}

fn amount_sign(direction: &TransactionDirection) -> GemAmountSign {
    match direction {
        TransactionDirection::Incoming => GemAmountSign::Incoming,
        TransactionDirection::Outgoing => GemAmountSign::Outgoing,
        TransactionDirection::SelfTransfer => GemAmountSign::None,
    }
}

fn resource(transaction: &Transaction) -> Option<primitives::Resource> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionResourceTypeMetadata>(metadata)
        .ok()
        .map(|metadata| metadata.resource_type)
}

pub fn header_kind(transaction: &Transaction) -> GemTransactionHeaderKind {
    match transaction.transaction_type {
        TransactionType::Transfer
        | TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRedelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze
        | TransactionType::EarnDeposit
        | TransactionType::EarnWithdraw
        | TransactionType::SmartContractCall => GemTransactionHeaderKind::Amount { shows_fiat: true },
        TransactionType::Swap => match transaction.swap_metadata() {
            Some(_) => GemTransactionHeaderKind::Swap,
            None => GemTransactionHeaderKind::Amount { shows_fiat: true },
        },
        TransactionType::TransferNFT => match transaction.nft_asset_id() {
            Some(_) => GemTransactionHeaderKind::Nft,
            None => GemTransactionHeaderKind::Amount { shows_fiat: false },
        },
        TransactionType::TokenApproval => GemTransactionHeaderKind::AssetImage,
        TransactionType::AssetActivation | TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => {
            GemTransactionHeaderKind::Symbol
        }
    }
}

fn wallet_connect_metadata(transaction: &Transaction) -> Option<TransactionWalletConnectMetadata> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionWalletConnectMetadata>(metadata).ok()
}

fn perpetual_metadata(transaction: &Transaction) -> Option<TransactionPerpetualMetadata> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionPerpetualMetadata>(metadata).ok()
}

fn perpetual_direction(transaction: &Transaction) -> Option<PerpetualDirection> {
    perpetual_metadata(transaction).map(|metadata| metadata.direction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use num_bigint::BigUint;
    use primitives::{Chain, Resource};

    fn transaction(asset_id: AssetId, fee_asset_id: AssetId) -> Transaction {
        Transaction::new(
            "hash".into(),
            asset_id,
            "from".into(),
            "to".into(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(1u64),
            fee_asset_id,
            BigUint::from(1u64),
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
    fn test_transaction_subtitle_names_the_counterparty_the_row_shows() {
        use TransactionDirection::{Incoming, Outgoing};

        let confirmed = |transaction_type, direction| typed(transaction_type, TransactionState::Confirmed, direction);

        assert_eq!(
            transaction_subtitle(&confirmed(TransactionType::Transfer, Incoming)),
            GemTransactionSubtitle::FromAddress { address: "from".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&confirmed(TransactionType::Transfer, Outgoing)),
            GemTransactionSubtitle::ToAddress { address: "to".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&confirmed(TransactionType::StakeDelegate, Outgoing)),
            GemTransactionSubtitle::ToAddress { address: "to".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&confirmed(TransactionType::StakeUndelegate, Outgoing)),
            GemTransactionSubtitle::FromAddress { address: "to".to_string() }
        );
        assert_eq!(transaction_subtitle(&confirmed(TransactionType::Swap, Outgoing)), GemTransactionSubtitle::None);
        assert_eq!(transaction_subtitle(&confirmed(TransactionType::StakeRewards, Incoming)), GemTransactionSubtitle::None);
    }

    #[test]
    fn test_transaction_subtitle_reads_the_resource_and_the_price_from_the_metadata() {
        let mut freeze = typed(TransactionType::StakeFreeze, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_subtitle(&freeze), GemTransactionSubtitle::None);

        freeze.metadata = Some(serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Energy)).unwrap());
        assert_eq!(transaction_subtitle(&freeze), GemTransactionSubtitle::ToResource { resource: Resource::Energy });

        let mut unfreeze = freeze.clone();
        unfreeze.transaction_type = TransactionType::StakeUnfreeze;
        assert_eq!(transaction_subtitle(&unfreeze), GemTransactionSubtitle::FromResource { resource: Resource::Energy });

        let mut open = typed(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_subtitle(&open), GemTransactionSubtitle::None);

        open.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                pnl: 0.0,
                price: 12.5,
                direction: PerpetualDirection::Long,
                is_liquidation: None,
                provider: None,
            })
            .unwrap(),
        );
        assert_eq!(transaction_subtitle(&open), GemTransactionSubtitle::Price { value: 12.5 });
    }

    #[test]
    fn test_header_kind_falls_back_to_an_amount_without_metadata() {
        let mut swap = Transaction::mock();
        swap.transaction_type = TransactionType::Swap;
        swap.metadata = None;
        assert_eq!(header_kind(&swap), GemTransactionHeaderKind::Amount { shows_fiat: true });
        swap.metadata = Some(
            serde_json::to_value(primitives::TransactionSwapMetadata {
                from_asset: AssetId::from_chain(primitives::Chain::Ethereum),
                from_value: 1u32.into(),
                to_asset: AssetId::from_chain(primitives::Chain::Bitcoin),
                to_value: 1u32.into(),
                provider: None,
            })
            .unwrap(),
        );
        assert_eq!(header_kind(&swap), GemTransactionHeaderKind::Swap);
        let mut approval = Transaction::mock();
        approval.transaction_type = TransactionType::TokenApproval;
        assert_eq!(header_kind(&approval), GemTransactionHeaderKind::AssetImage);
    }

    #[test]
    fn test_transaction_participant_names_the_role_of_the_address_the_screen_shows() {
        use GemTransactionParticipantRole::{Contract, Provider, Recipient, Sender, Validator};
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};

        let confirmed = |transaction_type, direction| typed(transaction_type, TransactionState::Confirmed, direction);
        let participant = |transaction_type, direction| transaction_participant(&confirmed(transaction_type, direction));

        assert_eq!(participant(TransactionType::Transfer, Incoming), Some((Sender, "from".to_string())));
        assert_eq!(participant(TransactionType::Transfer, Outgoing), Some((Recipient, "to".to_string())));
        assert_eq!(participant(TransactionType::TransferNFT, SelfTransfer), Some((Recipient, "to".to_string())));
        assert_eq!(participant(TransactionType::TokenApproval, Outgoing), Some((Contract, "to".to_string())));
        assert_eq!(participant(TransactionType::StakeDelegate, Outgoing), Some((Validator, "to".to_string())));
        assert_eq!(participant(TransactionType::EarnWithdraw, Outgoing), Some((Provider, "to".to_string())));
        assert_eq!(participant(TransactionType::SmartContractCall, Outgoing), Some((Contract, "to".to_string())));
        assert_eq!(participant(TransactionType::Swap, Outgoing), None);
        assert_eq!(participant(TransactionType::StakeUndelegate, Outgoing), None);
        assert_eq!(participant(TransactionType::StakeFreeze, Outgoing), None);

        let mut send = confirmed(TransactionType::SmartContractCall, Outgoing);
        send.metadata = Some(
            serde_json::to_value(TransactionWalletConnectMetadata {
                output_action: TransferDataOutputAction::Send,
            })
            .unwrap(),
        );
        assert_eq!(transaction_participant(&send), Some((Recipient, "to".to_string())));
        send.metadata = Some(
            serde_json::to_value(TransactionWalletConnectMetadata {
                output_action: TransferDataOutputAction::Sign,
            })
            .unwrap(),
        );
        assert_eq!(transaction_participant(&send), Some((Contract, "to".to_string())));

        let mut blank = confirmed(TransactionType::Transfer, Outgoing);
        blank.to = String::new();
        assert_eq!(transaction_participant(&blank), None);
    }

    #[test]
    fn test_transaction_value_signs_what_the_row_shows() {
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};

        let confirmed = |transaction_type, direction| typed(transaction_type, TransactionState::Confirmed, direction);

        assert_eq!(
            transaction_value(&confirmed(TransactionType::Transfer, Incoming)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&confirmed(TransactionType::Transfer, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Outgoing }
        );
        assert_eq!(
            transaction_value(&confirmed(TransactionType::Transfer, SelfTransfer)),
            GemTransactionValue::Amount { sign: GemAmountSign::None }
        );
        assert_eq!(
            transaction_value(&confirmed(TransactionType::StakeRewards, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&confirmed(TransactionType::StakeWithdraw, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&confirmed(TransactionType::StakeDelegate, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::None }
        );
        assert_eq!(transaction_value(&confirmed(TransactionType::TokenApproval, Outgoing)), GemTransactionValue::AssetSymbol);
        assert_eq!(transaction_value(&confirmed(TransactionType::TransferNFT, Incoming)), GemTransactionValue::None);
    }

    #[test]
    fn test_transaction_value_gives_a_swap_both_legs_and_a_perpetual_close_only_a_real_pnl() {
        let mut swap = typed(TransactionType::Swap, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_value(&swap), GemTransactionValue::SwapReceived);
        assert_eq!(transaction_equivalent_value(&swap), GemTransactionValue::SwapSpent);

        swap.transaction_type = TransactionType::Transfer;
        assert_eq!(transaction_equivalent_value(&swap), GemTransactionValue::None);

        let mut close = typed(TransactionType::PerpetualClosePosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_value(&close), GemTransactionValue::None);

        let metadata = |pnl| TransactionPerpetualMetadata {
            pnl,
            price: 1.0,
            direction: PerpetualDirection::Long,
            is_liquidation: None,
            provider: None,
        };

        close.metadata = Some(serde_json::to_value(metadata(0.0)).unwrap());
        assert_eq!(transaction_value(&close), GemTransactionValue::None);

        close.metadata = Some(serde_json::to_value(metadata(-4.5)).unwrap());
        assert_eq!(transaction_value(&close), GemTransactionValue::PerpetualPnl { value: -4.5 });
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
