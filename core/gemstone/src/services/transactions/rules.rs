use std::str::FromStr;

use strum::IntoEnumIterator;

use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetId, AssetPrice, AssetType, BlockExplorerLink, Chain, ChainAsset, Currency, PerpetualDirection, Price, Transaction, TransactionDirection, TransactionExtended, TransactionNFTTransferMetadata, TransactionPerpetualMetadata,
    TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionType, TransactionWalletConnectMetadata, TransferDataOutputAction, WalletType,
};

use super::model::{
    GemActivityFilters, GemAmountSign, GemSwapAgain, GemSwapProgress, GemSwapProgressStep, GemTransactionAmount, GemTransactionDetailRow, GemTransactionDetailRows, GemTransactionDetailSection, GemTransactionDetails, GemTransactionFeeRow,
    GemTransactionFilter, GemTransactionHeader, GemTransactionHeaderAction, GemTransactionHeaderKind, GemTransactionParticipant, GemTransactionParticipantRole, GemTransactionRow, GemTransactionRowSubtitle, GemTransactionRowValue,
    GemTransactionStateTone, GemTransactionStatus, GemTransactionSubtitle, GemTransactionTitle, GemTransactionValue,
};
use crate::address_formatter::{GemAddressFormatStyle, format_address};
use crate::config::image::GemImage;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::asset::wallet_default_assets;
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::collections::unique;
use crate::services::localization::GemLocalizedText;
use crate::services::swap::model::GemSwapRate;
use crate::services::swap::rules as swap_rules;
use swapper::{ProviderType as SwapperProviderType, SwapperProvider, SwapperProviderMode};

pub fn transaction_filters() -> Vec<GemTransactionFilter> {
    vec![
        GemTransactionFilter::Transfers,
        GemTransactionFilter::Swaps,
        GemTransactionFilter::Stake,
        GemTransactionFilter::SmartContract,
        GemTransactionFilter::Perpetuals,
        GemTransactionFilter::Others,
    ]
}

fn transaction_filter(transaction_type: &TransactionType) -> GemTransactionFilter {
    match transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT => GemTransactionFilter::Transfers,
        TransactionType::Swap | TransactionType::TokenApproval => GemTransactionFilter::Swaps,
        TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeRedelegate
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze
        | TransactionType::EarnDeposit
        | TransactionType::EarnWithdraw => GemTransactionFilter::Stake,
        TransactionType::SmartContractCall => GemTransactionFilter::SmartContract,
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => GemTransactionFilter::Perpetuals,
        TransactionType::AssetActivation => GemTransactionFilter::Others,
    }
}

pub fn filter_transaction_types(filter: GemTransactionFilter) -> Vec<TransactionType> {
    TransactionType::all().into_iter().filter(|transaction_type| transaction_filter(transaction_type) == filter).collect()
}

pub fn pending_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions.iter().filter(|transaction| !transaction.state.is_completed()).cloned().collect()
}

pub fn transaction_asset_ids(transactions: &[Transaction]) -> Vec<AssetId> {
    unique(transactions.iter().flat_map(|transaction| transaction.associated_asset_ids()))
}

pub fn row(extended: &TransactionExtended) -> GemTransactionRow {
    let transaction = &extended.transaction;
    let value = row_value(extended, transaction_value(transaction));
    GemTransactionRow {
        id: transaction.id.clone(),
        asset: extended.asset.clone(),
        transaction_type: transaction.transaction_type.clone(),
        direction: transaction.direction.clone(),
        state: transaction.state,
        created_at: transaction.created_at,
        status: status(transaction.state),
        title: transaction_title(transaction),
        subtitle: row_subtitle(extended),
        value_tone: value_tone(&value),
        value,
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
        text: match &name {
            Some(name) => name.name.clone(),
            None => format_address(&address, Some(transaction.asset_id.chain), GemAddressFormatStyle::Short),
        },
        name,
        address,
        can_add_contact,
    })
}

pub fn detail_rows(extended: &TransactionExtended, wallet_type: WalletType, participant: Option<GemTransactionParticipant>, explorer: BlockExplorerLink, currency: Currency) -> GemTransactionDetailRows {
    let transaction = &extended.transaction;
    let details = details(extended, wallet_type);
    let fee = GemTransactionAmount {
        asset: extended.fee_asset.clone(),
        value: transaction.fee.clone(),
        sign: GemAmountSign::None,
        price: asset_price(extended.fee_price.as_ref(), &extended.fee_asset.id),
    };
    GemTransactionDetailRows {
        id: transaction.id.clone(),
        asset: extended.asset.clone(),
        transaction_type: transaction.transaction_type.clone(),
        direction: transaction.direction.clone(),
        state: transaction.state,
        created_at: transaction.created_at,
        status: status(transaction.state),
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
        pnl: details.pnl.map(GemFormattedNumber::signed_usd),
        price: details.price.map(GemFormattedNumber::usd),
        fee_row: fee_row(&fee, currency),
        fee,
        explorer,
    }
}

fn fee_row(fee: &GemTransactionAmount, currency: Currency) -> GemTransactionFeeRow {
    let value = BigNumberFormatter::f64_value(fee.value.to_string(), fee.asset.decimals as u32);
    GemTransactionFeeRow {
        title: GemListRowTitle::NetworkFee,
        amount: GemFormattedNumber::amount(value, Some(fee.asset.symbol.clone()), GemValueStyle::Auto),
        fiat: fee.price.as_ref().map(|price| GemFormattedNumber::currency(value * price.price, currency, GemCurrencyStyle::Currency)),
        info: GemInfoTopic::NetworkFee { asset: fee.asset.clone() },
    }
}

pub fn detail_sections(rows: &GemTransactionDetailRows) -> Vec<GemTransactionDetailSection> {
    use GemTransactionDetailRow::*;
    let list = |row: GemListRow| Row { row };
    let details = [
        Some(list(GemListRow::Date {
            title: GemListRowTitle::Date,
            date: rows.created_at,
        })),
        Some(list(status_row(rows))),
        rows.estimated_confirmation_seconds.is_some().then_some(EstimatedConfirmation),
        rows.participant.is_some().then_some(Participant),
        rows.memo.clone().filter(|memo| !memo.is_empty()).map(|memo| list(GemListRow::Memo { value: memo.clone(), copy: Some(memo) })),
        rows.resource.map(|resource| {
            list(GemListRow::Label {
                title: GemListRowTitle::Resource,
                text: GemLocalizedText::Resource { resource },
                tone: GemValueTone::Plain,
                info: None,
                progress: false,
            })
        }),
        rows.rate.is_some().then_some(Rate),
        Some(list(GemListRow::Network {
            title: GemListRowTitle::Network,
            chain: rows.asset.chain(),
            name: ChainAsset::from_chain(rows.asset.chain()).network_name,
        })),
        rows.provider_name.clone().map(|name| {
            list(GemListRow::Text {
                title: GemListRowTitle::Provider,
                value: name,
            })
        }),
        rows.pnl.clone().map(|pnl| {
            list(GemListRow::Amount {
                title: GemListRowTitle::Pnl,
                amount: pnl,
                info: None,
            })
        }),
        rows.price.clone().map(|price| {
            list(GemListRow::Amount {
                title: GemListRowTitle::Price,
                amount: price,
                info: None,
            })
        }),
    ];
    [
        vec![Header],
        rows.swap_progress.is_some().then_some(SwapProgress).into_iter().collect(),
        rows.swap_again.is_some().then_some(SwapAgain).into_iter().collect(),
        details.into_iter().flatten().collect(),
        vec![Fee],
        vec![list(GemListRow::Explorer {
            name: rows.explorer.name.clone(),
            url: rows.explorer.link.clone(),
        })],
    ]
    .into_iter()
    .filter(|rows| !rows.is_empty())
    .map(|rows| GemTransactionDetailSection { rows })
    .collect()
}

fn status_row(rows: &GemTransactionDetailRows) -> GemListRow {
    GemListRow::Label {
        title: GemListRowTitle::Status,
        text: GemLocalizedText::TransactionState { state: rows.state },
        tone: match rows.status.tone {
            GemTransactionStateTone::Pending | GemTransactionStateTone::Refunded => GemValueTone::Warning,
            GemTransactionStateTone::Success => GemValueTone::Positive,
            GemTransactionStateTone::Error => GemValueTone::Negative,
        },
        info: Some(GemInfoTopic::TransactionStatus { state: rows.state, tone: rows.status.tone }),
        progress: rows.status.shows_progress,
    }
}

fn row_subtitle(extended: &TransactionExtended) -> GemTransactionRowSubtitle {
    match transaction_subtitle(&extended.transaction) {
        GemTransactionSubtitle::None => GemTransactionRowSubtitle::None,
        GemTransactionSubtitle::ToAddress { address } => GemTransactionRowSubtitle::ToAddress {
            participant: participant_name(extended, &address),
        },
        GemTransactionSubtitle::FromAddress { address } => GemTransactionRowSubtitle::FromAddress {
            participant: participant_name(extended, &address),
        },
        GemTransactionSubtitle::ToResource { resource } => GemTransactionRowSubtitle::ToResource { resource },
        GemTransactionSubtitle::FromResource { resource } => GemTransactionRowSubtitle::FromResource { resource },
        GemTransactionSubtitle::Price { value } => GemTransactionRowSubtitle::Price { price: GemFormattedNumber::usd(value) },
    }
}

pub fn status(state: TransactionState) -> GemTransactionStatus {
    let tone = match state {
        TransactionState::Pending | TransactionState::InTransit => GemTransactionStateTone::Pending,
        TransactionState::Confirmed => GemTransactionStateTone::Success,
        TransactionState::Failed | TransactionState::Reverted => GemTransactionStateTone::Error,
        TransactionState::Refunded => GemTransactionStateTone::Refunded,
    };
    GemTransactionStatus {
        tone,
        shows_badge: state != TransactionState::Confirmed,
        shows_progress: tone == GemTransactionStateTone::Pending,
    }
}

fn participant_name(extended: &TransactionExtended, address: &str) -> String {
    address_name(extended, address)
        .map(|name| name.name)
        .unwrap_or_else(|| format_address(address, Some(extended.transaction.asset_id.chain), GemAddressFormatStyle::Short))
}

fn value_tone(value: &GemTransactionRowValue) -> GemValueTone {
    match value {
        GemTransactionRowValue::Number { number } => match number.tone {
            GemValueTone::Positive => GemValueTone::Positive,
            GemValueTone::Negative => GemValueTone::Negative,
            GemValueTone::Neutral | GemValueTone::Plain | GemValueTone::Warning => GemValueTone::Plain,
        },
        GemTransactionRowValue::None | GemTransactionRowValue::AssetSymbol { .. } => GemValueTone::Plain,
    }
}

fn amount_value(amount: GemTransactionAmount) -> GemTransactionRowValue {
    GemTransactionRowValue::Number {
        number: amount.sign.amount(amount.value.into(), amount.asset.decimals as u32, Some(amount.asset.symbol), GemValueStyle::Short),
    }
}

fn row_value(extended: &TransactionExtended, value: GemTransactionValue) -> GemTransactionRowValue {
    let transaction = &extended.transaction;
    match value {
        GemTransactionValue::None => GemTransactionRowValue::None,
        GemTransactionValue::AssetSymbol => GemTransactionRowValue::AssetSymbol { asset: extended.asset.clone() },
        GemTransactionValue::Amount { sign } => amount_value(transaction_amount(extended, sign)),
        GemTransactionValue::SwapReceived => swap_leg(extended, SwapLeg::To, GemAmountSign::Incoming).map_or(GemTransactionRowValue::None, amount_value),
        GemTransactionValue::SwapSpent => swap_leg(extended, SwapLeg::From, GemAmountSign::Outgoing).map_or(GemTransactionRowValue::None, amount_value),
        GemTransactionValue::PerpetualNotional => perpetual_collateral_asset()
            .map(|asset| BigNumberFormatter::f64_value(&transaction.value, asset.decimals as u32))
            .map_or(GemTransactionRowValue::None, |value| GemTransactionRowValue::Number { number: GemFormattedNumber::usd(value) }),
        GemTransactionValue::PerpetualPnl { value } => GemTransactionRowValue::Number {
            number: GemFormattedNumber::signed_usd(value),
        },
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
        GemTransactionHeaderKind::Payment => amount(true),
        GemTransactionHeaderKind::Swap => match (swap_leg(extended, SwapLeg::From, GemAmountSign::None), swap_leg(extended, SwapLeg::To, GemAmountSign::None)) {
            (Some(from), Some(to)) => GemTransactionHeader::Swap { from, to },
            _ => amount(true),
        },
        GemTransactionHeaderKind::Nft => match nft_metadata(transaction) {
            Some(metadata) => GemTransactionHeader::Nft {
                image_url: GemImage::NftAsset { asset_id: metadata.asset_id.to_string() }.url(),
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
        | TransactionType::StakeUnfreeze => Some(GemTransactionHeaderAction::Asset { asset_id: transaction.asset_id.clone() }),
        TransactionType::TransferNFT => transaction.nft_asset_id().map(|asset_id| GemTransactionHeaderAction::Nft { asset_id }),
        TransactionType::Swap => transaction.swap_metadata().map(|metadata| GemTransactionHeaderAction::Swap {
            from_asset_id: metadata.from_asset,
            to_asset_id: metadata.to_asset,
        }),
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => Some(GemTransactionHeaderAction::Perpetual { asset_id: transaction.asset_id.clone() }),
        TransactionType::SmartContractCall | TransactionType::AssetActivation | TransactionType::EarnDeposit | TransactionType::EarnWithdraw => None,
    }
}

fn swap_rate(extended: &TransactionExtended) -> Option<GemSwapRate> {
    let from = swap_leg(extended, SwapLeg::From, GemAmountSign::None)?;
    let to = swap_leg(extended, SwapLeg::To, GemAmountSign::None)?;
    swap_rules::swap_rate(&from.asset, &from.value, &to.asset, &to.value)
}

enum SwapLeg {
    From,
    To,
}

fn swap_leg(extended: &TransactionExtended, leg: SwapLeg, sign: GemAmountSign) -> Option<GemTransactionAmount> {
    let metadata = extended.transaction.swap_metadata()?;
    let (asset_id, value) = match leg {
        SwapLeg::From => (metadata.from_asset, metadata.from_value),
        SwapLeg::To => (metadata.to_asset, metadata.to_value),
    };
    let asset = extended.assets.iter().chain([&extended.asset]).find(|asset| asset.id == asset_id)?.clone();
    let price = extended.prices.iter().find(|price| price.asset_id == asset_id && price.has_price()).cloned();
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
    price
        .map(|price| AssetPrice {
            asset_id: asset_id.clone(),
            price: price.price,
            price_change_percentage_24h: price.price_change_percentage_24h,
            updated_at: price.updated_at,
        })
        .filter(AssetPrice::has_price)
}

fn address_name(extended: &TransactionExtended, address: &str) -> Option<primitives::AddressName> {
    [extended.from_address.as_ref(), extended.to_address.as_ref()].into_iter().flatten().find(|name| name.address == address).cloned()
}

fn perpetual_collateral_asset() -> Option<Asset> {
    wallet_default_assets(Chain::HyperCore).into_iter().find(|asset| asset.asset_type == AssetType::PERPETUAL)
}

fn nft_metadata(transaction: &Transaction) -> Option<TransactionNFTTransferMetadata> {
    let metadata = transaction.metadata.clone()?;
    serde_json::from_value::<TransactionNFTTransferMetadata>(metadata).ok()
}

fn transaction_title(transaction: &Transaction) -> GemTransactionTitle {
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
        TransactionType::PerpetualOpenPosition => GemTransactionTitle::PerpetualOpen { direction: perpetual_direction(transaction) },
        TransactionType::PerpetualClosePosition => GemTransactionTitle::PerpetualClose { direction: perpetual_direction(transaction) },
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

fn transaction_subtitle(transaction: &Transaction) -> GemTransactionSubtitle {
    match transaction.transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT | TransactionType::TokenApproval | TransactionType::SmartContractCall => match transaction.direction {
            TransactionDirection::Incoming => GemTransactionSubtitle::FromAddress { address: transaction.from.clone() },
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => GemTransactionSubtitle::ToAddress { address: transaction.to.clone() },
        },
        TransactionType::StakeDelegate | TransactionType::StakeRedelegate | TransactionType::EarnDeposit => GemTransactionSubtitle::ToAddress { address: transaction.to.clone() },
        TransactionType::StakeUndelegate | TransactionType::EarnWithdraw => GemTransactionSubtitle::FromAddress { address: transaction.to.clone() },
        TransactionType::StakeFreeze => resource(transaction).map_or(GemTransactionSubtitle::None, |resource| GemTransactionSubtitle::ToResource { resource }),
        TransactionType::StakeUnfreeze => resource(transaction).map_or(GemTransactionSubtitle::None, |resource| GemTransactionSubtitle::FromResource { resource }),
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => match perpetual_metadata(transaction).map(|metadata| metadata.price).filter(|price| *price > 0.0) {
            Some(value) => GemTransactionSubtitle::Price { value },
            None => GemTransactionSubtitle::None,
        },
        TransactionType::Swap | TransactionType::StakeRewards | TransactionType::StakeWithdraw | TransactionType::AssetActivation => GemTransactionSubtitle::None,
    }
}

fn transaction_participant(transaction: &Transaction) -> Option<(GemTransactionParticipantRole, String)> {
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

fn transaction_value(transaction: &Transaction) -> GemTransactionValue {
    match transaction.transaction_type {
        TransactionType::Swap => GemTransactionValue::SwapReceived,
        TransactionType::TokenApproval => GemTransactionValue::AssetSymbol,
        TransactionType::PerpetualOpenPosition => GemTransactionValue::PerpetualNotional,
        TransactionType::PerpetualClosePosition => match perpetual_metadata(transaction).map(|metadata| metadata.pnl).filter(|pnl| *pnl != 0.0) {
            Some(value) => GemTransactionValue::PerpetualPnl { value },
            None => GemTransactionValue::None,
        },
        TransactionType::StakeRewards | TransactionType::StakeWithdraw => GemTransactionValue::Amount { sign: GemAmountSign::Incoming },
        TransactionType::Transfer => GemTransactionValue::Amount { sign: amount_sign(&transaction.direction) },
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

fn transaction_equivalent_value(transaction: &Transaction) -> GemTransactionValue {
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
    serde_json::from_value::<TransactionResourceTypeMetadata>(metadata).ok().map(|metadata| metadata.resource_type)
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
        TransactionType::AssetActivation | TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition | TransactionType::PerpetualModifyPosition => GemTransactionHeaderKind::Symbol,
    }
}

pub fn details(extended: &TransactionExtended, wallet_type: WalletType) -> GemTransactionDetails {
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
        estimated_confirmation_seconds: extended.confirmation_eta_seconds.filter(|seconds| *seconds > 0 && transaction.state == TransactionState::Pending && swap_progress.is_none()),
        swap_again: swap_again(transaction, swap_metadata.as_ref(), wallet_type),
        swap_progress,
        provider_name: provider.map(|provider| provider.name).or_else(|| swap_metadata.as_ref().and_then(|metadata| metadata.provider.clone())),
        pnl: perpetual.as_ref().map(|metadata| metadata.pnl).filter(|pnl| *pnl != 0.0),
        price: perpetual.as_ref().map(|metadata| metadata.price).filter(|price| *price > 0.0),
    }
}

fn swap_again(transaction: &Transaction, metadata: Option<&TransactionSwapMetadata>, wallet_type: WalletType) -> Option<GemSwapAgain> {
    let metadata = metadata?;
    if wallet_type == WalletType::View || transaction.state != TransactionState::Confirmed {
        return None;
    }
    Some(GemSwapAgain {
        from_asset_id: metadata.from_asset.clone(),
        to_asset_id: metadata.to_asset.clone(),
    })
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
        transfer: transfer.state(),
        swap: swap.state(),
        eta_seconds: extended.confirmation_eta_seconds.filter(|seconds| *seconds > 0 && !extended.transaction.state.is_completed()),
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

pub fn activity_filters(chains: Vec<Chain>, filters: Vec<GemTransactionFilter>) -> GemActivityFilters {
    let transaction_types = match filters.is_empty() {
        true => TransactionType::iter().collect(),
        false => {
            let mut types: Vec<TransactionType> = Vec::new();
            for transaction_type in filters.into_iter().flat_map(filter_transaction_types) {
                if !types.contains(&transaction_type) {
                    types.push(transaction_type);
                }
            }
            types
        }
    };
    GemActivityFilters {
        asset_rank_greater_than: crate::models::asset::default_token_rank(),
        chains,
        transaction_types,
    }
}

#[cfg(test)]
mod tests {
    use crate::formatted_number::GemNumberNotation;

    #[test]
    fn test_the_value_tone_greens_an_incoming_amount_and_signs_a_pnl() {
        use super::super::model::{GemAmountSign, GemTransactionAmount, GemTransactionRowValue};
        use super::{amount_value, value_tone};
        use crate::formatted_number::{GemFormattedNumber, GemValueTone};

        let amount = |sign| {
            amount_value(GemTransactionAmount {
                asset: Asset::from_chain(Chain::Ethereum),
                value: Transaction::mock().value,
                sign,
                price: None,
            })
        };
        let usd = |number| GemTransactionRowValue::Number { number };
        let number_of = |value| match value {
            GemTransactionRowValue::Number { number } => number,
            _ => panic!("expected a number"),
        };

        let incoming = number_of(amount(GemAmountSign::Incoming));
        let outgoing = number_of(amount(GemAmountSign::Outgoing));
        let unsigned = number_of(amount(GemAmountSign::None));

        assert_eq!(incoming.notation, GemNumberNotation::Signed);
        assert!(incoming.value > 0.0, "an incoming amount reads positive without a second call to add the sign");
        assert_eq!(outgoing.notation, GemNumberNotation::Signed);
        assert!(outgoing.value < 0.0, "an outgoing amount carries its own minus");
        assert_eq!(unsigned.notation, GemNumberNotation::Plain);

        assert_eq!(value_tone(&amount(GemAmountSign::Incoming)), GemValueTone::Positive);
        assert_eq!(value_tone(&amount(GemAmountSign::Outgoing)), GemValueTone::Plain, "an outgoing amount is not coloured like a loss");
        assert_eq!(value_tone(&usd(GemFormattedNumber::signed_usd(12.5))), GemValueTone::Positive);
        assert_eq!(value_tone(&usd(GemFormattedNumber::signed_usd(-0.5))), GemValueTone::Negative);
        assert_eq!(value_tone(&usd(GemFormattedNumber::signed_usd(0.0))), GemValueTone::Plain);
        assert_eq!(value_tone(&usd(GemFormattedNumber::usd(10.0))), GemValueTone::Plain);
    }

    #[test]
    fn test_an_empty_activity_list_reads_as_no_results_only_once_a_filter_is_on() {
        use super::super::model::{GemTransactionsEmptyState, transactions_empty_state};

        assert_eq!(transactions_empty_state(vec![], vec![]), GemTransactionsEmptyState::NoActivity);
        assert_eq!(transactions_empty_state(vec![Chain::Ethereum], vec![]), GemTransactionsEmptyState::NoResults);
        assert_eq!(
            transactions_empty_state(vec![], vec![GemTransactionFilter::Swaps]),
            GemTransactionsEmptyState::NoResults,
            "a type filter hides activity just as a chain filter does"
        );
    }
    #[test]
    fn test_every_transaction_type_belongs_to_exactly_one_filter_in_list_order() {
        let filters = transaction_filters();
        assert_eq!(filters.len(), 6);
        let grouped: Vec<TransactionType> = filters.iter().flat_map(|filter| filter_transaction_types(*filter)).collect();
        assert_eq!(grouped.len(), TransactionType::all().len());
        for transaction_type in TransactionType::all() {
            assert!(filter_transaction_types(transaction_filter(&transaction_type)).contains(&transaction_type));
        }
        assert_eq!(transaction_filter(&TransactionType::TokenApproval), GemTransactionFilter::Swaps);
        assert_eq!(transaction_filter(&TransactionType::EarnWithdraw), GemTransactionFilter::Stake);
        assert_eq!(transaction_filter(&TransactionType::AssetActivation), GemTransactionFilter::Others);
        assert_eq!(
            filter_transaction_types(GemTransactionFilter::Perpetuals),
            vec![TransactionType::PerpetualOpenPosition, TransactionType::PerpetualClosePosition, TransactionType::PerpetualModifyPosition],
            "the perpetual screen reads this list for its activity"
        );
    }

    use super::super::model::GemSwapProgressMarker;
    use super::*;
    use chrono::Utc;
    use primitives::{Chain, Resource};

    fn signing_details(extended: &TransactionExtended) -> GemTransactionDetails {
        details(extended, WalletType::Multicoin)
    }

    #[test]
    fn test_pending_transactions_keeps_what_a_synced_wallet_still_has_to_watch() {
        use TransactionState::{Confirmed, Failed, InTransit, Pending, Reverted};

        let transactions: Vec<Transaction> = [Pending, Confirmed, InTransit, Failed, Reverted]
            .into_iter()
            .map(|state| Transaction::mock_with_state(TransactionType::Transfer, state, TransactionDirection::Outgoing))
            .collect();

        let pending: Vec<TransactionState> = pending_transactions(&transactions).into_iter().map(|transaction| transaction.state).collect();

        assert_eq!(pending, vec![Pending, InTransit], "a swap in transit is not settled yet, so the tracker keeps polling it");
    }

    #[test]
    fn test_transaction_title_reads_a_transfer_from_its_state_and_direction() {
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};
        use TransactionState::{Confirmed, Failed, InTransit, Pending};

        assert_eq!(transaction_title(&Transaction::mock_with_state(TransactionType::Transfer, Confirmed, Incoming)), GemTransactionTitle::Received);
        assert_eq!(transaction_title(&Transaction::mock_with_state(TransactionType::Transfer, Confirmed, Outgoing)), GemTransactionTitle::Sent);
        assert_eq!(transaction_title(&Transaction::mock_with_state(TransactionType::Transfer, Confirmed, SelfTransfer)), GemTransactionTitle::Sent);
        assert_eq!(transaction_title(&Transaction::mock_with_state(TransactionType::TransferNFT, Confirmed, Incoming)), GemTransactionTitle::Received);

        for state in [Pending, Failed, InTransit] {
            assert_eq!(transaction_title(&Transaction::mock_with_state(TransactionType::Transfer, state, Incoming)), GemTransactionTitle::Transfer);
        }
    }

    #[test]
    fn test_transaction_title_separates_earn_from_stake() {
        assert_eq!(
            transaction_title(&Transaction::mock_with_state(TransactionType::StakeDelegate, TransactionState::Confirmed, TransactionDirection::Outgoing)),
            GemTransactionTitle::Stake
        );
        assert_eq!(
            transaction_title(&Transaction::mock_with_state(TransactionType::EarnDeposit, TransactionState::Confirmed, TransactionDirection::Outgoing)),
            GemTransactionTitle::Earn
        );
        assert_eq!(
            transaction_title(&Transaction::mock_with_state(TransactionType::EarnWithdraw, TransactionState::Confirmed, TransactionDirection::Outgoing)),
            GemTransactionTitle::Withdraw
        );
        assert_eq!(
            transaction_title(&Transaction::mock_with_state(TransactionType::StakeWithdraw, TransactionState::Confirmed, TransactionDirection::Outgoing)),
            GemTransactionTitle::Withdraw
        );
    }

    #[test]
    fn test_transaction_title_carries_the_perpetual_direction_when_the_metadata_has_one() {
        let mut open = Transaction::mock_with_state(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_title(&open), GemTransactionTitle::PerpetualOpen { direction: None });

        open.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                direction: PerpetualDirection::Short,
                ..TransactionPerpetualMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!(transaction_title(&open), GemTransactionTitle::PerpetualOpen { direction: Some(PerpetualDirection::Short) });

        let mut close = open.clone();
        close.transaction_type = TransactionType::PerpetualClosePosition;
        assert_eq!(transaction_title(&close), GemTransactionTitle::PerpetualClose { direction: Some(PerpetualDirection::Short) });
    }

    #[test]
    fn test_transaction_subtitle_names_the_counterparty_the_row_shows() {
        use TransactionDirection::{Incoming, Outgoing};

        assert_eq!(
            transaction_subtitle(&Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, Incoming)),
            GemTransactionSubtitle::FromAddress { address: "from".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, Outgoing)),
            GemTransactionSubtitle::ToAddress { address: "to".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&Transaction::mock_with_state(TransactionType::StakeDelegate, TransactionState::Confirmed, Outgoing)),
            GemTransactionSubtitle::ToAddress { address: "to".to_string() }
        );
        assert_eq!(
            transaction_subtitle(&Transaction::mock_with_state(TransactionType::StakeUndelegate, TransactionState::Confirmed, Outgoing)),
            GemTransactionSubtitle::FromAddress { address: "to".to_string() }
        );
        assert_eq!(transaction_subtitle(&Transaction::mock_with_state(TransactionType::Swap, TransactionState::Confirmed, Outgoing)), GemTransactionSubtitle::None);
        assert_eq!(
            transaction_subtitle(&Transaction::mock_with_state(TransactionType::StakeRewards, TransactionState::Confirmed, Incoming)),
            GemTransactionSubtitle::None
        );
    }

    #[test]
    fn test_transaction_subtitle_reads_the_resource_and_the_price_from_the_metadata() {
        let mut freeze = Transaction::mock_with_state(TransactionType::StakeFreeze, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_subtitle(&freeze), GemTransactionSubtitle::None);

        freeze.metadata = Some(serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Energy)).unwrap());
        assert_eq!(transaction_subtitle(&freeze), GemTransactionSubtitle::ToResource { resource: Resource::Energy });

        let mut unfreeze = freeze.clone();
        unfreeze.transaction_type = TransactionType::StakeUnfreeze;
        assert_eq!(transaction_subtitle(&unfreeze), GemTransactionSubtitle::FromResource { resource: Resource::Energy });

        let mut open = Transaction::mock_with_state(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_subtitle(&open), GemTransactionSubtitle::None);

        open.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                price: 12.5,
                ..TransactionPerpetualMetadata::mock()
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
                from_value: 1u32.into(),
                to_value: 1u32.into(),
                provider: None,
                ..primitives::TransactionSwapMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!(header_kind(&swap), GemTransactionHeaderKind::Swap);
        let mut approval = Transaction::mock();
        approval.transaction_type = TransactionType::TokenApproval;
        assert_eq!(header_kind(&approval), GemTransactionHeaderKind::AssetImage);
    }

    #[test]
    fn test_the_participant_reads_as_its_name_or_a_short_address() {
        let mut extended = TransactionExtended::mock();
        extended.transaction.to = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string();
        let link = |address: &str| BlockExplorerLink::mock_with_address(address);

        let unnamed = participant(&extended, link).expect("a transfer has a participant");
        assert_ne!(unnamed.text, unnamed.address, "an unnamed participant reads short on both apps");
        assert_eq!(unnamed.text, format_address(&unnamed.address, Some(extended.transaction.asset_id.chain), GemAddressFormatStyle::Short));

        let mut named = extended.clone();
        named.to_address = Some(primitives::AddressName {
            chain: extended.transaction.asset_id.chain,
            address: unnamed.address.clone(),
            name: "Binance".to_string(),
            address_type: primitives::AddressType::Address,
            status: primitives::VerificationStatus::Verified,
            image_url: None,
        });
        assert_eq!(participant(&named, link).unwrap().text, "Binance");
    }

    #[test]
    fn test_transaction_participant_names_the_role_of_the_address_the_screen_shows() {
        use GemTransactionParticipantRole::{Contract, Provider, Recipient, Sender, Validator};
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};

        let participant = |transaction_type, direction| transaction_participant(&Transaction::mock_with_state(transaction_type, TransactionState::Confirmed, direction));

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

        let mut send = Transaction::mock_with_state(TransactionType::SmartContractCall, TransactionState::Confirmed, Outgoing);
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

        let mut blank = Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, Outgoing);
        blank.to = String::new();
        assert_eq!(transaction_participant(&blank), None);
    }

    #[test]
    fn test_transaction_value_signs_what_the_row_shows() {
        use TransactionDirection::{Incoming, Outgoing, SelfTransfer};

        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, Incoming)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Outgoing }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, SelfTransfer)),
            GemTransactionValue::Amount { sign: GemAmountSign::None }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::StakeRewards, TransactionState::Confirmed, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::StakeWithdraw, TransactionState::Confirmed, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::Incoming }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::StakeDelegate, TransactionState::Confirmed, Outgoing)),
            GemTransactionValue::Amount { sign: GemAmountSign::None }
        );
        assert_eq!(
            transaction_value(&Transaction::mock_with_state(TransactionType::TokenApproval, TransactionState::Confirmed, Outgoing)),
            GemTransactionValue::AssetSymbol
        );
        assert_eq!(transaction_value(&Transaction::mock_with_state(TransactionType::TransferNFT, TransactionState::Confirmed, Incoming)), GemTransactionValue::None);
    }

    #[test]
    fn test_transaction_value_gives_a_swap_both_legs_and_a_perpetual_close_only_a_real_pnl() {
        let mut swap = Transaction::mock_with_state(TransactionType::Swap, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_value(&swap), GemTransactionValue::SwapReceived);
        assert_eq!(transaction_equivalent_value(&swap), GemTransactionValue::SwapSpent);

        swap.transaction_type = TransactionType::Transfer;
        assert_eq!(transaction_equivalent_value(&swap), GemTransactionValue::None);

        let mut close = Transaction::mock_with_state(TransactionType::PerpetualClosePosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        assert_eq!(transaction_value(&close), GemTransactionValue::None);

        close.metadata = Some(serde_json::to_value(TransactionPerpetualMetadata::mock()).unwrap());
        assert_eq!(transaction_value(&close), GemTransactionValue::None);

        close.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                pnl: -4.5,
                ..TransactionPerpetualMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!(transaction_value(&close), GemTransactionValue::PerpetualPnl { value: -4.5 });
    }

    #[test]
    fn test_transaction_asset_ids_includes_fee_assets_once() {
        let solana = AssetId::from_chain(Chain::Solana);
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let usdc = AssetId::from_token(Chain::Solana, "usdc");

        let mut asset_ids = transaction_asset_ids(&[
            Transaction {
                asset_id: usdc.clone(),
                fee_asset_id: solana.clone(),
                ..Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::SelfTransfer)
            },
            Transaction {
                asset_id: ethereum.clone(),
                fee_asset_id: ethereum.clone(),
                ..Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::SelfTransfer)
            },
        ]);
        asset_ids.sort_by_key(|asset_id| asset_id.to_string());
        let mut expected = vec![usdc, solana, ethereum];
        expected.sort_by_key(|asset_id| asset_id.to_string());

        assert_eq!(asset_ids, expected);
    }

    #[test]
    fn test_row_resolves_the_swap_legs_from_the_metadata_and_the_known_assets() {
        let extended = TransactionExtended {
            assets: vec![Asset::mock_eth(), Asset::mock_btc()],
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, None))
        };

        let swap_row = row(&extended);

        assert_eq!(swap_row.title, GemTransactionTitle::Swap);
        assert_eq!(
            (swap_row.value, swap_row.equivalent_value),
            (
                GemTransactionRowValue::Number {
                    number: GemFormattedNumber {
                        notation: GemNumberNotation::Signed,
                        tone: GemValueTone::Positive,
                        ..GemFormattedNumber::amount(0.00000001, Some("BTC".to_string()), GemValueStyle::Short)
                    },
                },
                GemTransactionRowValue::Number {
                    number: GemFormattedNumber {
                        value: -0.000000000000000005,
                        notation: GemNumberNotation::Signed,
                        tone: GemValueTone::Plain,
                        ..GemFormattedNumber::amount(0.000000000000000005, Some("ETH".to_string()), GemValueStyle::Short)
                    },
                },
            ),
            "a swap row shows both legs"
        );
        assert_eq!(
            row(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, None))).value,
            GemTransactionRowValue::None,
            "a leg whose asset is unknown is not shown as a number"
        );
    }

    #[test]
    fn test_status_groups_the_states_the_screens_render_alike() {
        assert_eq!(status(TransactionState::InTransit).tone, GemTransactionStateTone::Pending);
        assert_eq!(status(TransactionState::Reverted).tone, GemTransactionStateTone::Error);
        assert_eq!(status(TransactionState::Refunded).tone, GemTransactionStateTone::Refunded);
        assert!(!status(TransactionState::Confirmed).shows_badge, "a confirmed transaction carries no badge");
        assert!(status(TransactionState::Pending).shows_progress);
        assert!(!status(TransactionState::Refunded).shows_progress, "a refund is settled, so nothing spins");
    }

    #[test]
    fn test_row_shows_the_counterparty_name_when_the_wallet_knows_the_address() {
        let mut incoming = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Incoming));
        incoming.from_address = Some(primitives::AddressName::mock("from", "Alice", primitives::AddressType::Address, primitives::VerificationStatus::Verified));
        assert_eq!(row(&incoming).subtitle, GemTransactionRowSubtitle::FromAddress { participant: "Alice".to_string() });

        let outgoing = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Outgoing));
        assert_eq!(row(&outgoing).subtitle, GemTransactionRowSubtitle::ToAddress { participant: "to".to_string() });
        assert_eq!(
            row(&outgoing).value,
            GemTransactionRowValue::Number {
                number: GemFormattedNumber {
                    value: -0.000000000000000001,
                    notation: GemNumberNotation::Signed,
                    tone: GemValueTone::Plain,
                    ..GemFormattedNumber::amount(0.000000000000000001, Some("ETH".to_string()), GemValueStyle::Short)
                },
            },
            "a transfer row shows its amount"
        );
    }

    #[test]
    fn test_row_carries_the_nft_image_and_the_perpetual_notional_in_collateral_units() {
        let mut nft = Transaction::mock_with_state(TransactionType::TransferNFT, TransactionState::Confirmed, TransactionDirection::Outgoing);
        let asset_id = primitives::NFTAssetId::new(Chain::Ethereum, "0xcontract", "7");
        nft.metadata = Some(serde_json::to_value(TransactionNFTTransferMetadata::new(asset_id.clone(), Some("Punk".to_string()))).unwrap());
        let nft_row = row(&TransactionExtended::mock_transaction(nft));
        assert!(nft_row.nft_image_url.as_deref().is_some_and(|url| url.contains(&asset_id.to_string())));
        assert_eq!(nft_row.value, GemTransactionRowValue::None);

        let mut open = Transaction::mock_with_state(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing);
        open.value = 1_500_000u32.into();
        assert_eq!(
            row(&TransactionExtended::mock_transaction(open)).value,
            GemTransactionRowValue::Number { number: GemFormattedNumber::usd(1.5) },
            "the notional is the value in collateral units"
        );
    }

    #[test]
    fn test_detail_rows_build_the_header_the_participant_the_rate_and_the_fee() {
        let explorer = BlockExplorerLink::mock_with_address("tx");

        let swap = TransactionExtended {
            assets: vec![Asset::mock_eth(), Asset::mock_btc()],
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, None))
        };
        let rows = detail_rows(&swap, WalletType::Multicoin, participant(&swap, BlockExplorerLink::mock_with_address), explorer.clone(), Currency::USD);
        match rows.header {
            GemTransactionHeader::Swap { from, to } => {
                assert_eq!((from.asset.id, to.asset.id), (AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Bitcoin)));
                assert_eq!((from.sign, to.sign), (GemAmountSign::None, GemAmountSign::None), "the header shows both legs unsigned");
            }
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

        let mut transfer = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Outgoing));
        transfer.transaction.memo = Some(String::new());
        let unnamed = detail_rows(&transfer, WalletType::Multicoin, participant(&transfer, BlockExplorerLink::mock_with_address), explorer.clone(), Currency::USD);
        let recipient = unnamed.participant.clone().unwrap();
        assert_eq!((recipient.role, recipient.address.as_str(), recipient.can_add_contact), (GemTransactionParticipantRole::Recipient, "to", true));
        assert_eq!(recipient.link.link, "https://explorer/to");
        assert_eq!(unnamed.memo, None, "an empty memo is not a row");
        assert_eq!((unnamed.fee.asset.id.clone(), unnamed.fee.value.clone(), unnamed.fee.sign), (transfer.fee_asset.id.clone(), 1u32.into(), GemAmountSign::None));
        assert_eq!(unnamed.fee_row.title, GemListRowTitle::NetworkFee);
        assert_eq!(unnamed.fee_row.amount.unit, crate::formatted_number::GemNumberUnit::Symbol { symbol: transfer.fee_asset.symbol.clone() });
        assert_eq!(unnamed.fee_row.info, GemInfoTopic::NetworkFee { asset: transfer.fee_asset.clone() });
        assert_eq!(unnamed.fee_row.fiat, None, "a fee with no price names no fiat value");
        let priced = TransactionExtended {
            fee_price: Some(Price::new(4.0, 0.0, Utc::now(), primitives::PriceProvider::Coingecko)),
            ..transfer.clone()
        };
        assert_eq!(
            detail_rows(&priced, WalletType::Multicoin, None, explorer.clone(), Currency::USD).fee_row.fiat,
            Some(GemFormattedNumber::currency(unnamed.fee_row.amount.value * 4.0, Currency::USD, GemCurrencyStyle::Currency))
        );
        assert!(matches!(unnamed.header, GemTransactionHeader::Amount { shows_fiat: true, .. }));
        assert_eq!(
            unnamed.header_action,
            Some(GemTransactionHeaderAction::Asset {
                asset_id: AssetId::from_chain(Chain::Ethereum)
            })
        );

        transfer.to_address = Some(primitives::AddressName::mock("to", "Bob", primitives::AddressType::Address, primitives::VerificationStatus::Verified));
        let named_rows = detail_rows(&transfer, WalletType::Multicoin, participant(&transfer, BlockExplorerLink::mock_with_address), explorer.clone(), Currency::USD);
        let recipient = named_rows.participant.unwrap();
        assert_eq!((recipient.name.map(|name| name.name), recipient.can_add_contact), (Some("Bob".to_string()), false));

        let approval = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::TokenApproval, TransactionState::Confirmed, TransactionDirection::Outgoing));
        let approval_rows = detail_rows(&approval, WalletType::Multicoin, participant(&approval, BlockExplorerLink::mock_with_address), explorer, Currency::USD);
        assert!(matches!(approval_rows.header, GemTransactionHeader::AssetImage { .. }));
        let contract = approval_rows.participant.unwrap();
        assert_eq!((contract.role, contract.can_add_contact), (GemTransactionParticipantRole::Contract, false));
    }

    #[test]
    fn test_detail_sections_list_only_the_rows_the_transaction_has_in_one_order() {
        let explorer = BlockExplorerLink::mock_with_address("tx");
        let kind = |row: GemTransactionDetailRow| match row {
            GemTransactionDetailRow::Row { row: GemListRow::Explorer { .. } } => "Explorer".to_string(),
            GemTransactionDetailRow::Row { row: GemListRow::Memo { .. } } => "Memo".to_string(),
            GemTransactionDetailRow::Row {
                row: GemListRow::Date { title, .. } | GemListRow::Label { title, .. } | GemListRow::Text { title, .. } | GemListRow::Network { title, .. } | GemListRow::Amount { title, .. },
            } => format!("{title:?}"),
            other => format!("{other:?}"),
        };
        let rows_of = |sections: Vec<GemTransactionDetailSection>| sections.into_iter().map(|section| section.rows.into_iter().map(kind).collect::<Vec<_>>()).collect::<Vec<_>>();

        let mut transfer = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::Transfer, TransactionState::Confirmed, TransactionDirection::Outgoing));
        transfer.transaction.memo = Some("gm".to_string());
        assert_eq!(
            rows_of(detail_sections(&detail_rows(
                &transfer,
                WalletType::Multicoin,
                participant(&transfer, BlockExplorerLink::mock_with_address),
                explorer.clone(),
                Currency::USD
            ))),
            vec![vec!["Header"], vec!["Date", "Status", "Participant", "Memo", "Network"], vec!["Fee"], vec!["Explorer"]],
            "a transfer has no swap sections and shows its memo beside the recipient"
        );

        let pending = TransactionExtended {
            assets: vec![Asset::mock_eth(), Asset::mock_btc()],
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Pending, Some("near_intents")))
        };
        assert_eq!(
            rows_of(detail_sections(&detail_rows(
                &pending,
                WalletType::Multicoin,
                participant(&pending, BlockExplorerLink::mock_with_address),
                explorer.clone(),
                Currency::USD
            ))),
            vec![vec!["Header"], vec!["SwapProgress"], vec!["Date", "Status", "Rate", "Network", "Provider"], vec!["Fee"], vec!["Explorer"]],
            "a swap in flight shows its progress instead of a confirmation estimate, and its provider instead of a participant"
        );

        let confirmed = TransactionExtended {
            assets: vec![Asset::mock_eth(), Asset::mock_btc()],
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, Some("near_intents")))
        };
        assert_eq!(
            rows_of(detail_sections(&detail_rows(
                &confirmed,
                WalletType::Multicoin,
                participant(&confirmed, BlockExplorerLink::mock_with_address),
                explorer.clone(),
                Currency::USD
            )))[1],
            vec!["SwapAgain"],
            "a confirmed swap offers to swap again"
        );
        assert_eq!(
            rows_of(detail_sections(&detail_rows(
                &confirmed,
                WalletType::View,
                participant(&confirmed, BlockExplorerLink::mock_with_address),
                explorer.clone(),
                Currency::USD
            )))[1],
            vec!["Date", "Status", "Rate", "Network", "Provider"],
            "a watch-only wallet cannot sign, so the confirmed swap offers no swap again"
        );

        let mut open = TransactionExtended::mock_transaction(Transaction::mock_with_state(TransactionType::PerpetualOpenPosition, TransactionState::Confirmed, TransactionDirection::Outgoing));
        open.transaction.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                pnl: -3.5,
                price: 61.0,
                direction: PerpetualDirection::Short,
                ..TransactionPerpetualMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!(
            rows_of(detail_sections(&detail_rows(&open, WalletType::Multicoin, participant(&open, BlockExplorerLink::mock_with_address), explorer, Currency::USD)))[1],
            vec!["Date", "Status", "Network", "Pnl", "Price"],
            "a perpetual has no participant and shows its pnl and price after the network"
        );

        let status = detail_sections(&detail_rows(&transfer, WalletType::Multicoin, None, BlockExplorerLink::mock_with_address("tx"), Currency::USD))[1].rows[1].clone();
        assert_eq!(
            status,
            GemTransactionDetailRow::Row {
                row: GemListRow::Label {
                    title: GemListRowTitle::Status,
                    text: GemLocalizedText::TransactionState { state: TransactionState::Confirmed },
                    tone: GemValueTone::Positive,
                    info: Some(GemInfoTopic::TransactionStatus {
                        state: TransactionState::Confirmed,
                        tone: GemTransactionStateTone::Success,
                    }),
                    progress: false,
                },
            },
            "the status reads the state in its tone and explains it"
        );
    }

    #[test]
    fn test_swap_rate_needs_both_legs_to_have_a_value() {
        let mut zero = Transaction::mock_swap_with_provider(TransactionState::Confirmed, None);
        zero.metadata = Some(
            serde_json::to_value(TransactionSwapMetadata {
                from_value: 0u32.into(),
                to_value: 1u32.into(),
                provider: None,
                ..TransactionSwapMetadata::mock()
            })
            .unwrap(),
        );
        assert!(
            swap_rate(&TransactionExtended {
                assets: vec![Asset::mock_eth(), Asset::mock_btc()],
                ..TransactionExtended::mock_transaction(zero)
            })
            .is_none()
        );
    }

    #[test]
    fn test_details_show_swap_progress_only_for_an_unfinished_cross_chain_swap() {
        let pending = signing_details(&TransactionExtended {
            confirmation_eta_seconds: Some(90),
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Pending, Some("thorchain")))
        });
        let progress = pending.swap_progress.unwrap();
        assert_eq!((progress.transfer.step, progress.swap.step, progress.eta_seconds), (GemSwapProgressStep::Pending, GemSwapProgressStep::Waiting, Some(90)));
        assert_eq!(progress.from_value, 5u32.into());
        assert_eq!(pending.provider_name.as_deref(), Some(progress.provider_name.as_str()));
        assert_eq!(pending.estimated_confirmation_seconds, None, "the progress steps carry the eta");
        assert!(pending.swap_again.is_none());

        let in_transit = signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::InTransit, Some("thorchain"))))
            .swap_progress
            .unwrap();
        assert_eq!((in_transit.transfer.step, in_transit.swap.step), (GemSwapProgressStep::Completed, GemSwapProgressStep::Pending));
        let failed = signing_details(&TransactionExtended {
            confirmation_eta_seconds: Some(90),
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Failed, Some("thorchain")))
        })
        .swap_progress
        .unwrap();
        assert_eq!((failed.transfer.step, failed.swap.step, failed.eta_seconds), (GemSwapProgressStep::Completed, GemSwapProgressStep::Failed, None));
        let reverted = signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Reverted, Some("thorchain"))))
            .swap_progress
            .unwrap();
        assert_eq!((reverted.transfer.step, reverted.swap.step), (GemSwapProgressStep::Reverted, GemSwapProgressStep::Waiting));
        let refunded = signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Refunded, Some("thorchain"))))
            .swap_progress
            .unwrap();
        assert_eq!((refunded.transfer.step, refunded.swap.step), (GemSwapProgressStep::Completed, GemSwapProgressStep::Refunded));
        assert_eq!(
            (refunded.transfer.marker, refunded.swap.marker),
            (GemSwapProgressMarker::Check, GemSwapProgressMarker::Swap),
            "a refund reads as a swap back, not as a failure"
        );
        assert_eq!((progress.transfer.marker, progress.swap.marker), (GemSwapProgressMarker::Spinner, GemSwapProgressMarker::Dots));
        assert_eq!(failed.swap.marker, GemSwapProgressMarker::Cross);

        let confirmed = signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, Some("thorchain"))));
        assert!(confirmed.swap_progress.is_none());
        assert_eq!(
            confirmed.swap_again,
            Some(GemSwapAgain {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: AssetId::from_chain(Chain::Bitcoin),
            })
        );
        assert!(
            details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, Some("thorchain"))), WalletType::View)
                .swap_again
                .is_none(),
            "a watch-only wallet cannot sign a swap"
        );

        let on_chain = signing_details(&TransactionExtended {
            confirmation_eta_seconds: Some(90),
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Pending, Some("uniswap_v3")))
        });
        assert!(on_chain.swap_progress.is_none());
        assert_eq!(on_chain.estimated_confirmation_seconds, Some(90));
        assert_eq!(
            signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Pending, Some("unknown"))))
                .provider_name
                .as_deref(),
            Some("unknown"),
            "an unknown provider still shows its id"
        );
        assert!(
            signing_details(&TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Pending, None)))
                .swap_progress
                .is_none()
        );
    }

    #[test]
    fn test_details_show_a_pending_eta_and_nonzero_perpetual_figures() {
        let mut transfer = TransactionExtended::mock();
        transfer.transaction.state = TransactionState::Pending;
        transfer.confirmation_eta_seconds = Some(30);
        assert_eq!(signing_details(&transfer).estimated_confirmation_seconds, Some(30));
        transfer.transaction.state = TransactionState::Confirmed;
        assert_eq!(signing_details(&transfer).estimated_confirmation_seconds, None);
        transfer.transaction.state = TransactionState::Pending;
        transfer.confirmation_eta_seconds = Some(0);
        assert_eq!(signing_details(&transfer).estimated_confirmation_seconds, None);

        let mut close = TransactionExtended::mock();
        close.transaction.transaction_type = TransactionType::PerpetualClosePosition;
        close.transaction.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                price: 0.0,
                ..TransactionPerpetualMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!((signing_details(&close).pnl, signing_details(&close).price), (None, None));
        close.transaction.metadata = Some(
            serde_json::to_value(TransactionPerpetualMetadata {
                pnl: -4.5,
                price: 12.0,
                ..TransactionPerpetualMetadata::mock()
            })
            .unwrap(),
        );
        assert_eq!((signing_details(&close).pnl, signing_details(&close).price), (Some(-4.5), Some(12.0)));
    }

    #[test]
    fn test_a_zero_stored_price_is_not_a_price() {
        let mut transfer = TransactionExtended::mock();
        let amount_price = |extended: &TransactionExtended| transaction_amount(extended, GemAmountSign::None).price.map(|price| price.price);

        transfer.price = Some(Price::new(0.0, 0.0, Utc::now(), Default::default()));
        assert_eq!(amount_price(&transfer), None);

        transfer.price = Some(Price::new(12.0, 0.0, Utc::now(), Default::default()));
        assert_eq!(amount_price(&transfer), Some(12.0));
    }

    #[test]
    fn test_a_zero_swap_leg_price_is_not_a_price() {
        let mut swap = TransactionExtended {
            assets: vec![Asset::mock_eth(), Asset::mock_btc()],
            ..TransactionExtended::mock_transaction(Transaction::mock_swap_with_provider(TransactionState::Confirmed, None))
        };
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let leg_price = |extended: &TransactionExtended| swap_leg(extended, SwapLeg::From, GemAmountSign::Outgoing).and_then(|amount| amount.price).map(|price| price.price);

        swap.prices = vec![AssetPrice::new(ethereum.clone(), 0.0, 0.0, Utc::now())];
        assert_eq!(leg_price(&swap), None);

        swap.prices = vec![AssetPrice::new(ethereum, 12.0, 0.0, Utc::now())];
        assert_eq!(leg_price(&swap), Some(12.0));
    }

    #[test]
    fn test_the_activity_screen_asks_for_every_type_until_one_is_picked() {
        let unfiltered = activity_filters(vec![], vec![]);

        assert_eq!(unfiltered.chains, vec![]);
        assert_eq!(unfiltered.transaction_types.len(), TransactionType::iter().count(), "no filter means every type, not no type filter");
        assert_eq!(unfiltered.asset_rank_greater_than, crate::models::asset::default_token_rank());

        let swaps = activity_filters(vec![Chain::Ethereum], vec![GemTransactionFilter::Swaps]);

        assert_eq!(swaps.chains, vec![Chain::Ethereum]);
        assert_eq!(swaps.transaction_types, filter_transaction_types(GemTransactionFilter::Swaps));
    }

    #[test]
    fn test_two_filters_that_share_a_type_ask_for_it_once() {
        let filters = activity_filters(vec![], vec![GemTransactionFilter::Transfers, GemTransactionFilter::Transfers]);

        assert_eq!(filters.transaction_types, filter_transaction_types(GemTransactionFilter::Transfers));
    }
}
