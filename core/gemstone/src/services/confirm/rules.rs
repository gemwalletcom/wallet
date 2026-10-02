use super::model::{GemConfirmRowContent, GemConfirmSection, GemConfirmSimulation};
use super::swap::ConfirmSwapQuote;
use crate::address_formatter::{GemAddressFormatStyle, GemAddressService, format_address};
use crate::application;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::copy::address_copy;
use crate::models::list::{GemAddressRow, GemListRow, GemListRowTitle};
use crate::models::placeholder::text_or_placeholder;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::assets::model::{GemAssetItemRow, GemAssetItemTrailing, GemRowText};
use crate::services::assets::rules::{asset_text, balance_text, fee_amount};
use crate::services::contact::model::contact_avatar;
use crate::services::error_text::GemErrorText;
use crate::services::localization::{GemLocalizedText, GemPerpetualConfirmedAction};
use crate::services::name::rules::names_the_user_owns;
use crate::services::transfer::model::{GemConfirmDestination, GemConfirmRow, GemTransferData};
use crate::services::wallet::model::wallet_row;
use primitives::{AddressName, AssetType, BlockExplorerLink, PaymentVerification, PerpetualType};
use primitives::{
    Asset, AssetId, Chain, ChainType, EVMChain, FeePriority, FeeUnitType, GasPriceType, ScanTransaction, SimulationResult, SimulationWarningType, Transaction, TransactionType, TransferDataOutputAction, TransferDataOutputType, Wallet,
};
use primitives::{OptionStringExt, currency::Currency};

use super::error::{GemConfirmError, GemConfirmErrorDisplay, GemConfirmErrorInfo, GemConfirmErrorSheet, GemConfirmRequirement};
use super::model::{
    ConfirmState, GemAcquireAsset, GemAcquireAssetFlow, GemAcquireOption, GemApprovalValue, GemConfirmData, GemConfirmFee, GemConfirmFeeLoad, GemConfirmFeeSelection, GemConfirmInput, GemConfirmLoad, GemConfirmMetadata,
    GemConfirmSimulationState, GemFeeAsset, GemFeeRateKind, GemFeeRateRow, GemFeeRateRows, GemSubmitMessage, GemTransferAmountResult, SendInput,
};
use crate::config::chain::custom_fee_enabled;
use crate::config::fiat_config::get_fiat_config;
use crate::fee::fee_rate_text;
use crate::fee::rate_of;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::gateway::{GemBroadcastOptions, GemFeeRate};
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionLoadFee, GemTransactionLoadInput};
use crate::services::balance::GemAssetBalance;
use crate::services::balance::GemBalanceRequirement;
use crate::services::collections::unique;
use crate::services::swap::model::GemSwapPairSelection;
use crate::services::transactions::GemAmountSign;
use crate::services::transfer::GemPendingTransactionInput;
use crate::services::transfer::rules::TransferInput;
use crate::transfer_amount::GemTransferAmountError;
use num_bigint::{BigInt, Sign};
use primitives::AssetPrice;
use primitives::{TransactionInputType, TransferAmountInput};

impl SendInput {
    pub(super) fn signer_input(&self) -> Result<GemSignerInput, GemConfirmError> {
        let GemConfirmInput { from, transfer } = &self.confirm.input;
        let chain = transfer.input_type.get_asset().chain();
        let sender_address = signing_address(&self.wallet, chain, &from.address)?;
        Ok(GemSignerInput {
            input: GemTransactionLoadInput {
                input_type: transfer.input_type.clone(),
                sender_address,
                destination_address: transfer.recipient.address.clone(),
                value: self.value.to_biguint().ok_or_else(|| GemConfirmError::Load { msg: "negative transfer value".to_string() })?,
                gas_price: self.confirm.fee.gas_price_type.clone(),
                memo: transfer.recipient.memo.clone(),
                is_max_value: transfer.use_max_amount,
                metadata: self.confirm.metadata.clone(),
            },
            fee: GemTransactionLoadFee {
                fee: self.network_fee.clone(),
                ..self.confirm.fee.clone()
            },
        })
    }
}

fn signing_address(wallet: &Wallet, chain: Chain, from: &str) -> Result<String, GemConfirmError> {
    let signer = wallet.account(chain).ok_or(GemConfirmError::AccountMissing { chain })?;
    if signer.address != from {
        return Err(GemConfirmError::SenderMismatch {
            from: from.to_string(),
            signer: signer.address.clone(),
        });
    }
    Ok(signer.address.clone())
}

pub fn metadata_asset_ids(asset_id: &AssetId, fee_asset_id: &AssetId, extra_asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    unique([asset_id.clone(), fee_asset_id.clone()].into_iter().chain(extra_asset_ids))
}

pub(super) trait ConfirmInput {
    fn approval_value(&self) -> Option<(AssetId, GemApprovalValue)>;
    fn validate_approvals(&self, transactions: &[GemSignedTransaction]) -> Result<(), GemConfirmError>;
    fn simulation_payload(&self) -> Option<String>;
    fn validate_simulation(&self, simulation: &SimulationResult) -> Result<(), GemConfirmError>;
    fn broadcast_options(&self) -> GemBroadcastOptions;
}

impl ConfirmInput for TransactionInputType {
    fn approval_value(&self) -> Option<(AssetId, GemApprovalValue)> {
        match self {
            Self::TokenApprove { asset, approval_data } => Some((asset.id.clone(), approval_value_from(Some(&approval_data.value), approval_data.is_unlimited))),
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::Swap { .. }
            | Self::Stake { .. }
            | Self::Generic { .. }
            | Self::Payment { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => None,
        }
    }

    fn validate_approvals(&self, transactions: &[GemSignedTransaction]) -> Result<(), GemConfirmError> {
        for transaction in transactions {
            self.approval(transaction.transaction_type.clone()).map_err(|msg| GemConfirmError::ApprovalInvalid { msg })?;
        }
        Ok(())
    }

    fn simulation_payload(&self) -> Option<String> {
        let Self::Payment { extra, .. } = self else {
            return None;
        };
        if extra.output_type == TransferDataOutputType::Signature {
            return None;
        }
        extra.data.as_ref().filter(|data| !data.is_empty()).and_then(|data| String::from_utf8(data.clone()).ok())
    }

    fn validate_simulation(&self, simulation: &SimulationResult) -> Result<(), GemConfirmError> {
        let Self::Payment { .. } = self else {
            return Ok(());
        };
        if let Some(warning) = simulation.warnings.iter().find(|warning| warning.warning == SimulationWarningType::ValidationError) {
            return Err(GemConfirmError::Load {
                msg: warning.message.clone().unwrap_or_else(|| "Payment simulation failed".to_string()),
            });
        }
        Ok(())
    }

    fn broadcast_options(&self) -> GemBroadcastOptions {
        match (self.get_asset().chain(), self) {
            (Chain::Solana, Self::Swap { .. } | Self::Generic { .. }) => GemBroadcastOptions { skip_preflight: true },
            _ => GemBroadcastOptions { skip_preflight: false },
        }
    }
}

pub(super) fn is_broadcast(input_type: &TransactionInputType, transaction: &GemSignedTransaction) -> bool {
    match (input_type.output().output_action, input_type) {
        (TransferDataOutputAction::Send, _) => true,
        (TransferDataOutputAction::Sign, TransactionInputType::Payment { .. }) => transaction.transaction_type == TransactionType::TokenApproval,
        (TransferDataOutputAction::Sign, _) => false,
    }
}

pub(super) fn is_signature_only(input_type: &TransactionInputType) -> bool {
    match input_type {
        TransactionInputType::Payment { extra, .. } => extra.output_type == TransferDataOutputType::Signature && extra.approval.is_none(),
        _ => false,
    }
}

pub(super) fn asset_pick_needs_reload(current: &AssetId, picked: &AssetId, verification: Option<&PaymentVerification>) -> bool {
    current != picked || verification.is_some()
}

pub fn approval_value_from(value: Option<&GemBigUint>, is_unlimited: bool) -> GemApprovalValue {
    match value {
        Some(value) if !is_unlimited => GemApprovalValue::Exact { value: value.clone() },
        _ => GemApprovalValue::Unlimited,
    }
}

impl GemConfirmData {
    pub(super) fn fee_rate_rows(&self, fee_asset: &Asset, price: Option<f64>, currency: Currency) -> GemFeeRateRows {
        let rows = fee_rate_rows(self.input.transfer.input_type.get_asset().chain(), fee_asset, &self.fee_rates, &self.fee_selection, &self.fee);
        GemFeeRateRows {
            rows: rows
                .rows
                .into_iter()
                .map(|row| GemFeeRateRow {
                    amount: row.fee.as_ref().map(|fee| fee_amount(fee_asset, fee, price, currency.clone())),
                    ..row
                })
                .collect(),
            ..rows
        }
    }

    pub(super) fn fee_load(self, metadata: GemConfirmMetadata, fee_asset: Asset, currency: Currency) -> GemConfirmFeeLoad {
        let amount = self.preload_amount(&metadata, &fee_asset);
        let price = metadata.fee_price().map(|price| price.price);
        let formatted = |value: &GemBigInt| fee_amount(&fee_asset, value, price, currency.clone());
        GemConfirmFeeLoad {
            fee: GemConfirmFee {
                value: self.fee.fee.clone(),
                formatted: formatted(&self.fee.fee),
                additional_fees: self.fee.options.items(formatted),
                selected_priority: self.selected_priority,
                amount,
            },
            fee_asset,
            metadata,
            confirm_data: self,
            simulation: None,
        }
    }

    pub(super) fn preload_amount(&self, metadata: &GemConfirmMetadata, fee_asset: &Asset) -> GemTransferAmountResult {
        let transfer = &self.input.transfer;
        let available_value = transfer.available_value(&metadata.asset_balance);
        let input = TransferAmountInput {
            input_type: transfer.input_type.clone(),
            value: transfer.value.clone(),
            available_value,
            fee_asset: fee_asset.id.clone(),
            fee_asset_balance: transfer.fee_available_value(&metadata.fee_asset_balance),
            fee: self.fee.fee.clone(),
            is_max_amount: transfer.use_max_amount,
            destination_account_exists: self.metadata.get_is_destination_address_exist().ok(),
        };
        match input.calculate() {
            Ok(amount) => GemTransferAmountResult::Amount { amount },
            Err(error) => GemTransferAmountResult::Error {
                error: amount_error(error, transfer.input_type.get_asset(), fee_asset),
            },
        }
    }
}

fn amount_error(error: GemTransferAmountError, asset: &Asset, fee_asset: &Asset) -> GemConfirmError {
    let error_asset = |asset_id: &AssetId| if &asset.id == asset_id { asset.clone() } else { fee_asset.clone() };
    match error {
        GemTransferAmountError::InsufficientBalance { asset_id, required, available } => GemConfirmError::InsufficientBalance {
            asset: error_asset(&asset_id),
            requirement: GemBalanceRequirement::new(required, available),
        },
        GemTransferAmountError::InsufficientNetworkFee { asset_id, required, available } => GemConfirmError::InsufficientNetworkFee {
            asset: error_asset(&asset_id),
            requirement: Some(GemBalanceRequirement::new(required, available)),
        },
        GemTransferAmountError::MinimumAccountBalanceTooLow { asset_id, required, available } => GemConfirmError::MinimumAccountBalanceTooLow {
            asset: error_asset(&asset_id),
            requirement: GemBalanceRequirement::new(required, available),
        },
        GemTransferAmountError::DestinationAccountActivation { asset_id, required, .. } => GemConfirmError::DestinationAccountActivation { asset: error_asset(&asset_id), required },
        GemTransferAmountError::BelowSwapMinimum { asset_id, provider, minimum, value } => GemConfirmError::BelowSwapMinimum {
            asset: error_asset(&asset_id),
            provider,
            provider_name: provider.name().to_string(),
            requirement: GemBalanceRequirement::new(minimum, value),
        },
    }
}

pub fn preload_simulation(request: Option<&SimulationResult>, confirm_data: &GemConfirmData) -> Option<SimulationResult> {
    match request {
        Some(_) => None,
        None => confirm_data.simulation.clone(),
    }
}

impl GemConfirmLoad {
    pub(super) fn with_fee(self, fee: GemConfirmFeeLoad, requested: Option<GemConfirmSimulationState>, swap: Option<ConfirmSwapQuote>) -> ConfirmState {
        ConfirmState {
            load: Self {
                fee_asset: fee.fee_asset,
                metadata: fee.metadata,
                simulation: fee.simulation.or(requested).unwrap_or(self.simulation),
                fee: Some(fee.fee),
                ..self
            },
            confirm_data: Some(fee.confirm_data),
            swap,
        }
    }
}

pub fn balance_change_amount(value: &BigInt, asset: &Asset) -> GemFormattedNumber {
    let sign = match value.sign() {
        Sign::Plus => GemAmountSign::Incoming,
        Sign::Minus => GemAmountSign::Outgoing,
        Sign::NoSign => GemAmountSign::None,
    };
    let tone = match sign {
        GemAmountSign::Incoming => GemValueTone::Positive,
        GemAmountSign::Outgoing => GemValueTone::Negative,
        GemAmountSign::None => GemValueTone::Neutral,
    };
    GemFormattedNumber {
        tone,
        ..sign.amount(value.magnitude(), asset, GemValueStyle::Full)
    }
}

pub fn shows_fee_assets(fee_asset_ids: &[AssetId], selected: Option<&AssetId>) -> bool {
    fee_asset_ids.iter().any(|asset_id| Some(asset_id) != selected)
}

pub fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<AssetPrice>, currency: &Currency) -> Vec<GemFeeAsset> {
    balances
        .into_iter()
        .filter(|balance| balance.available > num_bigint::BigUint::from(0u32))
        .filter_map(|balance| {
            let asset = assets.iter().find(|asset| asset.id == balance.asset_id)?.clone();
            let price = prices.iter().find(|price| price.asset_id == balance.asset_id).cloned();
            Some(GemFeeAsset {
                row: fee_asset_row(&asset, &balance, price.as_ref().map(|price| price.price), currency),
                asset,
                balance,
                price,
            })
        })
        .collect()
}

pub fn fee_asset_row(asset: &Asset, balance: &GemAssetBalance, price: Option<f64>, currency: &Currency) -> GemAssetItemRow {
    GemAssetItemRow {
        icon: crate::services::assets::icon::asset_icon(&asset.id),
        title: asset.symbol.clone(),
        title_extra: (asset.name != asset.symbol).then(|| asset.name.clone()),
        subtitle: None,
        subtitle_extra: None,
        trailing: GemAssetItemTrailing::Value {
            value: balance_text(&balance.available, asset),
            extra: crate::services::assets::rules::fiat_amount_of(asset, &balance.available, price, currency.clone(), GemCurrencyStyle::Currency).map(GemRowText::neutral_number),
        },
        masks_balance: false,
    }
}

pub fn build_metadata(asset_id: AssetId, fee_asset_id: AssetId, balances: Vec<GemAssetBalance>, prices: Vec<AssetPrice>) -> Result<GemConfirmMetadata, GemConfirmError> {
    Ok(GemConfirmMetadata {
        asset_balance: asset_balance(&balances, &asset_id)?,
        fee_asset_balance: asset_balance(&balances, &fee_asset_id)?,
        prices,
    })
}

fn asset_balance(balances: &[GemAssetBalance], asset_id: &AssetId) -> Result<GemAssetBalance, GemConfirmError> {
    balances
        .iter()
        .find(|balance| balance.asset_id == *asset_id)
        .cloned()
        .ok_or_else(|| GemConfirmError::BalanceMissing { asset_id: asset_id.clone() })
}

fn acquire_options(flow: GemAcquireAssetFlow) -> Vec<GemAcquireOption> {
    match flow {
        GemAcquireAssetFlow::Options => vec![GemAcquireOption::Buy, GemAcquireOption::Swap, GemAcquireOption::Receive],
        GemAcquireAssetFlow::Fiat => vec![],
    }
}

pub fn error_info(display: &GemConfirmErrorDisplay, prices: &[AssetPrice], currency: Currency, input_asset_id: &AssetId, fee_asset_id: &AssetId) -> Option<GemConfirmErrorInfo> {
    let info = |sheet: GemConfirmErrorSheet, asset: Option<&Asset>, title: String, requirement: Option<&GemConfirmRequirement>, required: Option<&GemFormattedNumber>| {
        let required = required.or_else(|| requirement.map(|requirement| &requirement.required)).cloned();
        let price = asset.and_then(|asset| prices.iter().find(|price| price.asset_id == asset.id)).map(|price| price.price);
        let buy_amount = matches!(sheet, GemConfirmErrorSheet::NetworkFeeRequired | GemConfirmErrorSheet::NetworkFeeMissing).then(|| get_fiat_config().insufficient_network_fee_buy_amount);
        GemConfirmErrorInfo {
            sheet,
            asset: asset.cloned(),
            title,
            required_fiat: required.as_ref().zip(price).map(|(required, price)| GemFormattedNumber::currency(required.value * price, currency, GemCurrencyStyle::Currency)),
            required,
            available: requirement.map(|requirement| requirement.available.clone()),
            shortfall: requirement.map(|requirement| requirement.shortfall.clone()),
            acquire: asset.filter(|asset| asset.asset_type != AssetType::PERPETUAL).map(|asset| GemAcquireAsset {
                flow: acquire_asset_flow(asset.chain()),
                options: acquire_options(acquire_asset_flow(asset.chain())),
                buy_amount,
                swap_pair: acquire_swap_pair(input_asset_id, fee_asset_id, asset.id.clone()),
            }),
        }
    };
    match display {
        GemConfirmErrorDisplay::BalanceRequired { asset, requirement } => Some(info(GemConfirmErrorSheet::BalanceRequired, Some(asset), asset.symbol.clone(), Some(requirement), None)),
        GemConfirmErrorDisplay::NetworkFeeRequired { asset, title, requirement } => Some(info(GemConfirmErrorSheet::NetworkFeeRequired, Some(asset), title.clone(), Some(requirement), None)),
        GemConfirmErrorDisplay::NetworkFeeMissing { asset, title } => Some(info(GemConfirmErrorSheet::NetworkFeeMissing, Some(asset), title.clone(), None, None)),
        GemConfirmErrorDisplay::MinimumAccountBalance { asset, required } => Some(info(GemConfirmErrorSheet::MinimumAccountBalance, Some(asset), asset.symbol.clone(), None, Some(required))),
        GemConfirmErrorDisplay::SwapMinimum { asset, provider, provider_name, requirement } => Some(info(
            GemConfirmErrorSheet::SwapMinimum {
                provider: *provider,
                provider_name: provider_name.clone(),
            },
            Some(asset),
            asset.symbol.clone(),
            Some(requirement),
            None,
        )),
        GemConfirmErrorDisplay::DustThreshold { chain } => Some(info(GemConfirmErrorSheet::DustThreshold { chain: *chain }, None, chain.as_ref().to_string(), None, None)),
        GemConfirmErrorDisplay::Malicious => Some(info(GemConfirmErrorSheet::Malicious, None, String::new(), None, None)),
        GemConfirmErrorDisplay::MemoRequired { symbol } => Some(info(GemConfirmErrorSheet::MemoRequired { symbol: symbol.clone() }, None, symbol.clone(), None, None)),
        GemConfirmErrorDisplay::Offline
        | GemConfirmErrorDisplay::FeeRatesMissing
        | GemConfirmErrorDisplay::Cancelled
        | GemConfirmErrorDisplay::AccountMissing
        | GemConfirmErrorDisplay::Unknown
        | GemConfirmErrorDisplay::InsufficientFunds
        | GemConfirmErrorDisplay::DestinationAccountActivation { .. }
        | GemConfirmErrorDisplay::Payment { .. }
        | GemConfirmErrorDisplay::Message { .. } => None,
    }
}

fn acquire_asset_flow(chain: Chain) -> GemAcquireAssetFlow {
    match chain {
        Chain::Tron => GemAcquireAssetFlow::Options,
        _ => GemAcquireAssetFlow::Fiat,
    }
}

fn acquire_swap_pair(input_asset_id: &AssetId, fee_asset_id: &AssetId, asset_id: AssetId) -> GemSwapPairSelection {
    let pay_asset_id = if *input_asset_id == asset_id { fee_asset_id } else { input_asset_id };
    GemSwapPairSelection {
        pay_asset_id: Some(pay_asset_id.clone()).filter(|pay| *pay != asset_id),
        receive_asset_id: Some(asset_id),
    }
}

pub fn is_insufficient_network_fee(fee_asset_id: &AssetId, fee_available: &GemBigUint) -> bool {
    !matches!(fee_asset_id.chain, Chain::HyperCore | Chain::Tron) && fee_asset_id.is_native() && *fee_available == GemBigUint::ZERO
}

impl SendInput {
    pub(super) fn pending_transactions(&self, hashes: &[String], transactions: &[GemSignedTransaction]) -> Result<Vec<Transaction>, GemConfirmError> {
        let chain = self.confirm.input.transfer.input_type.get_asset().chain();
        let sender = self.wallet.account(chain).map(|account| account.address.clone()).ok_or_else(|| GemConfirmError::Record {
            msg: format!("wallet has no {chain} account"),
        })?;
        hashes
            .iter()
            .enumerate()
            .filter_map(|(index, hash)| {
                let transaction_type = transactions.get(index)?.transaction_type.clone();
                Some(
                    GemPendingTransactionInput {
                        sender: sender.clone(),
                        transfer: self.confirm.input.transfer.clone(),
                        value: self.value.clone(),
                        transaction_type,
                        hash: hash.clone(),
                        fee: self.confirm.fee.clone(),
                        network_fee: self.network_fee.clone(),
                        metadata: self.confirm.metadata.clone(),
                        simulation: self.simulation.clone(),
                        transaction_index: index as u32,
                        transaction_count: transactions.len() as u32,
                    }
                    .pending_transaction()
                    .map_err(|msg| GemConfirmError::Record { msg }),
                )
            })
            .map(|result| result.map(IntoIterator::into_iter))
            .collect::<Result<Vec<_>, _>>()
            .map(|transactions| transactions.into_iter().flatten().collect())
    }
}

pub(super) fn broadcast_delay_milliseconds(chain: Chain) -> u64 {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => 0,
        ChainType::Solana
        | ChainType::Bitcoin
        | ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Xrp
        | ChainType::Polkadot
        | ChainType::Cardano => 500,
    }
}

pub(super) fn validate_scan(scan: Option<&ScanTransaction>, memo: Option<&str>, symbol: &str) -> Result<(), GemConfirmError> {
    let Some(scan) = scan else {
        return Ok(());
    };
    if scan.is_malicious == Some(true) {
        return Err(GemConfirmError::ScanMalicious);
    }
    if scan.is_memo_required == Some(true) && memo.unwrap_or_default().trim().is_empty() {
        return Err(GemConfirmError::ScanMemoRequired { symbol: symbol.to_string() });
    }
    Ok(())
}

fn fee_rate_rows(chain: Chain, fee_asset: &Asset, rates: &[GemFeeRate], selection: &GemConfirmFeeSelection, loaded_fee: &GemTransactionLoadFee) -> GemFeeRateRows {
    let unit_value = |rate: &GemFeeRate| rate.gas_price_type.clone().total_fee();
    let base_rate = base_fee_rate(rates);
    let selected = selection.select_fee_rate(rates).ok().map(|rate| rate.gas_price_type);
    let selected_total = selected.as_ref().map(GasPriceType::total_fee);
    let custom_total = match selection {
        GemConfirmFeeSelection::Custom { .. } => selected_total.clone(),
        GemConfirmFeeSelection::Priority { .. } => None,
    };
    let base = selected_total.filter(|total| total > &BigInt::ZERO);
    let fixed_fee = loaded_fee.options.total();
    let rate_fee = &loaded_fee.fee - &fixed_fee;
    let unit_type = chain.fee_unit_type();
    let unit_decimals = match unit_type {
        FeeUnitType::Native => fee_asset.decimals as u32,
        FeeUnitType::SatVb | FeeUnitType::Gwei => unit_type.decimals(),
    };
    let rows = rates.iter().map(|rate| {
        let unit_value = unit_value(rate);
        let fee = base.as_ref().map(|base| &rate_fee * &unit_value / base + &fixed_fee);
        let display_value = match unit_type {
            FeeUnitType::Native => fee.clone().unwrap_or_else(|| unit_value.clone()),
            FeeUnitType::SatVb | FeeUnitType::Gwei => unit_value,
        };
        let is_selected = matches!(selection, GemConfirmFeeSelection::Priority { priority } if *priority == rate.priority);
        let value = fee_rate_text(unit_type, &display_value, unit_decimals, &fee_asset.symbol);
        GemFeeRateRow::new(GemFeeRateKind::Priority { priority: rate.priority }, fee, Some(value), is_selected)
    });
    let custom = (custom_fee_enabled(chain) && rates.len() > 1).then(|| {
        GemFeeRateRow::new(
            GemFeeRateKind::Custom,
            custom_total.as_ref().map(|_| loaded_fee.fee.clone()),
            custom_total.as_ref().map(|total| fee_rate_text(unit_type, total, unit_decimals, &fee_asset.symbol)),
            custom_total.is_some(),
        )
    });
    GemFeeRateRows {
        rows: rows.chain(custom).collect(),
        unit_type,
        unit_decimals,
        shows_options: rates.len() > 1,
        selected,
        normal: base_rate.map(|rate| rate.gas_price_type.clone()),
        base_fee: base_rate.and_then(|rate| match &rate.gas_price_type {
            GasPriceType::Eip1559 { gas_price, .. } => Some(gas_price.clone()),
            GasPriceType::Regular { .. } | GasPriceType::Solana { .. } => None,
        }),
    }
}

fn base_fee_rate(rates: &[GemFeeRate]) -> Option<&GemFeeRate> {
    rates.iter().find(|rate| rate.priority == FeePriority::Normal).or_else(|| rates.first())
}

pub(super) fn confirmation_fee_rates(asset_id: &AssetId, is_max_amount: bool, rates: Vec<GemFeeRate>) -> Vec<GemFeeRate> {
    let increase_percent = match EVMChain::from_chain(asset_id.chain) {
        Some(chain) if is_max_amount && asset_id.is_native() => chain.chain_stack().max_amount_base_fee_increase_percent(),
        _ => 0,
    };
    let mut rates = rates;
    for rate in &mut rates {
        match &mut rate.gas_price_type {
            GasPriceType::Eip1559 { gas_price, .. } => *gas_price = &*gas_price * (100 + increase_percent) / 100u32,
            GasPriceType::Regular { .. } | GasPriceType::Solana { .. } => {}
        }
    }
    rates.sort_by_key(|rate| match rate.priority {
        FeePriority::Normal => 0,
        FeePriority::Fast => 1,
    });
    rates
}

impl GemConfirmFeeSelection {
    pub(super) fn applied(&self, rate: &GemFeeRate) -> Self {
        match self {
            Self::Priority { .. } => Self::Priority { priority: rate.priority },
            Self::Custom { .. } => self.clone(),
        }
    }

    pub(super) fn select_fee_rate(&self, rates: &[GemFeeRate]) -> Result<GemFeeRate, GemConfirmError> {
        match self {
            Self::Priority { priority } => rates.iter().find(|rate| &rate.priority == priority).or_else(|| rates.first()).cloned().ok_or(GemConfirmError::FeeRatesMissing),
            Self::Custom { base_fee, rate } => {
                let network = base_fee_rate(rates).ok_or(GemConfirmError::FeeRatesMissing)?;
                let rate = rate.clone().unwrap_or_else(|| rate_of(&network.gas_price_type));
                let gas_price_type = match &network.gas_price_type {
                    GasPriceType::Eip1559 { gas_price, .. } => GasPriceType::eip1559(base_fee.clone().unwrap_or_else(|| gas_price.clone()), rate),
                    GasPriceType::Regular { .. } | GasPriceType::Solana { .. } => GasPriceType::regular(rate),
                };
                Ok(GemFeeRate { priority: network.priority, gas_price_type })
            }
        }
    }
}

pub fn confirm_row_contents(transfer: &GemTransferData, wallet: Wallet, address_name: Option<AddressName>, address_url: impl Fn(Chain, String) -> BlockExplorerLink) -> Vec<GemConfirmRowContent> {
    let asset = transfer.input_asset();
    let chain = asset.chain();
    transfer
        .confirm_rows()
        .into_iter()
        .filter_map(|row| match row {
            GemConfirmRow::App => transfer.application_short_name().map(|name| {
                let metadata = match &transfer.input_type {
                    TransactionInputType::Generic { metadata, .. } => Some(metadata),
                    _ => None,
                };
                GemConfirmRowContent::Row {
                    row: GemListRow::app(name, metadata.and_then(application::icon_url), metadata.map(|metadata| metadata.url.clone()).non_empty()),
                }
            }),
            GemConfirmRow::Sender => wallet.account(chain).map(|account| GemConfirmRowContent::Row {
                row: GemListRow::wallet(wallet_row(wallet.clone()), address_copy(chain, account.address.clone()), address_url(chain, account.address.clone())),
            }),
            GemConfirmRow::Recipient => transfer.destination().map(|destination| {
                let destination = destination.with_address_name(address_name.clone());
                let avatar = contact_avatar(address_name.as_ref(), destination.name().as_deref());
                let address = destination.address();
                let short_address = format_address(&address, Some(chain), GemAddressFormatStyle::Short);
                let is_named_by_user = address_name.as_ref().is_some_and(|address_name| names_the_user_owns(&address_name.address_type));
                let name = GemAddressService::new().name_text(destination.name(), short_address.clone(), is_named_by_user || !destination.shows_address_beside_name());
                let text = match &destination {
                    GemConfirmDestination::Resource { resource } => GemLocalizedText::Resource { resource: *resource },
                    _ => GemLocalizedText::Text { text: name.unwrap_or(short_address) },
                };
                let link = address_url(chain, address.clone());
                GemConfirmRowContent::Recipient {
                    row: GemAddressRow {
                        text,
                        avatar,
                        is_selectable: !address.is_empty(),
                        ..GemAddressRow::new(GemLocalizedText::ConfirmDestination { destination: destination.clone() }, chain, address, destination.name().as_deref(), &link)
                    },
                }
            }),
            GemConfirmRow::Network => {
                let text = asset_text(&asset);
                Some(GemConfirmRowContent::Row {
                    row: GemListRow::Network {
                        title: GemListRowTitle::Network,
                        chain,
                        name: match transfer.input_type {
                            TransactionInputType::Transfer { .. } | TransactionInputType::Deposit { .. } | TransactionInputType::Withdrawal { .. } => text.network_full_name,
                            _ => text.network_name,
                        },
                    },
                })
            }
            GemConfirmRow::Memo => {
                let memo = transfer.recipient.memo.clone().filter(|memo| !memo.trim().is_empty());
                Some(GemConfirmRowContent::Row {
                    row: GemListRow::memo(text_or_placeholder(memo.as_deref()), memo),
                })
            }
            GemConfirmRow::Details => Some(GemConfirmRowContent::Details),
            GemConfirmRow::PaymentAsset => match &transfer.input_type {
                TransactionInputType::Payment { asset, invoice, .. } => Some(GemConfirmRowContent::PaymentAsset {
                    title: GemListRowTitle::PayWith,
                    symbol: asset.symbol.clone(),
                    selectable: invoice.quotes.len() > 1,
                    asset_ids: invoice.quotes.iter().map(|quote| quote.asset_id.clone()).collect(),
                }),
                _ => None,
            },
        })
        .collect()
}

pub fn submit_message(input_type: &TransactionInputType, warning: Option<GemErrorText>) -> Option<GemSubmitMessage> {
    if let Some(text) = warning {
        return Some(GemSubmitMessage::Warning { text });
    }
    let TransactionInputType::Perpetual { perpetual_type, .. } = input_type else {
        return None;
    };
    let action = match perpetual_type {
        PerpetualType::Open { data } => GemPerpetualConfirmedAction::Open { direction: data.direction.clone() },
        PerpetualType::Close { .. } => GemPerpetualConfirmedAction::Close,
        PerpetualType::Modify { .. } => GemPerpetualConfirmedAction::Modify,
        PerpetualType::Increase { .. } => GemPerpetualConfirmedAction::Increase,
        PerpetualType::Reduce { .. } => GemPerpetualConfirmedAction::Reduce,
    };
    Some(GemSubmitMessage::Confirmed {
        text: GemLocalizedText::PerpetualConfirmed { action },
    })
}

pub fn confirm_sections(rows: Vec<GemConfirmRowContent>, warnings: Vec<GemListRow>, simulation: Option<GemConfirmSimulation>, verifies: bool, load_error: Option<GemConfirmError>) -> Vec<GemConfirmSection> {
    let notice = load_error.as_ref().and_then(GemConfirmError::notice);
    let (primary, secondary, changes) = simulation.map(|simulation| (simulation.primary_fields, simulation.secondary_fields, simulation.balance_changes)).unwrap_or_default();
    [
        Some(GemConfirmSection::Header),
        notice.clone().map(|row| GemConfirmSection::Notice { row }),
        Some(GemConfirmSection::Details { rows }),
        (!warnings.is_empty()).then_some(GemConfirmSection::Warnings { rows: warnings }),
        (!primary.is_empty()).then_some(GemConfirmSection::Payload { primary, secondary }),
        (!changes.is_empty()).then_some(GemConfirmSection::BalanceChanges { rows: changes }),
        Some(match verifies {
            true => GemConfirmSection::Verification,
            false => GemConfirmSection::NetworkFee,
        }),
        load_error.filter(|_| notice.is_none()).map(|error| GemConfirmSection::Error { error }),
    ]
    .into_iter()
    .flatten()
    .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::copy::GemCopy;
    use crate::models::list::GemRowMenuItem;
    use crate::services::confirm::testkit::gwei;

    #[test]
    fn test_the_confirm_screen_lists_its_blocks_in_one_order() {
        let text = |value: &str| crate::services::simulation::GemSimulationPayloadRow {
            title: crate::services::simulation::GemSimulationPayloadTitle::Method,
            value: crate::services::simulation::GemSimulationPayloadValue::Text { text: value.to_string() },
        };
        let asset = Asset::mock_eth();
        let change = GemListRow::AssetChange {
            name: asset.name.clone(),
            icon: crate::services::assets::icon::asset_icon(&asset.id),
            amount: GemFormattedNumber::amount(1.0, Some("ETH".to_string()), GemValueStyle::Auto),
        };
        let simulation = GemConfirmSimulation {
            primary_fields: vec![text("approve")],
            secondary_fields: vec![text("nonce")],
            header: None,
            balance_changes: vec![change.clone()],
            has_critical_warning: false,
        };
        let warning = GemListRow::Text {
            title: GemListRowTitle::Warning,
            value: "careful".to_string(),
        };

        assert_eq!(
            confirm_sections(vec![], vec![warning.clone()], Some(simulation), false, None),
            vec![
                GemConfirmSection::Header,
                GemConfirmSection::Details { rows: vec![] },
                GemConfirmSection::Warnings { rows: vec![warning] },
                GemConfirmSection::Payload {
                    primary: vec![text("approve")],
                    secondary: vec![text("nonce")],
                },
                GemConfirmSection::BalanceChanges { rows: vec![change] },
                GemConfirmSection::NetworkFee,
            ]
        );
        assert_eq!(
            confirm_sections(vec![], vec![], None, true, None),
            vec![GemConfirmSection::Header, GemConfirmSection::Details { rows: vec![] }, GemConfirmSection::Verification],
            "a payment to verify shows its verification instead of the fee, and empty simulation blocks are left out"
        );
    }

    #[test]
    fn test_a_notice_replaces_the_load_error_it_explains() {
        let malicious = confirm_sections(vec![], vec![], None, false, Some(GemConfirmError::ScanMalicious));
        let offline = confirm_sections(vec![], vec![], None, false, Some(GemConfirmError::Offline));

        assert!(matches!(malicious[1], GemConfirmSection::Notice { .. }));
        assert!(!malicious.iter().any(|section| matches!(section, GemConfirmSection::Error { .. })));
        assert_eq!(offline.last(), Some(&GemConfirmSection::Error { error: GemConfirmError::Offline }));
        assert!(!offline.iter().any(|section| matches!(section, GemConfirmSection::Notice { .. })));
    }
    use crate::models::custom_types::GemBigInt;
    use crate::models::custom_types::GemBigUint;
    use crate::models::transaction::GemFeeOptions;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use num_bigint::BigInt;
    use num_bigint::BigUint;
    use primitives::FeeOption;
    use primitives::{
        Account, ApplicationMetadata, Asset, PerpetualConfirmData, PerpetualDirection, PerpetualType, SimulationWarning, StakeType, SwapProvider, TransactionType, TransferAmount, TransferDataExtra, TransferDataOutputAction,
        known_assets::HYPERCORE_PERPETUAL_USDC,
        swap::{ApprovalData, SwapData},
    };
    use primitives::{AddressName, AddressType, Delegation, DelegationValidator, VerificationStatus};
    use std::collections::HashMap;

    #[test]
    fn test_a_submit_names_its_warning_or_the_confirmed_position() {
        let open = TransactionInputType::Perpetual {
            asset: Asset::mock(),
            perpetual_type: PerpetualType::Open {
                data: primitives::PerpetualConfirmData::mock(primitives::PerpetualDirection::Long, 0, None, None),
            },
        };
        let warning = GemErrorText::Message { text: "gateway".into() };

        assert_eq!(
            submit_message(&open, None),
            Some(GemSubmitMessage::Confirmed {
                text: GemLocalizedText::PerpetualConfirmed {
                    action: GemPerpetualConfirmedAction::Open {
                        direction: primitives::PerpetualDirection::Long
                    }
                }
            })
        );
        assert_eq!(submit_message(&open, Some(warning.clone())), Some(GemSubmitMessage::Warning { text: warning }), "a warning wins");
        assert_eq!(submit_message(&TransactionInputType::Transfer { asset: Asset::mock() }, None), None, "a plain send shows nothing");
    }
    #[test]
    fn test_signer_input_uses_wallet_account_and_network_fee() {
        let input = SendInput::mock(Chain::Solana, TransactionInputType::Transfer { asset: Asset::mock_sol() });

        let signer_input = input.signer_input().unwrap();

        assert_eq!(signer_input.input.sender_address, "sender");
        assert_eq!(signer_input.input.destination_address, "recipient");
        assert_eq!(signer_input.input.value, BigUint::from(9u64));
        assert_eq!(signer_input.input.memo.as_deref(), Some("memo"));
        assert!(signer_input.input.is_max_value);
        assert_eq!(signer_input.fee.fee, BigInt::from(1));
        assert_eq!(signer_input.fee.gas_limit, BigInt::from(21_000));
    }

    #[test]
    fn test_signer_input_refuses_to_sign_for_an_address_the_transaction_was_not_priced_for() {
        let matching = SendInput::mock(Chain::Solana, TransactionInputType::Transfer { asset: Asset::mock_sol() });
        assert!(matching.signer_input().is_ok());

        let mut switched = matching;
        switched.confirm.input.from = Account::mock(Chain::Solana, "other");

        assert_eq!(
            switched.signer_input().unwrap_err(),
            GemConfirmError::SenderMismatch {
                from: "other".to_string(),
                signer: "sender".to_string(),
            }
        );
    }

    #[test]
    fn test_signer_input_requires_account_for_chain() {
        let input = SendInput::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_sol() });

        match input.signer_input() {
            Err(GemConfirmError::AccountMissing { chain: Chain::Solana }) => {}
            result => panic!("expected a missing account error, got {result:?}"),
        }
    }

    #[test]
    fn test_only_a_token_approve_input_can_sign_a_token_approval() {
        let approval = TransactionInputType::TokenApprove {
            asset: Asset::mock_sol(),
            approval_data: ApprovalData::mock(),
        };
        assert!(approval.validate_approvals(&[GemSignedTransaction::mock(TransactionType::TokenApproval)]).is_ok());

        let transfer = TransactionInputType::Transfer { asset: Asset::mock_sol() };
        assert!(transfer.validate_approvals(&[GemSignedTransaction::mock(TransactionType::Transfer)]).is_ok());
        match transfer.validate_approvals(&[GemSignedTransaction::mock(TransactionType::TokenApproval)]) {
            Err(GemConfirmError::ApprovalInvalid { .. }) => {}
            result => panic!("expected an invalid approval error, got {result:?}"),
        }
    }

    #[test]
    fn test_is_broadcast() {
        let payment = TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock_signature(vec![], None));

        assert!(is_broadcast(&payment, &GemSignedTransaction::mock(TransactionType::TokenApproval)), "a payment sends its approval");
        assert!(!is_broadcast(&payment, &GemSignedTransaction::mock(TransactionType::Transfer)), "and hands over its signature");
        assert!(is_broadcast(&TransactionInputType::Transfer { asset: Asset::mock_sol() }, &GemSignedTransaction::mock(TransactionType::Transfer)));

        let approve_request = TransactionInputType::Generic {
            asset: Asset::mock_erc20(),
            metadata: ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_action: TransferDataOutputAction::Sign,
                transaction_type: TransactionType::TokenApproval,
                approval: Some(ApprovalData::mock()),
                ..TransferDataExtra::mock()
            },
        };
        assert!(
            !is_broadcast(&approve_request, &GemSignedTransaction::mock(TransactionType::TokenApproval)),
            "an approval a dapp only asked to sign is handed back, never sent"
        );
    }

    #[test]
    fn test_is_signature_only() {
        let payment = |approval| TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock_signature(vec![], approval));

        assert!(is_signature_only(&payment(None)));
        assert!(!is_signature_only(&payment(Some(ApprovalData::mock()))), "the approval is a transaction");
        assert!(!is_signature_only(&TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock())));
        assert!(!is_signature_only(&TransactionInputType::Transfer { asset: Asset::mock_erc20() }));
    }

    #[test]
    fn test_output_action_only_generic_transfers_can_sign() {
        let generic = TransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata: ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_action: TransferDataOutputAction::Sign,
                ..TransferDataExtra::mock()
            },
        };

        assert_eq!(generic.output().output_action, TransferDataOutputAction::Sign);
        assert_eq!((TransactionInputType::Transfer { asset: Asset::mock_sol() }).output().output_action, TransferDataOutputAction::Send);
    }
    fn custom_row(rows: &GemFeeRateRows) -> Option<&GemFeeRateRow> {
        rows.rows.iter().find(|row| row.kind == GemFeeRateKind::Custom)
    }

    #[test]
    fn test_fee_rate_rows_scale_the_loaded_fee_by_each_rate() {
        let rates = vec![GemFeeRate::mock(FeePriority::Normal, 10), GemFeeRate::mock(FeePriority::Fast, 25)];
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let normal = GemConfirmFeeSelection::Priority { priority: FeePriority::Normal };

        let rows = fee_rate_rows(Chain::Ethereum, &ethereum, &rates, &normal, &GemTransactionLoadFee::mock(1_000));
        assert_eq!(rows.rows[0].fee, Some(BigInt::from(1_000)), "the selected rate shows the fee that was loaded");
        assert_eq!(rows.rows[1].fee, Some(BigInt::from(2_500)), "another rate scales the loaded fee by its unit value");
        assert_eq!((rows.unit_type, rows.unit_decimals), (FeeUnitType::Gwei, 9));
        assert!(custom_row(&rows).is_some(), "an evm chain takes a custom rate");
        assert!(
            custom_row(&fee_rate_rows(Chain::Solana, &Asset::from_chain(Chain::Solana), &rates, &normal, &GemTransactionLoadFee::mock(1_000))).is_none(),
            "solana takes no custom rate"
        );
        assert_eq!((rows.selected, rows.normal), (Some(GasPriceType::regular(10)), Some(GasPriceType::regular(10))));
        assert_eq!(rows.rows.iter().map(|row| row.is_selected).collect::<Vec<_>>(), vec![true, false, false], "only the selected priority is highlighted");

        let bitcoin = Asset::from_chain(Chain::Bitcoin);
        let custom = fee_rate_rows(
            Chain::Bitcoin,
            &bitcoin,
            &rates,
            &GemConfirmFeeSelection::Custom {
                base_fee: None,
                rate: Some(BigInt::from(50)),
            },
            &GemTransactionLoadFee::mock(5_000),
        );
        assert_eq!(custom.rows[0].fee, Some(BigInt::from(1_000)), "a custom rate is the base the loaded fee was computed for");
        assert_eq!(custom.selected, Some(GasPriceType::regular(50)), "a custom rate is the rate the fee was loaded with");
        assert!(custom.rows.iter().filter(|row| row.kind != GemFeeRateKind::Custom).all(|row| !row.is_selected), "a custom rate highlights no priority row");

        let zero = fee_rate_rows(
            Chain::Solana,
            &ethereum,
            &rates,
            &GemConfirmFeeSelection::Custom { base_fee: None, rate: Some(BigInt::ZERO) },
            &GemTransactionLoadFee::mock(5_000),
        );
        assert!(zero.rows.iter().all(|row| row.fee.is_none()), "nothing scales against a zero base");

        let offered = fee_rate_rows(Chain::Bitcoin, &bitcoin, &rates, &normal, &GemTransactionLoadFee::mock(5_000));
        assert_eq!(
            offered.rows.iter().map(|row| (row.title, row.emoji.as_str())).collect::<Vec<_>>(),
            vec![(GemListRowTitle::NormalFee, "💎"), (GemListRowTitle::FastFee, "⚡️"), (GemListRowTitle::CustomFee, "⚙️")],
            "the custom row comes last, after the priorities"
        );
        let unpicked = custom_row(&offered).unwrap();
        assert_eq!((unpicked.fee.clone(), unpicked.value.clone(), unpicked.is_selected), (None, None, false), "an unpicked custom row shows no rate");
        let picked = fee_rate_rows(
            Chain::Bitcoin,
            &bitcoin,
            &rates,
            &GemConfirmFeeSelection::Custom {
                base_fee: None,
                rate: Some(BigInt::from(20)),
            },
            &GemTransactionLoadFee::mock(5_000),
        );
        let picked = custom_row(&picked).unwrap();
        assert_eq!(picked.fee, Some(BigInt::from(5_000)), "a picked custom row shows the fee that was loaded");
        assert_eq!(picked.value, Some(fee_rate_text(FeeUnitType::SatVb, &BigInt::from(20), 1, "BTC")), "a custom selection reads back its rate");
        assert!(picked.is_selected);
        assert!(
            custom_row(&fee_rate_rows(Chain::Bitcoin, &bitcoin, &rates[..1], &normal, &GemTransactionLoadFee::mock(5_000))).is_none(),
            "one rate is nothing to pick a custom value against"
        );

        let solana = Asset::from_chain(Chain::Solana);
        let native = fee_rate_rows(Chain::Solana, &solana, &rates[..1], &normal, &GemTransactionLoadFee::mock(5_000));
        assert_eq!((native.unit_type, native.unit_decimals), (FeeUnitType::Native, solana.decimals as u32));
    }

    #[test]
    fn test_an_evm_custom_price_reads_as_a_total_next_to_the_priorities() {
        let rates = vec![
            GemFeeRate {
                priority: FeePriority::Normal,
                gas_price_type: GasPriceType::eip1559(gwei(24), gwei(1)),
            },
            GemFeeRate {
                priority: FeePriority::Fast,
                gas_price_type: GasPriceType::eip1559(gwei(24), gwei(2)),
            },
        ];
        let ethereum = Asset::from_chain(Chain::Ethereum);

        let normal = fee_rate_rows(Chain::Ethereum, &ethereum, &rates, &GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }, &GemTransactionLoadFee::mock(21_000));
        assert_eq!(normal.selected, Some(GasPriceType::eip1559(gwei(24), gwei(1))), "the fields start from the picked priority");
        assert_eq!(normal.base_fee, Some(gwei(24)), "the normal row's base fee is the floor of the base fee field");

        let custom = fee_rate_rows(
            Chain::Ethereum,
            &ethereum,
            &rates,
            &GemConfirmFeeSelection::Custom { base_fee: None, rate: Some(gwei(5)) },
            &GemTransactionLoadFee::mock(21_000),
        );
        assert_eq!(custom.selected, Some(GasPriceType::eip1559(gwei(24), gwei(5))), "a custom price is the price the fee was loaded with");
        assert_eq!(
            custom_row(&custom).unwrap().value,
            Some(fee_rate_text(FeeUnitType::Gwei, &gwei(29), 9, "ETH")),
            "the custom row reads the total price like the priorities around it"
        );
    }

    #[test]
    fn test_a_row_displays_the_fee_on_a_native_unit_chain_and_the_rate_elsewhere() {
        let rates = vec![GemFeeRate::mock(FeePriority::Normal, 10), GemFeeRate::mock(FeePriority::Fast, 25)];
        let normal = GemConfirmFeeSelection::Priority { priority: FeePriority::Normal };

        let gwei = fee_rate_rows(Chain::Ethereum, &Asset::from_chain(Chain::Ethereum), &rates, &normal, &GemTransactionLoadFee::mock(1_000));
        assert_eq!(gwei.rows[1].value, Some(fee_rate_text(FeeUnitType::Gwei, &BigInt::from(25), 9, "ETH")), "a gwei row shows the rate the user picks");

        let native = fee_rate_rows(Chain::Solana, &Asset::from_chain(Chain::Solana), &rates, &normal, &GemTransactionLoadFee::mock(1_000));
        assert_eq!(
            native.rows[1].value,
            Some(fee_rate_text(FeeUnitType::Native, &BigInt::from(2_500), 9, "SOL")),
            "a native-unit row shows what the transfer costs"
        );

        let unscaled = fee_rate_rows(
            Chain::Solana,
            &Asset::from_chain(Chain::Solana),
            &rates,
            &GemConfirmFeeSelection::Custom { base_fee: None, rate: Some(BigInt::ZERO) },
            &GemTransactionLoadFee::mock(1_000),
        );
        assert_eq!(unscaled.rows[1].value, Some(fee_rate_text(FeeUnitType::Native, &BigInt::from(25), 9, "SOL")), "with no fee to scale, the rate stands in");
    }

    #[test]
    fn test_fee_rate_rows_keep_fixed_options_out_of_the_scaling() {
        let rates = vec![GemFeeRate::mock(FeePriority::Normal, 10000), GemFeeRate::mock(FeePriority::Fast, 20000)];
        let solana = Asset::from_chain(Chain::Solana);
        let rent = BigInt::from(2_039_280u64);
        let loaded = GemTransactionLoadFee {
            options: GemFeeOptions {
                options: HashMap::from([(FeeOption::TokenAccountCreation, rent.clone())]),
            },
            ..GemTransactionLoadFee::mock(2_049_280)
        };

        let rows = fee_rate_rows(Chain::Solana, &solana, &rates, &GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }, &loaded);
        assert_eq!(rows.rows[0].fee, Some(loaded.fee), "the selected rate shows the fee that was loaded");
        assert_eq!(rows.rows[1].fee, Some(BigInt::from(20_000) + &rent), "the rent does not grow with the priority");

        let fast = fee_rate_rows(
            Chain::Solana,
            &solana,
            &rates,
            &GemConfirmFeeSelection::Priority { priority: FeePriority::Fast },
            &GemTransactionLoadFee {
                options: GemFeeOptions {
                    options: HashMap::from([(FeeOption::TokenAccountCreation, rent.clone())]),
                },
                ..GemTransactionLoadFee::mock(2_059_280)
            },
        );
        assert_eq!(fast.rows[0].fee, Some(BigInt::from(10_000) + &rent), "the rent does not shrink with the priority either");
    }

    #[test]
    fn test_fee_rate_rows_highlight_the_priority_core_selected_when_the_asked_one_is_not_offered() {
        let rates = vec![GemFeeRate::mock(FeePriority::Normal, 10)];
        let asked = GemConfirmFeeSelection::Priority { priority: FeePriority::Fast };
        let selected = asked.select_fee_rate(&rates).unwrap();
        let confirm = GemConfirmData {
            selected_priority: selected.priority,
            fee_selection: asked.applied(&selected),
            fee_rates: rates,
            ..GemConfirmData::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock() })
        };
        let rows = confirm.fee_rate_rows(&Asset::mock(), Some(2.0), Currency::USD);

        assert_eq!(rows.selected, Some(GasPriceType::regular(10)));
        assert_eq!(
            rows.rows.iter().map(|row| (row.kind, row.is_selected)).collect::<Vec<_>>(),
            vec![(GemFeeRateKind::Priority { priority: FeePriority::Normal }, true)]
        );
        assert!(
            rows.rows.iter().all(|row| row.amount == row.fee.as_ref().map(|fee| fee_amount(&Asset::mock(), fee, Some(2.0), Currency::USD))),
            "each rate row carries its fee in the fee asset and the currency"
        );
    }

    #[test]
    fn test_confirmation_fee_rates_list_normal_before_fast() {
        let rates = confirmation_fee_rates(&AssetId::from_chain(Chain::Ethereum), false, vec![GemFeeRate::mock(FeePriority::Fast, 20), GemFeeRate::mock(FeePriority::Normal, 10)]);
        assert_eq!(rates.iter().map(|rate| rate.priority).collect::<Vec<_>>(), vec![FeePriority::Normal, FeePriority::Fast]);
    }

    #[test]
    fn test_confirmation_fee_rates_keep_the_base_fee_except_for_a_max_send_on_a_stack_that_needs_room() {
        let native = AssetId::from_chain;
        let token = |chain| AssetId::from_token(chain, "0x1111111111111111111111111111111111111111");
        for (asset_id, is_max_amount, base_fee) in [
            (native(Chain::Ethereum), false, 100),
            (native(Chain::Ethereum), true, 100),
            (token(Chain::Ethereum), true, 100),
            (native(Chain::Arbitrum), false, 100),
            (native(Chain::Arbitrum), true, 105),
            (native(Chain::Robinhood), true, 105),
            (token(Chain::Robinhood), true, 100),
            (native(Chain::Optimism), false, 100),
            (native(Chain::Optimism), true, 100),
            (native(Chain::ZkSync), true, 100),
        ] {
            let rates = confirmation_fee_rates(
                &asset_id,
                is_max_amount,
                vec![
                    GemFeeRate {
                        priority: FeePriority::Normal,
                        gas_price_type: GasPriceType::eip1559(100, 5),
                    },
                    GemFeeRate {
                        priority: FeePriority::Fast,
                        gas_price_type: GasPriceType::eip1559(100, 10),
                    },
                ],
            );
            assert_eq!(
                rates.iter().map(|rate| rate.gas_price_type.clone()).collect::<Vec<_>>(),
                vec![GasPriceType::eip1559(base_fee, 5), GasPriceType::eip1559(base_fee, 10)],
                "{asset_id} max={is_max_amount}"
            );
        }
    }

    #[test]
    fn test_select_fee_rate() {
        let rates = vec![GemFeeRate::mock(FeePriority::Normal, 10), GemFeeRate::mock(FeePriority::Fast, 20)];

        let fast = (GemConfirmFeeSelection::Priority { priority: FeePriority::Fast }).select_fee_rate(&rates).unwrap();
        assert_eq!(fast.priority, FeePriority::Fast);

        let fallback = (GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }).select_fee_rate(&[GemFeeRate::mock(FeePriority::Fast, 20)]).unwrap();
        assert_eq!(fallback.priority, FeePriority::Fast);

        let custom = (GemConfirmFeeSelection::Custom {
            base_fee: None,
            rate: Some(BigInt::from(33)),
        })
        .select_fee_rate(&rates)
        .unwrap();
        assert_eq!(custom.priority, FeePriority::Normal);
        match custom.gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, BigInt::from(33)),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match (GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }).select_fee_rate(&[]) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_select_fee_rate_custom() {
        let rates = vec![
            GemFeeRate {
                priority: FeePriority::Normal,
                gas_price_type: GasPriceType::eip1559(gwei(20), gwei(1)),
            },
            GemFeeRate {
                priority: FeePriority::Fast,
                gas_price_type: GasPriceType::eip1559(gwei(20), gwei(2)),
            },
        ];
        let select = |base_fee: Option<BigInt>, rate: Option<BigInt>| (GemConfirmFeeSelection::Custom { base_fee, rate }).select_fee_rate(&rates).unwrap();

        let tip = select(None, Some(gwei(3)));
        assert_eq!(tip.priority, FeePriority::Normal, "a custom price stands in for the normal rate");
        assert_eq!(tip.gas_price_type, GasPriceType::eip1559(gwei(20), gwei(3)), "an untyped base fee follows the network");
        assert_eq!(
            select(Some(gwei(25)), None).gas_price_type,
            GasPriceType::eip1559(gwei(25), gwei(1)),
            "a typed base fee is kept and an untyped tip follows the normal one"
        );
        assert_eq!(select(Some(gwei(25)), Some(gwei(3))).gas_price_type, GasPriceType::eip1559(gwei(25), gwei(3)), "typed values are signed as typed");

        let bitcoin = (GemConfirmFeeSelection::Custom { base_fee: None, rate: Some(BigInt::from(4)) })
            .select_fee_rate(&[GemFeeRate::mock(FeePriority::Fast, 9)])
            .unwrap();
        assert_eq!((bitcoin.priority, bitcoin.gas_price_type), (FeePriority::Fast, GasPriceType::regular(4)));

        match (GemConfirmFeeSelection::Custom { base_fee: None, rate: None }).select_fee_rate(&[]) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_validate_simulation() {
        for chain in [Chain::Solana, Chain::Ethereum, Chain::Sui] {
            let payment = TransactionInputType::mock_payment(Asset::from_chain(chain), TransferDataExtra::mock());
            let wallet_connect = TransactionInputType::Generic {
                asset: Asset::from_chain(chain),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock(),
            };
            for warning in [SimulationWarning::execution_error("InstructionError"), SimulationWarning::validation_error("Invalid transaction")] {
                let message = warning.message.clone().unwrap();
                let simulation = SimulationResult::new(vec![warning], vec![]);
                assert_eq!(payment.validate_simulation(&simulation).map_err(|error| error.to_string()), Err(message));
                assert!(wallet_connect.validate_simulation(&simulation).is_ok());
            }
            assert!(payment.validate_simulation(&SimulationResult::default()).is_ok());
            assert!(payment.validate_simulation(&SimulationResult::new(vec![SimulationWarning::mock(SimulationWarningType::SuspiciousSpender)], vec![])).is_ok());
            assert_eq!(
                payment
                    .validate_simulation(&SimulationResult::new(vec![SimulationWarning::mock(SimulationWarningType::ValidationError)], vec![]))
                    .map_err(|error| error.to_string()),
                Err("Payment simulation failed".to_string())
            );
        }
    }

    #[test]
    fn test_broadcast_policy() {
        let transfer = TransactionInputType::Transfer { asset: Asset::mock_sol() };
        let swap = TransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        for (chain, skip_preflight) in [(Chain::Solana, true), (Chain::Ethereum, false), (Chain::Sui, false)] {
            let wallet_connect = TransactionInputType::Generic {
                asset: Asset::from_chain(chain),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock(),
            };
            let payment = TransactionInputType::mock_payment(Asset::from_chain(chain), TransferDataExtra::mock());
            assert_eq!(wallet_connect.broadcast_options().skip_preflight, skip_preflight, "{chain}: wallet connect");
            assert!(!payment.broadcast_options().skip_preflight, "{chain}: payment");
        }

        let approve = TransactionInputType::TokenApprove {
            asset: Asset::mock_sol(),
            approval_data: ApprovalData::mock(),
        };
        let stake = TransactionInputType::Stake {
            asset: Asset::mock_sol(),
            stake_type: StakeType::Rewards(vec![]),
        };
        let perpetual = TransactionInputType::Perpetual {
            asset: Asset::mock_sol(),
            perpetual_type: PerpetualType::Open {
                data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
            },
        };
        let ethereum_swap = TransactionInputType::Swap {
            from_asset: Asset::mock(),
            to_asset: Asset::mock_erc20(),
            swap_data: SwapData::mock(),
        };

        assert!(swap.broadcast_options().skip_preflight);
        assert!(!transfer.broadcast_options().skip_preflight);
        assert!(!approve.broadcast_options().skip_preflight);
        assert!(!stake.broadcast_options().skip_preflight);
        assert!(!perpetual.broadcast_options().skip_preflight);
        assert!(!ethereum_swap.broadcast_options().skip_preflight);

        assert_eq!(broadcast_delay_milliseconds(Chain::Ethereum), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::HyperCore), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::Solana), 500);
        assert_eq!(broadcast_delay_milliseconds(Chain::Bitcoin), 500);
        assert_eq!(broadcast_delay_milliseconds(Chain::Polygon), 0);
    }

    #[test]
    fn test_simulation_payload_only_for_utf8_payment_calls() {
        let extra = TransferDataExtra {
            data: Some(b"0xdeadbeef".to_vec()),
            ..TransferDataExtra::mock()
        };
        let payment = |extra: TransferDataExtra| TransactionInputType::mock_payment(Asset::mock_sol(), extra);

        assert_eq!(payment(extra.clone()).simulation_payload(), Some("0xdeadbeef".to_string()));
        assert_eq!(
            TransactionInputType::Generic {
                asset: Asset::mock_sol(),
                metadata: ApplicationMetadata::mock(),
                extra: extra.clone(),
            }
            .simulation_payload(),
            None
        );
        assert_eq!(
            payment(TransferDataExtra {
                data: Some(vec![0xff, 0xfe]),
                ..extra.clone()
            })
            .simulation_payload(),
            None
        );
        assert_eq!(payment(TransferDataExtra { data: None, ..extra.clone() }).simulation_payload(), None);
        assert_eq!(payment(TransferDataExtra { data: Some(Vec::new()), ..extra }).simulation_payload(), None, "a coin transfer carries no calldata to simulate");
        assert_eq!(
            payment(TransferDataExtra::mock_signature(b"0xdeadbeef".to_vec(), None)).simulation_payload(),
            None,
            "typed data is not a transaction to simulate"
        );

        let swap = TransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(swap.simulation_payload(), None);
    }

    #[test]
    fn test_pending_transactions_follow_broadcast_hashes() {
        let mut input = SendInput::mock(Chain::Solana, TransactionInputType::Transfer { asset: Asset::mock_sol() });
        input.value = BigInt::from(10);
        let signed = vec![GemSignedTransaction::mock(primitives::TransactionType::Transfer)];
        let transactions = input.pending_transactions(&["hash".to_string()], &signed).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id.hash, "hash");
        assert_eq!(transactions[0].from, "sender");

        let mut no_account = input;
        no_account.wallet.accounts.clear();
        assert!(matches!(no_account.pending_transactions(&["hash".to_string()], &signed), Err(GemConfirmError::Record { .. })));
    }

    #[test]
    fn test_an_error_sheet_carries_the_requirement_its_fiat_and_the_way_to_acquire() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let requirement = GemBalanceRequirement::new(GemBigInt::from(3_000_000_000_000_000_000u64), GemBigInt::from(1_000_000_000_000_000_000u64));
        let display = GemConfirmErrorDisplay::NetworkFeeRequired {
            asset: asset.clone(),
            title: asset.display_title(),
            requirement: GemConfirmRequirement::new(&requirement, &asset),
        };

        let prices = vec![AssetPrice::new(asset.id.clone(), 2_000.0, 0.0, chrono::Utc::now())];
        let token = AssetId::from_token(Chain::Ethereum, "0xtoken");
        let info = error_info(&display, &prices, Currency::USD, &token, &asset.id).unwrap();

        assert_eq!(info.sheet, GemConfirmErrorSheet::NetworkFeeRequired);
        assert_eq!(info.required.as_ref().map(|number| number.value), Some(3.0), "the sheet names what the transaction requires, not the fee alone");
        assert_eq!(info.available.as_ref().map(|number| number.value), Some(1.0));
        assert_eq!(info.shortfall.as_ref().map(|number| number.value), Some(2.0));
        assert_eq!(info.required_fiat.as_ref().map(|number| number.value), Some(6_000.0));
        let acquire = info.acquire.unwrap();
        assert_eq!(acquire.flow, GemAcquireAssetFlow::Fiat);
        assert_eq!(acquire.buy_amount, Some(get_fiat_config().insufficient_network_fee_buy_amount), "a fee sheet buys the fee amount");
        assert_eq!(acquire.swap_pair, acquire_swap_pair(&token, &asset.id, asset.id.clone()));

        let balance = GemConfirmErrorDisplay::BalanceRequired {
            requirement: GemConfirmRequirement::new(&requirement, &asset),
            asset: asset.clone(),
        };
        assert_eq!(
            error_info(&balance, &prices, Currency::USD, &token, &asset.id).unwrap().acquire.unwrap().buy_amount,
            None,
            "a balance sheet leaves the amount to the user"
        );

        let usdc = HYPERCORE_PERPETUAL_USDC.clone();
        let perpetual_balance = GemConfirmErrorDisplay::BalanceRequired {
            requirement: GemConfirmRequirement::new(&requirement, &usdc),
            asset: usdc.clone(),
        };
        assert_eq!(
            error_info(&perpetual_balance, &prices, Currency::USD, &usdc.id, &usdc.id).unwrap().acquire,
            None,
            "a perpetual balance is funded by a deposit, never bought"
        );

        let without_price = error_info(&display, &[], Currency::USD, &token, &asset.id).unwrap();
        assert_eq!(without_price.required_fiat, None, "no price means no fiat, never a zero");
    }

    #[test]
    fn test_an_error_the_user_cannot_act_on_opens_no_sheet() {
        for display in [
            GemConfirmErrorDisplay::Offline,
            GemConfirmErrorDisplay::FeeRatesMissing,
            GemConfirmErrorDisplay::Cancelled,
            GemConfirmErrorDisplay::AccountMissing,
            GemConfirmErrorDisplay::Unknown,
            GemConfirmErrorDisplay::InsufficientFunds,
            GemConfirmErrorDisplay::Message { msg: "boom".to_string() },
        ] {
            assert_eq!(error_info(&display, &[], Currency::USD, &AssetId::from_chain(Chain::Ethereum), &AssetId::from_chain(Chain::Ethereum)), None, "{display:?}");
        }
    }

    #[test]
    fn test_a_sheet_without_an_asset_carries_no_amount_and_no_acquire() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let info = error_info(&GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin }, &[], Currency::USD, &bitcoin, &bitcoin).unwrap();

        assert_eq!(info.sheet, GemConfirmErrorSheet::DustThreshold { chain: Chain::Bitcoin });
        assert!(info.required.is_none() && info.available.is_none() && info.shortfall.is_none() && info.acquire.is_none());
    }

    #[test]
    fn test_acquire_asset_flow_offers_options_only_on_tron() {
        assert_eq!(acquire_asset_flow(Chain::Tron), GemAcquireAssetFlow::Options);
        assert_eq!(acquire_asset_flow(Chain::Ethereum), GemAcquireAssetFlow::Fiat);
        assert_eq!(acquire_options(GemAcquireAssetFlow::Options), vec![GemAcquireOption::Buy, GemAcquireOption::Swap, GemAcquireOption::Receive]);
        assert!(acquire_options(GemAcquireAssetFlow::Fiat).is_empty(), "the fiat flow goes straight to buying");
    }

    #[test]
    fn test_acquiring_an_asset_pays_with_the_other_one_on_the_screen() {
        let usdt = Asset::mock_ethereum_usdc().id;
        let ethereum = Asset::from_chain(Chain::Ethereum).id;

        assert_eq!(
            acquire_swap_pair(&usdt, &ethereum, usdt.clone()),
            GemSwapPairSelection {
                pay_asset_id: Some(ethereum.clone()),
                receive_asset_id: Some(usdt.clone())
            },
            "the missing transfer asset is bought with the fee asset"
        );
        assert_eq!(
            acquire_swap_pair(&usdt, &ethereum, ethereum.clone()),
            GemSwapPairSelection {
                pay_asset_id: Some(usdt),
                receive_asset_id: Some(ethereum.clone())
            },
            "the missing fee asset is bought with the transfer asset"
        );
        assert_eq!(
            acquire_swap_pair(&ethereum, &ethereum, ethereum.clone()),
            GemSwapPairSelection {
                pay_asset_id: None,
                receive_asset_id: Some(ethereum)
            },
            "an asset cannot be swapped for itself"
        );
    }

    #[test]
    fn test_default_fee_priority_is_fast_only_for_bitcoin_swaps() {
        let bitcoin_swap = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Bitcoin),
            to_asset: Asset::mock_sol(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(bitcoin_swap.default_fee_priority(), FeePriority::Fast);
        let solana_swap = TransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(solana_swap.default_fee_priority(), FeePriority::Normal);
        assert_eq!((TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Bitcoin) }).default_fee_priority(), FeePriority::Normal);
    }

    #[test]
    fn test_insufficient_network_fee_only_for_empty_native_balances() {
        let empty = GemBigUint::ZERO;
        let funded = GemBigUint::from(10u32);
        assert!(is_insufficient_network_fee(&AssetId::from_chain(Chain::Ethereum), &empty));
        assert!(!is_insufficient_network_fee(&AssetId::from_chain(Chain::Ethereum), &funded));
        assert!(!is_insufficient_network_fee(&AssetId::from_chain(Chain::Tron), &empty));
        assert!(!is_insufficient_network_fee(&AssetId::from_chain(Chain::HyperCore), &empty));
        assert!(!is_insufficient_network_fee(&AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".into())), &empty));
    }

    #[test]
    fn test_validate_scan() {
        let safe = ScanTransaction {
            is_malicious: Some(false),
            is_memo_required: Some(false),
            is_scan_complete: false,
            malicious_addresses: None,
            malicious_assets: None,
            malicious_website: None,
        };
        let malicious = ScanTransaction { is_malicious: Some(true), ..safe.clone() };
        let memo_required = ScanTransaction {
            is_memo_required: Some(true),
            ..safe.clone()
        };

        assert!(validate_scan(None, None, "USDT").is_ok());
        assert!(validate_scan(Some(&safe), None, "USDT").is_ok());
        assert!(validate_scan(Some(&memo_required), Some("deposit"), "USDT").is_ok());

        match validate_scan(Some(&malicious), Some("memo"), "USDT") {
            Err(GemConfirmError::ScanMalicious) => {}
            result => panic!("expected a malicious verdict, got {result:?}"),
        }
        match validate_scan(Some(&memo_required), Some("  "), "USDT") {
            Err(GemConfirmError::ScanMemoRequired { symbol }) => assert_eq!(symbol, "USDT"),
            result => panic!("expected a required memo, got {result:?}"),
        }
    }

    #[test]
    fn test_a_fee_asset_with_no_available_balance_is_not_selectable() {
        let funded = Asset::from_chain(Chain::Tempo);
        let empty = Asset::from_chain(Chain::Ethereum);
        let assets = vec![funded.clone(), empty.clone()];
        let balances = vec![
            GemAssetBalance {
                asset_id: funded.id.clone(),
                ..GemAssetBalance::mock_with_available(1)
            },
            GemAssetBalance {
                asset_id: empty.id,
                ..GemAssetBalance::mock_with_available(0)
            },
        ];

        let selectable = selectable_fee_assets(assets, balances, vec![], &Currency::USD);

        assert_eq!(selectable.iter().map(|fee| fee.asset.id.clone()).collect::<Vec<_>>(), vec![funded.id]);
        assert_eq!(selectable[0].row.title, funded.symbol, "a fee asset reads by its symbol");
        assert!(matches!(&selectable[0].row.trailing, GemAssetItemTrailing::Value { extra: None, .. }), "an unpriced fee asset shows its balance and no value");
    }

    #[test]
    fn test_a_max_amount_below_the_provider_minimum_reports_the_minimum_not_a_missing_balance() {
        let chain = Chain::Solana;
        let asset = Asset::from_chain(chain);
        let mut swap_data = SwapData::mock_transfer(SwapProvider::NearIntents, "50200", "1000000", "deposit");
        swap_data.quote.min_from_value = Some(num_bigint::BigUint::from(49_949u64));
        let input_type = TransactionInputType::Swap {
            from_asset: asset.clone(),
            to_asset: Asset::from_chain(Chain::Bitcoin),
            swap_data,
        };
        let mut data = SendInput::mock(chain, input_type).confirm;
        data.input.transfer.use_max_amount = true;
        data.input.transfer.value = BigInt::from(50_200);
        data.fee.fee = BigInt::from(537);

        let metadata = GemConfirmMetadata::mock(&asset.id, 50_200);

        match data.preload_amount(&metadata, &asset) {
            GemTransferAmountResult::Error {
                error: GemConfirmError::BelowSwapMinimum {
                    asset: error_asset,
                    provider,
                    provider_name,
                    requirement,
                },
            } => {
                assert_eq!(error_asset, asset);
                assert_eq!(provider, SwapProvider::NearIntents, "the sheet shows the icon of the provider that set the minimum");
                assert_eq!(provider_name, "NEAR Intents", "the sheet names it");
                assert_eq!(requirement.required, BigInt::from(49_949), "the minimum the provider asked for");
                assert_eq!(requirement.available, BigInt::from(49_663), "what the balance can send once the fee is paid");
                assert_eq!(requirement.shortfall, BigInt::from(286));
            }
            other => panic!("expected a below minimum value error, got {other:?}"),
        }
    }

    #[test]
    fn test_preload_reports_the_amount_error_instead_of_failing_the_whole_preload() {
        let chain = Chain::Solana;
        let asset = Asset::from_chain(chain);
        let mut data = SendInput::mock(chain, TransactionInputType::Transfer { asset: asset.clone() }).confirm;
        data.input.transfer.use_max_amount = false;
        data.input.transfer.value = BigInt::from(1_000);
        data.fee.fee = BigInt::from(1);

        let short = GemConfirmMetadata::mock(&asset.id, 10);
        let funded = GemConfirmMetadata::mock(&asset.id, 2_000_000);

        match data.preload_amount(&short, &asset) {
            GemTransferAmountResult::Error {
                error: GemConfirmError::InsufficientBalance { asset: error_asset, requirement },
            } => {
                assert_eq!(error_asset, asset, "the error names the asset the screen shows, not just its id");
                assert_eq!(requirement.required, BigInt::from(1_001));
                assert_eq!(requirement.available, BigInt::from(10));
                assert_eq!(requirement.shortfall, BigInt::from(991));
            }
            other => panic!("expected an insufficient balance error, got {other:?}"),
        }
        assert!(matches!(data.preload_amount(&funded, &asset), GemTransferAmountResult::Amount { .. }));
    }

    #[test]
    fn test_preload_amount_hypercore_withdrawal() {
        let usdc = HYPERCORE_PERPETUAL_USDC.clone();
        let mut data = SendInput::mock(Chain::HyperCore, TransactionInputType::Withdrawal { asset: usdc.clone() }).confirm;
        data.input.transfer.value = BigInt::from(305_000_000);
        data.fee.fee = BigInt::from(1_000_000);
        let underwater = GemAssetBalance {
            asset_id: usdc.id.clone(),
            withdrawable: GemBigUint::from(305_000_000u32),
            ..GemAssetBalance::mock_with_available(0)
        };
        let metadata = GemConfirmMetadata {
            asset_balance: underwater.clone(),
            fee_asset_balance: underwater,
            ..GemConfirmMetadata::mock(&usdc.id, 0)
        };

        assert_eq!(
            data.preload_amount(&metadata, &usdc),
            GemTransferAmountResult::Amount {
                amount: TransferAmount {
                    value: BigInt::from(304_000_000),
                    network_fee: BigInt::from(1_000_000),
                    is_max_amount: true,
                },
            },
            "the withdrawable balance pays the fee even when nothing is available for margin"
        );
    }

    #[test]
    fn test_only_a_token_approval_carries_an_approval_header_value() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let approval = TransactionInputType::TokenApprove {
            asset: asset.clone(),
            approval_data: primitives::swap::ApprovalData {
                token: String::new(),
                spender: String::new(),
                value: GemBigUint::from(42u32),
                is_unlimited: true,
            },
        };

        assert_eq!(approval.approval_value(), Some((asset.id.clone(), GemApprovalValue::Unlimited)));
        assert!((TransactionInputType::Transfer { asset }).approval_value().is_none());
        assert_eq!(approval_value_from(Some(&GemBigUint::from(42u32)), false), GemApprovalValue::Exact { value: GemBigUint::from(42u32) });
        assert_eq!(approval_value_from(Some(&GemBigUint::from(42u32)), true), GemApprovalValue::Unlimited);
    }

    #[test]
    fn test_metadata_asset_ids_keeps_one_entry_per_asset() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);
        let extra = AssetId::from_chain(Chain::Bitcoin);

        let asset_ids = metadata_asset_ids(&asset_id, &fee_asset_id, vec![extra.clone(), extra.clone(), asset_id.clone()]);

        assert_eq!(asset_ids, vec![asset_id, extra]);
    }

    #[test]
    fn test_a_balance_change_reads_every_digit_signed_and_toned_by_its_direction() {
        use crate::formatted_number::GemNumberNotation;
        let solana = Asset::from_chain(Chain::Solana);
        let usdc = Asset {
            decimals: 6,
            ..Asset::from_chain(Chain::Ethereum)
        };

        let spent = balance_change_amount(&BigInt::from(-100_005_000), &solana);
        assert_eq!(
            (spent.value, spent.exact.as_deref(), spent.notation, spent.tone),
            (-0.100005, Some("0.100005"), GemNumberNotation::Signed, GemValueTone::Negative)
        );

        let received = balance_change_amount(&BigInt::from(750_000), &usdc);
        assert_eq!((received.exact.as_deref(), received.notation, received.tone), (Some("0.75"), GemNumberNotation::Signed, GemValueTone::Positive));

        let nothing = balance_change_amount(&BigInt::ZERO, &solana);
        assert_eq!((nothing.notation, nothing.tone), (GemNumberNotation::Plain, GemValueTone::Neutral), "a change of nothing is neither a gain nor a loss");
    }

    #[test]
    fn test_build_metadata_reads_each_balance_from_its_own_asset() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);
        let balances = vec![
            GemAssetBalance {
                asset_id: fee_asset_id.clone(),
                ..GemAssetBalance::mock_with_available(7)
            },
            GemAssetBalance {
                asset_id: asset_id.clone(),
                ..GemAssetBalance::mock_with_available(3)
            },
        ];

        let metadata = build_metadata(asset_id, fee_asset_id, balances, vec![]).unwrap();

        assert_eq!(metadata.asset_balance.available, GemBigUint::from(3u32));
        assert_eq!(metadata.fee_asset_balance.available, GemBigUint::from(7u32));
    }

    #[test]
    fn test_build_metadata_rejects_a_missing_balance() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);

        match build_metadata(
            asset_id.clone(),
            fee_asset_id.clone(),
            vec![GemAssetBalance {
                asset_id: fee_asset_id.clone(),
                ..GemAssetBalance::mock_with_available(7)
            }],
            vec![],
        ) {
            Err(GemConfirmError::BalanceMissing { asset_id: missing }) => assert_eq!(missing, asset_id),
            result => panic!("expected the asset balance to be required, got {result:?}"),
        }
        match build_metadata(
            asset_id.clone(),
            fee_asset_id.clone(),
            vec![GemAssetBalance {
                asset_id,
                ..GemAssetBalance::mock_with_available(3)
            }],
            vec![],
        ) {
            Err(GemConfirmError::BalanceMissing { asset_id: missing }) => assert_eq!(missing, fee_asset_id),
            result => panic!("expected the fee balance to be required, got {result:?}"),
        }
    }

    #[test]
    fn test_preload_simulation_is_only_needed_when_the_request_has_none() {
        let mut confirm_data = SendInput::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_sol() }).confirm;
        confirm_data.simulation = Some(SimulationResult {
            warnings: vec![SimulationWarning::validation_error("preload")],
            ..SimulationResult::default()
        });

        assert!(
            preload_simulation(
                Some(&SimulationResult {
                    warnings: vec![SimulationWarning::validation_error("request")],
                    ..SimulationResult::default()
                }),
                &confirm_data
            )
            .is_none(),
            "the request simulation is already on the screen"
        );
        assert_eq!(preload_simulation(None, &confirm_data).unwrap().warnings[0].message.as_deref(), Some("preload"));
        let unsimulated = GemConfirmData { simulation: None, ..confirm_data };
        assert!(preload_simulation(None, &unsimulated).is_none());
    }

    #[test]
    fn test_with_fee_takes_the_preload_and_keeps_the_rest_of_the_screen() {
        let eth = Asset::mock_eth();
        let btc = Asset::mock_btc();
        let screen = GemConfirmLoad {
            metadata: GemConfirmMetadata::mock(&eth.id, 1),
            fee_assets: vec![GemFeeAsset {
                asset: eth.clone(),
                balance: GemAssetBalance {
                    asset_id: eth.id.clone(),
                    ..GemAssetBalance::mock_with_available(1)
                },
                price: None,
                row: fee_asset_row(&eth, &GemAssetBalance::mock_with_available(1), None, &Currency::USD),
            }],
            address_name: Some(AddressName::mock("0xrecipient", "recipient.eth", AddressType::Address, VerificationStatus::Verified)),
            ..GemConfirmLoad::mock()
        };
        let fee = GemConfirmFeeLoad {
            fee_asset: btc.clone(),
            metadata: GemConfirmMetadata::mock(&eth.id, 2),
            fee: GemConfirmFee::mock(GemTransferAmountResult::mock()),
            confirm_data: SendInput::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: eth.clone() }).confirm,
            simulation: None,
        };

        let ConfirmState { load: loaded, confirm_data, .. } = screen.clone().with_fee(fee.clone(), None, None);
        assert_eq!(loaded.fee_asset, btc);
        assert_eq!(loaded.metadata.asset_balance.available, GemBigUint::from(2u32));
        assert!(loaded.fee.is_some());
        assert!(confirm_data.is_some(), "the signing data stays in Core next to the screen");
        assert_eq!(loaded.fee_assets, screen.fee_assets, "the selectable fee assets come from the first answer");
        assert_eq!(loaded.address_name, screen.address_name, "the recipient's name comes from the first answer");
        assert!(loaded.simulation.warnings.is_empty(), "without a preload simulation the request simulation state stays");

        let resimulated = screen.with_fee(
            fee,
            Some(GemConfirmSimulationState {
                warnings: crate::services::simulation::warning_rows(&[SimulationWarning::validation_error("preload")]),
                ..GemConfirmSimulationState::mock()
            }),
            None,
        );
        assert_eq!(resimulated.load.simulation.warnings.len(), 1);
    }

    #[test]
    fn test_a_recipient_row_is_finished_before_it_leaves_core() {
        let link = |chain: Chain, address: String| BlockExplorerLink { name: chain.to_string(), link: address };
        let transfer = GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) });
        let contact = AddressName::mock("recipient", "John Smith", AddressType::Contact, VerificationStatus::Verified);
        let text = |text: &str| GemLocalizedText::Text { text: text.to_string() };
        let recipient_row = |transfer: &GemTransferData, address_name: Option<AddressName>| {
            confirm_row_contents(transfer, Wallet::mock(), address_name, link).into_iter().find_map(|content| match content {
                GemConfirmRowContent::Recipient { row } => Some(row),
                _ => None,
            })
        };

        let named = recipient_row(&transfer, Some(contact)).unwrap();
        assert_eq!(named.text, text("John Smith"), "a contact with a picture needs no address beside its name");
        assert_eq!(
            named.avatar.as_ref().map(|avatar| avatar.initials.clone()),
            Some("JO".to_string()),
            "a contact reads as the initials Core writes everywhere else"
        );
        assert_eq!(named.short_address, Some("recipient".to_string()), "a name reveals the short address on a tap");
        assert_eq!(named.address, "recipient");
        assert!(named.is_selectable);

        let unnamed = recipient_row(&transfer, None).unwrap();
        assert_eq!(unnamed.avatar, None, "an address nobody named shows no avatar");
        assert_eq!(unnamed.short_address, None, "an unnamed address has nothing to reveal");
        assert_eq!(unnamed.text, text("recipient"), "an unnamed address reads as its short form");

        let own_wallet = recipient_row(&transfer, Some(AddressName::mock("recipient", "Savings", AddressType::InternalWallet, VerificationStatus::Verified))).unwrap();
        assert_eq!(own_wallet.text, text("Savings"), "the user's own wallet reads as its name alone");
        assert_eq!(own_wallet.short_address, Some("recipient".to_string()), "its address shows on a tap");

        let resolved = recipient_row(&transfer, Some(AddressName::mock("recipient", "vitalik.eth", AddressType::Address, VerificationStatus::Verified))).unwrap();
        assert_eq!(resolved.text, text("vitalik.eth (recipient)"), "a name the user did not give keeps the address beside it");

        let validator = DelegationValidator::stake(Chain::HyperCore, "0x000000000056f99d36b6f2e0c51fd41496bbacb8".into(), "ValiDAO".into(), true, 0.0, 0.0);
        let unstake = GemTransferData::mock(TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::HyperCore),
            stake_type: StakeType::Unstake(Delegation::mock_with_validator(validator)),
        });
        assert_eq!(recipient_row(&unstake, None).map(|row| row.text), Some(text("ValiDAO")), "a validator reads as its name alone");

        let swap_data = SwapData::mock_with_provider(SwapProvider::PancakeswapV3);
        let provider_name = swap_data.quote.provider_data.name.clone();
        let swap = GemTransferData::mock(TransactionInputType::Swap {
            from_asset: Asset::mock_eth(),
            to_asset: Asset::mock_ethereum_usdc(),
            swap_data,
        });
        assert_eq!(recipient_row(&swap, None).map(|row| row.text), Some(text(&provider_name)), "a swap provider reads as its name alone");
    }

    #[test]
    fn test_a_known_contract_is_named_like_a_known_recipient() {
        use crate::services::transfer::model::GemConfirmDestination;
        let contract = GemConfirmDestination::Contract {
            name: None,
            address: "0xcontract".to_string(),
        };
        let named = contract.with_address_name(Some(AddressName::mock("0xcontract", "Uniswap", AddressType::Contract, VerificationStatus::Verified)));

        assert_eq!(named.name(), Some("Uniswap".to_string()));
        assert_eq!(contract.with_address_name(None).name(), None);
        assert_eq!(
            contract.with_address_name(Some(AddressName::mock("0xcontract", "", AddressType::Contract, VerificationStatus::Verified))).name(),
            None,
            "an empty name is no name"
        );
    }

    #[test]
    fn the_confirm_rows_carry_their_finished_content() {
        let link = |chain: Chain, address: String| BlockExplorerLink { name: chain.to_string(), link: address };
        let contents = confirm_row_contents(&GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }), Wallet::mock(), None, link);
        assert!(matches!(
            &contents[0],
            GemConfirmRowContent::Row { row: GemListRow::Wallet { menu, .. } } if matches!(menu.as_slice(), [GemRowMenuItem::Copy { copy }, GemRowMenuItem::Open { url, .. }] if copy.value == "address" && url == "address")
        ));
        assert!(matches!(&contents[1], GemConfirmRowContent::Recipient { row } if row.chain == Chain::Ethereum && matches!(row.menu.as_slice(), [_, GemRowMenuItem::Open { url, .. }] if url == "recipient")));
        assert!(matches!(
            &contents[2],
            GemConfirmRowContent::Row { row: GemListRow::Network { chain: Chain::Ethereum, name, .. } } if name == "Ethereum"
        ));

        let token = confirm_row_contents(&GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::mock_ethereum_usdc() }), Wallet::mock(), None, link);
        assert!(matches!(&token[2], GemConfirmRowContent::Row { row: GemListRow::Network { name, .. } } if name == "Ethereum (ERC20)"));

        let solana = GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Solana) });
        let without_memo = GemTransferData {
            recipient: GemRecipient { memo: None, ..solana.recipient.clone() },
            ..solana.clone()
        };
        let memo_row = |transfer: &GemTransferData| {
            confirm_row_contents(transfer, Wallet::mock(), None, link).into_iter().find_map(|content| match content {
                GemConfirmRowContent::Row { row: row @ GemListRow::Memo { .. } } => Some(row),
                _ => None,
            })
        };
        assert_eq!(
            memo_row(&solana),
            Some(GemListRow::Memo {
                title: GemListRowTitle::Memo,
                value: "memo".to_string(),
                menu: vec![GemRowMenuItem::Copy { copy: GemCopy::plain("memo".to_string()) }],
            })
        );
        assert_eq!(
            memo_row(&without_memo),
            Some(GemListRow::Memo {
                title: GemListRowTitle::Memo,
                value: "-".to_string(),
                menu: vec![]
            }),
            "a placeholder memo has nothing to copy"
        );

        let dapp = confirm_row_contents(
            &GemTransferData::mock(TransactionInputType::Generic {
                asset: Asset::from_chain(Chain::Ethereum),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock(),
            }),
            Wallet::mock(),
            None,
            link,
        );
        assert!(dapp.iter().any(|content| matches!(
            content,
            GemConfirmRowContent::Row { row: GemListRow::App { menu, .. } } if matches!(menu.as_slice(), [GemRowMenuItem::Open { url, .. }] if url == "https://example.com")
        )));
    }

    #[test]
    fn test_the_fee_asset_picker_shows_only_with_another_asset_to_pick() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let usdc = Asset::mock_ethereum_usdc().id;

        assert!(!shows_fee_assets(std::slice::from_ref(&ethereum), Some(&ethereum)));
        assert!(shows_fee_assets(&[ethereum.clone(), usdc], Some(&ethereum)));
        assert!(!shows_fee_assets(&[], Some(&ethereum)));
        assert!(shows_fee_assets(&[ethereum], None), "with nothing selected yet, another asset is still offered");
    }
}
