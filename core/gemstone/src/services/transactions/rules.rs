use std::str::FromStr;

use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetId, AssetPrice, AssetType, BlockExplorerLink, Chain, PerpetualDirection, Price, Transaction, TransactionDirection, TransactionExtended,
    TransactionNFTTransferMetadata, TransactionPerpetualMetadata, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionType,
    TransactionWalletConnectMetadata, TransferDataOutputAction,
};

use super::model::{
    GemAmountSign, GemSwapAgain, GemSwapProgress, GemSwapProgressStep, GemSwapRate, GemTransactionAmount, GemTransactionDetailRows, GemTransactionDetails, GemTransactionHeader,
    GemTransactionHeaderAction, GemTransactionHeaderKind, GemTransactionParticipant, GemTransactionParticipantRole, GemTransactionRow, GemTransactionRowSubtitle,
    GemTransactionRowValue, GemTransactionSubtitle, GemTransactionTitle, GemTransactionValue,
};
use crate::config::image::GemImage;
use crate::models::asset::wallet_default_assets;
use crate::services::collections::unique;
use swapper::{ProviderType as SwapperProviderType, SwapperProvider, SwapperProviderMode};

pub fn pending_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions.iter().filter(|transaction| !transaction.state.is_completed()).cloned().collect()
}

pub fn transaction_asset_ids(transactions: &[Transaction]) -> Vec<AssetId> {
    unique(transactions.iter().flat_map(|transaction| transaction.associated_asset_ids()))
}

pub fn row(extended: &TransactionExtended) -> GemTransactionRow {
    let transaction = &extended.transaction;
    GemTransactionRow {
        title: transaction_title(transaction),
        subtitle: row_subtitle(extended),
        value: row_value(extended, transaction_value(transaction)),
        equivalent_value: row_value(extended, transaction_equivalent_value(transaction)),
        nft_image_url: transaction.nft_asset_id().map(|asset_id| GemImage::NftAsset { asset_id: asset_id.to_string() }.url()),
    }
}

pub fn participant(extended: &TransactionExtended, link: impl FnOnce(&str) -> BlockExplorerLink) -> Option<GemTransactionParticipant> {
    let transaction = &extended.transaction;
    let (role, address) = transaction_participant(transaction)?;
    let name = address_name(extended, &address);
    let can_add_contact = name.is_none() && matches!(transaction.transaction_type, TransactionType::Transfer | TransactionType::TransferNFT);
    Some(GemTransactionParticipant {
        role,
        link: link(&address),
        name,
        address,
        can_add_contact,
    })
}

pub fn detail_rows(extended: &TransactionExtended, participant: Option<GemTransactionParticipant>, explorer: BlockExplorerLink) -> GemTransactionDetailRows {
    let transaction = &extended.transaction;
    let details = details(extended);
    GemTransactionDetailRows {
        title: transaction_title(transaction),
        header: header(extended),
        header_action: header_action(transaction),
        swap_progress: details.swap_progress,
        swap_again: details.swap_again,
        estimated_confirmation_seconds: details.estimated_confirmation_seconds,
        participant,
        provider_name: details.provider_name,
        memo: transaction.memo.clone().filter(|memo| !memo.is_empty()),
        resource: resource(transaction),
        rate: swap_rate(extended),
        pnl: details.pnl,
        price: details.price,
        fee: GemTransactionAmount {
            asset: extended.fee_asset.clone(),
            value: transaction.fee.clone(),
            sign: GemAmountSign::None,
            price: asset_price(extended.fee_price.as_ref(), &extended.fee_asset.id),
        },
        explorer,
    }
}

fn row_subtitle(extended: &TransactionExtended) -> GemTransactionRowSubtitle {
    match transaction_subtitle(&extended.transaction) {
        GemTransactionSubtitle::None => GemTransactionRowSubtitle::None,
        GemTransactionSubtitle::ToAddress { address } => GemTransactionRowSubtitle::ToAddress {
            name: address_name(extended, &address).map(|name| name.name),
            address,
        },
        GemTransactionSubtitle::FromAddress { address } => GemTransactionRowSubtitle::FromAddress {
            name: address_name(extended, &address).map(|name| name.name),
            address,
        },
        GemTransactionSubtitle::ToResource { resource } => GemTransactionRowSubtitle::ToResource { resource },
        GemTransactionSubtitle::FromResource { resource } => GemTransactionRowSubtitle::FromResource { resource },
        GemTransactionSubtitle::Price { value } => GemTransactionRowSubtitle::Price { value },
    }
}

fn row_value(extended: &TransactionExtended, value: GemTransactionValue) -> GemTransactionRowValue {
    let transaction = &extended.transaction;
    match value {
        GemTransactionValue::None => GemTransactionRowValue::None,
        GemTransactionValue::AssetSymbol => GemTransactionRowValue::AssetSymbol { asset: extended.asset.clone() },
        GemTransactionValue::Amount { sign } => GemTransactionRowValue::Amount {
            amount: transaction_amount(extended, sign),
        },
        GemTransactionValue::SwapReceived => swap_leg(extended, SwapLeg::To).map_or(GemTransactionRowValue::None, |amount| GemTransactionRowValue::Amount { amount }),
        GemTransactionValue::SwapSpent => swap_leg(extended, SwapLeg::From).map_or(GemTransactionRowValue::None, |amount| GemTransactionRowValue::Amount { amount }),
        GemTransactionValue::PerpetualNotional => perpetual_collateral_asset()
            .and_then(|asset| BigNumberFormatter::value_as_f64(&transaction.value.to_string(), asset.decimals as u32).ok())
            .map_or(GemTransactionRowValue::None, |value| GemTransactionRowValue::Fiat { value }),
        GemTransactionValue::PerpetualPnl { value } => GemTransactionRowValue::Pnl { value },
    }
}

fn header(extended: &TransactionExtended) -> GemTransactionHeader {
    let transaction = &extended.transaction;
    let amount = |shows_fiat: bool| GemTransactionHeader::Amount {
        amount: transaction_amount(extended, value_sign(transaction)),
        shows_fiat,
    };
    match header_kind(transaction) {
        GemTransactionHeaderKind::Amount { shows_fiat } => amount(shows_fiat),
        GemTransactionHeaderKind::Swap => match (swap_leg(extended, SwapLeg::From), swap_leg(extended, SwapLeg::To)) {
            (Some(from), Some(to)) => GemTransactionHeader::Swap { from, to },
            _ => amount(true),
        },
        GemTransactionHeaderKind::Nft => match nft_metadata(transaction) {
            Some(metadata) => GemTransactionHeader::Nft {
                image_url: GemImage::NftAsset {
                    asset_id: metadata.asset_id.to_string(),
                }
                .url(),
                asset_id: metadata.asset_id,
                name: metadata.name,
            },
            None => amount(false),
        },
        GemTransactionHeaderKind::Symbol => GemTransactionHeader::Symbol { asset: extended.asset.clone() },
        GemTransactionHeaderKind::AssetImage => GemTransactionHeader::AssetImage { asset: extended.asset.clone() },
    }
}

fn header_action(transaction: &Transaction) -> Option<GemTransactionHeaderAction> {
    match transaction.transaction_type {
        TransactionType::Transfer
        | TransactionType::TokenApproval
        | TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeRedelegate
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze => Some(GemTransactionHeaderAction::Asset {
            asset_id: transaction.asset_id.clone(),
        }),
        TransactionType::TransferNFT => transaction.nft_asset_id().map(|asset_id| GemTransactionHeaderAction::Nft { asset_id }),
        TransactionType::Swap => transaction.swap_metadata().map(|metadata| GemTransactionHeaderAction::Swap {
            from_asset_id: metadata.from_asset,
            to_asset_id: metadata.to_asset,
        }),
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => {
            Some(GemTransactionHeaderAction::Perpetual {
                asset_id: transaction.asset_id.clone(),
            })
        }
        TransactionType::SmartContractCall | TransactionType::AssetActivation | TransactionType::EarnDeposit | TransactionType::EarnWithdraw => None,
    }
}

fn swap_rate(extended: &TransactionExtended) -> Option<GemSwapRate> {
    let from = swap_leg(extended, SwapLeg::From)?;
    let to = swap_leg(extended, SwapLeg::To)?;
    (from.value != 0u32.into() && to.value != 0u32.into()).then_some(GemSwapRate { from, to })
}

enum SwapLeg {
    From,
    To,
}

fn swap_leg(extended: &TransactionExtended, leg: SwapLeg) -> Option<GemTransactionAmount> {
    let metadata = extended.transaction.swap_metadata()?;
    let (asset_id, value, sign) = match leg {
        SwapLeg::From => (metadata.from_asset, metadata.from_value, GemAmountSign::Outgoing),
        SwapLeg::To => (metadata.to_asset, metadata.to_value, GemAmountSign::Incoming),
    };
    let asset = extended.assets.iter().chain([&extended.asset]).find(|asset| asset.id == asset_id)?.clone();
    let price = extended.prices.iter().find(|price| price.asset_id == asset_id).cloned();
    Some(GemTransactionAmount { asset, value, sign, price })
}

fn transaction_amount(extended: &TransactionExtended, sign: GemAmountSign) -> GemTransactionAmount {
    GemTransactionAmount {
        asset: extended.asset.clone(),
        value: extended.transaction.value.clone(),
        sign,
        price: asset_price(extended.price.as_ref(), &extended.asset.id),
    }
}

fn value_sign(transaction: &Transaction) -> GemAmountSign {
    match transaction_value(transaction) {
        GemTransactionValue::Amount { sign } => sign,
        _ => GemAmountSign::None,
    }
}

fn asset_price(price: Option<&Price>, asset_id: &AssetId) -> Option<AssetPrice> {
    price.map(|price| AssetPrice {
        asset_id: asset_id.clone(),
        price: price.price,
        price_change_percentage_24h: price.price_change_percentage_24h,
        updated_at: price.updated_at,
    })
}

fn address_name(extended: &TransactionExtended, address: &str) -> Option<primitives::AddressName> {
    [extended.from_address.as_ref(), extended.to_address.as_ref()]
        .into_iter()
        .flatten()
        .find(|name| name.address == address)
        .cloned()
}

fn perpetual_collateral_asset() -> Option<Asset> {
    wallet_default_assets(Chain::HyperCore).into_iter().find(|asset| asset.asset_type == AssetType::PERPETUAL)
}

fn nft_metadata(transaction: &Transaction) -> Option<TransactionNFTTransferMetadata> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionNFTTransferMetadata>(metadata).ok()
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

pub fn details(extended: &TransactionExtended) -> GemTransactionDetails {
    let transaction = &extended.transaction;
    let swap_metadata = (transaction.transaction_type == TransactionType::Swap).then(|| transaction.swap_metadata()).flatten();
    let provider = swap_metadata
        .as_ref()
        .and_then(|metadata| metadata.provider.as_deref())
        .and_then(|id| SwapperProvider::from_str(id).ok())
        .map(SwapperProviderType::new);
    let swap_progress = swap_progress(extended, swap_metadata.as_ref(), provider.as_ref());
    let perpetual = perpetual_metadata(transaction);
    GemTransactionDetails {
        estimated_confirmation_seconds: extended
            .confirmation_eta_seconds
            .filter(|seconds| *seconds > 0 && transaction.state == TransactionState::Pending && swap_progress.is_none()),
        swap_again: swap_metadata
            .as_ref()
            .filter(|_| transaction.state == TransactionState::Confirmed)
            .map(|metadata| GemSwapAgain {
                from_asset_id: metadata.from_asset.clone(),
                to_asset_id: metadata.to_asset.clone(),
            }),
        swap_progress,
        provider_name: provider
            .map(|provider| provider.name)
            .or_else(|| swap_metadata.as_ref().and_then(|metadata| metadata.provider.clone())),
        pnl: perpetual.as_ref().map(|metadata| metadata.pnl).filter(|pnl| *pnl != 0.0),
        price: perpetual.as_ref().map(|metadata| metadata.price).filter(|price| *price > 0.0),
    }
}

fn swap_progress(extended: &TransactionExtended, metadata: Option<&TransactionSwapMetadata>, provider: Option<&SwapperProviderType>) -> Option<GemSwapProgress> {
    let metadata = metadata?;
    let provider = provider.filter(|provider| provider.mode != SwapperProviderMode::OnChain)?;
    let from_asset = extended.assets.iter().chain([&extended.asset]).find(|asset| asset.id == metadata.from_asset)?;
    let (transfer, swap) = match extended.transaction.state {
        TransactionState::Pending => (GemSwapProgressStep::Pending, GemSwapProgressStep::Waiting),
        TransactionState::InTransit => (GemSwapProgressStep::Completed, GemSwapProgressStep::Pending),
        TransactionState::Failed => (GemSwapProgressStep::Completed, GemSwapProgressStep::Failed),
        TransactionState::Reverted => (GemSwapProgressStep::Reverted, GemSwapProgressStep::Waiting),
        TransactionState::Refunded => (GemSwapProgressStep::Completed, GemSwapProgressStep::Refunded),
        TransactionState::Confirmed => return None,
    };
    Some(GemSwapProgress {
        from_asset: from_asset.clone(),
        from_value: metadata.from_value.clone(),
        provider_name: provider.name.clone(),
        transfer,
        swap,
        eta_seconds: extended
            .confirmation_eta_seconds
            .filter(|seconds| *seconds > 0 && !extended.transaction.state.is_completed()),
    })
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
    fn test_pending_transactions_keeps_what_a_synced_wallet_still_has_to_watch() {
        use TransactionState::{Confirmed, Failed, InTransit, Pending, Reverted};

        let transactions: Vec<Transaction> = [Pending, Confirmed, InTransit, Failed, Reverted]
            .into_iter()
            .map(|state| typed(TransactionType::Transfer, state, TransactionDirection::Outgoing))
            .collect();

        let pending: Vec<TransactionState> = pending_transactions(&transactions).into_iter().map(|transaction| transaction.state).collect();

        assert_eq!(pending, vec![Pending, InTransit], "a swap in transit is not settled yet, so the tracker keeps polling it");
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

    fn swap(state: TransactionState, provider: Option<&str>, eta: Option<u32>) -> TransactionExtended {
        let mut transaction = Transaction::mock();
        transaction.transaction_type = TransactionType::Swap;
        transaction.state = state;
        transaction.metadata = Some(
            serde_json::to_value(primitives::TransactionSwapMetadata {
                from_asset: AssetId::from_chain(Chain::Ethereum),
                from_value: 5u32.into(),
                to_asset: AssetId::from_chain(Chain::Bitcoin),
                to_value: 1u32.into(),
                provider: provider.map(str::to_string),
            })
            .unwrap(),
        );
        let mut extended = TransactionExtended::mock_transaction(transaction);
        extended.confirmation_eta_seconds = eta;
        extended
    }

    fn extended_with(transaction: Transaction, assets: Vec<Asset>) -> TransactionExtended {
        let mut extended = TransactionExtended::mock_transaction(transaction);
        extended.assets = assets;
        extended
    }

    fn named(address: &str, name: &str) -> primitives::AddressName {
        primitives::AddressName::mock(address, name, primitives::AddressType::Address, primitives::VerificationStatus::Verified)
    }

    #[test]
    fn test_row_resolves_the_swap_legs_from_the_metadata_and_the_known_assets() {
        let extended = extended_with(swap(TransactionState::Confirmed, None, None).transaction, vec![Asset::mock_eth(), Asset::mock_btc()]);

        let swap_row = row(&extended);

        assert_eq!(swap_row.title, GemTransactionTitle::Swap);
        match (&swap_row.value, &swap_row.equivalent_value) {
            (GemTransactionRowValue::Amount { amount: received }, GemTransactionRowValue::Amount { amount: spent }) => {
                assert_eq!(
                    (received.asset.id.clone(), received.value.clone(), received.sign),
                    (AssetId::from_chain(Chain::Bitcoin), 1u32.into(), GemAmountSign::Incoming)
                );
                assert_eq!(
                    (spent.asset.id.clone(), spent.value.clone(), spent.sign),
                    (AssetId::from_chain(Chain::Ethereum), 5u32.into(), GemAmountSign::Outgoing)
                );
            }
            other => panic!("a swap row shows both legs, got {other:?}"),
        }
        assert_eq!(
            row(&extended_with(swap(TransactionState::Confirmed, None, None).transaction, vec![])).value,
            GemTransactionRowValue::None,
            "a leg whose asset is unknown is not shown as a number"
        );
    }

    #[test]
    fn test_row_names_the_counterparty_when_the_wallet_knows_the_address() {
        let mut incoming = extended_with(typed(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Incoming), vec![]);
        incoming.from_address = Some(named("from", "Alice"));
        assert_eq!(
            row(&incoming).subtitle,
            GemTransactionRowSubtitle::FromAddress {
                address: "from".to_string(),
                name: Some("Alice".to_string())
            }
        );

        let outgoing = extended_with(typed(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Outgoing), vec![]);
        assert_eq!(
            row(&outgoing).subtitle,
            GemTransactionRowSubtitle::ToAddress {
                address: "to".to_string(),
                name: None
            }
        );
        match row(&outgoing).value {
            GemTransactionRowValue::Amount { amount } => assert_eq!(
                (amount.asset.id, amount.value, amount.sign),
                (AssetId::from_chain(Chain::Ethereum), 1u32.into(), GemAmountSign::Outgoing)
            ),
            other => panic!("a transfer row shows its amount, got {other:?}"),
        }
    }

    #[test]
    fn test_row_carries_the_nft_image_and_the_perpetual_notional_in_collateral_units() {
        let mut nft = typed(TransactionType::TransferNFT, TransactionState::Confirmed, TransactionDirection::Outgoing);
        let asset_id = primitives::NFTAssetId::new(Chain::Ethereum, "0xcontract", "7");
        nft.metadata = Some(serde_json::to_value(TransactionNFTTransferMetadata::new(asset_id.clone(), Some("Punk".to_string()))).unwrap());
        let nft_row = row(&extended_with(nft, vec![]));
        assert!(nft_row.nft_image_url.as_deref().is_some_and(|url| url.contains(&asset_id.to_string())));
        assert_eq!(nft_row.value, GemTransactionRowValue::None);

        let mut open = typed(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        open.value = 1_500_000u32.into();
        assert_eq!(
            row(&extended_with(open, vec![])).value,
            GemTransactionRowValue::Fiat { value: 1.5 },
            "the notional is the value in collateral units"
        );
    }

    #[test]
    fn test_detail_rows_build_the_header_the_participant_the_rate_and_the_fee() {
        let link = |address: &str| BlockExplorerLink {
            name: "Explorer".to_string(),
            link: format!("https://explorer/{address}"),
        };
        let explorer = BlockExplorerLink {
            name: "Explorer".to_string(),
            link: "https://explorer/tx".to_string(),
        };

        let swap = extended_with(swap(TransactionState::Confirmed, None, None).transaction, vec![Asset::mock_eth(), Asset::mock_btc()]);
        let rows = detail_rows(&swap, participant(&swap, link), explorer.clone());
        match rows.header {
            GemTransactionHeader::Swap { from, to } => assert_eq!((from.asset.id, to.asset.id), (AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Bitcoin))),
            other => panic!("a swap with both assets shows the swap header, got {other:?}"),
        }
        assert!(rows.rate.is_some());
        assert!(rows.participant.is_none(), "a swap names its provider, not a participant");
        assert_eq!(
            rows.header_action,
            Some(GemTransactionHeaderAction::Swap {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: AssetId::from_chain(Chain::Bitcoin)
            })
        );

        let mut transfer = extended_with(typed(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Outgoing), vec![]);
        transfer.transaction.memo = Some(String::new());
        let unnamed = detail_rows(&transfer, participant(&transfer, link), explorer.clone());
        let recipient = unnamed.participant.clone().unwrap();
        assert_eq!(
            (recipient.role, recipient.address.as_str(), recipient.can_add_contact),
            (GemTransactionParticipantRole::Recipient, "to", true)
        );
        assert_eq!(recipient.link.link, "https://explorer/to");
        assert_eq!(unnamed.memo, None, "an empty memo is not a row");
        assert_eq!(
            (unnamed.fee.asset.id.clone(), unnamed.fee.value.clone(), unnamed.fee.sign),
            (transfer.fee_asset.id.clone(), 1u32.into(), GemAmountSign::None)
        );
        assert!(matches!(unnamed.header, GemTransactionHeader::Amount { shows_fiat: true, .. }));
        assert_eq!(
            unnamed.header_action,
            Some(GemTransactionHeaderAction::Asset {
                asset_id: AssetId::from_chain(Chain::Ethereum)
            })
        );

        transfer.to_address = Some(named("to", "Bob"));
        let named_rows = detail_rows(&transfer, participant(&transfer, link), explorer.clone());
        let recipient = named_rows.participant.unwrap();
        assert_eq!((recipient.name.map(|name| name.name), recipient.can_add_contact), (Some("Bob".to_string()), false));

        let approval = extended_with(typed(TransactionType::TokenApproval, TransactionState::Confirmed, TransactionDirection::Outgoing), vec![]);
        let approval_rows = detail_rows(&approval, participant(&approval, link), explorer);
        assert!(matches!(approval_rows.header, GemTransactionHeader::AssetImage { .. }));
        let contract = approval_rows.participant.unwrap();
        assert_eq!((contract.role, contract.can_add_contact), (GemTransactionParticipantRole::Contract, false));
    }

    #[test]
    fn test_swap_rate_needs_both_legs_to_have_a_value() {
        let mut zero = swap(TransactionState::Confirmed, None, None).transaction;
        zero.metadata = Some(
            serde_json::to_value(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(Chain::Ethereum),
                from_value: 0u32.into(),
                to_asset: AssetId::from_chain(Chain::Bitcoin),
                to_value: 1u32.into(),
                provider: None,
            })
            .unwrap(),
        );
        assert!(swap_rate(&extended_with(zero, vec![Asset::mock_eth(), Asset::mock_btc()])).is_none());
    }

    #[test]
    fn test_details_show_swap_progress_only_for_an_unfinished_cross_chain_swap() {
        let pending = details(&swap(TransactionState::Pending, Some("thorchain"), Some(90)));
        let progress = pending.swap_progress.unwrap();
        assert_eq!(
            (progress.transfer, progress.swap, progress.eta_seconds),
            (GemSwapProgressStep::Pending, GemSwapProgressStep::Waiting, Some(90))
        );
        assert_eq!(progress.from_value, 5u32.into());
        assert_eq!(pending.provider_name.as_deref(), Some(progress.provider_name.as_str()));
        assert_eq!(pending.estimated_confirmation_seconds, None, "the progress steps carry the eta");
        assert!(pending.swap_again.is_none());

        let in_transit = details(&swap(TransactionState::InTransit, Some("thorchain"), None)).swap_progress.unwrap();
        assert_eq!((in_transit.transfer, in_transit.swap), (GemSwapProgressStep::Completed, GemSwapProgressStep::Pending));
        let failed = details(&swap(TransactionState::Failed, Some("thorchain"), Some(90))).swap_progress.unwrap();
        assert_eq!(
            (failed.transfer, failed.swap, failed.eta_seconds),
            (GemSwapProgressStep::Completed, GemSwapProgressStep::Failed, None)
        );
        let reverted = details(&swap(TransactionState::Reverted, Some("thorchain"), None)).swap_progress.unwrap();
        assert_eq!((reverted.transfer, reverted.swap), (GemSwapProgressStep::Reverted, GemSwapProgressStep::Waiting));
        let refunded = details(&swap(TransactionState::Refunded, Some("thorchain"), None)).swap_progress.unwrap();
        assert_eq!((refunded.transfer, refunded.swap), (GemSwapProgressStep::Completed, GemSwapProgressStep::Refunded));

        let confirmed = details(&swap(TransactionState::Confirmed, Some("thorchain"), None));
        assert!(confirmed.swap_progress.is_none());
        assert_eq!(
            confirmed.swap_again,
            Some(GemSwapAgain {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: AssetId::from_chain(Chain::Bitcoin),
            })
        );

        let on_chain = details(&swap(TransactionState::Pending, Some("uniswap_v3"), Some(90)));
        assert!(on_chain.swap_progress.is_none());
        assert_eq!(on_chain.estimated_confirmation_seconds, Some(90));
        assert_eq!(
            details(&swap(TransactionState::Pending, Some("unknown"), None)).provider_name.as_deref(),
            Some("unknown"),
            "an unknown provider still shows its id"
        );
        assert!(details(&swap(TransactionState::Pending, None, None)).swap_progress.is_none());
    }

    #[test]
    fn test_details_show_a_pending_eta_and_nonzero_perpetual_figures() {
        let mut transfer = TransactionExtended::mock();
        transfer.transaction.state = TransactionState::Pending;
        transfer.confirmation_eta_seconds = Some(30);
        assert_eq!(details(&transfer).estimated_confirmation_seconds, Some(30));
        transfer.transaction.state = TransactionState::Confirmed;
        assert_eq!(details(&transfer).estimated_confirmation_seconds, None);
        transfer.transaction.state = TransactionState::Pending;
        transfer.confirmation_eta_seconds = Some(0);
        assert_eq!(details(&transfer).estimated_confirmation_seconds, None);

        let mut close = TransactionExtended::mock();
        close.transaction.transaction_type = TransactionType::PerpetualClosePosition;
        let metadata = |pnl, price| TransactionPerpetualMetadata {
            pnl,
            price,
            direction: PerpetualDirection::Long,
            is_liquidation: None,
            provider: None,
        };
        close.transaction.metadata = Some(serde_json::to_value(metadata(0.0, 0.0)).unwrap());
        assert_eq!((details(&close).pnl, details(&close).price), (None, None));
        close.transaction.metadata = Some(serde_json::to_value(metadata(-4.5, 12.0)).unwrap());
        assert_eq!((details(&close).pnl, details(&close).price), (Some(-4.5), Some(12.0)));
    }
}
