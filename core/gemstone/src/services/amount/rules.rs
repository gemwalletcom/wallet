use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, AutocloseEstimator, Chain, Currency, EarnType, StakeChain, TpslType};

use super::model::{
    GemAmountEarnType, GemAmountEntry, GemAmountError, GemAmountInput, GemAmountInputType, GemAmountMaxEntry, GemAmountPerpetualPosition, GemAmountStakeType, GemAmountTitle, GemAmountTransfer, GemAmountType, GemPerpetualAutoclose,
};
use crate::config::perpetual_config::{MIN_DEPOSIT_AMOUNT, MIN_WITHDRAW_AMOUNT};
use crate::config::stake::get_stake_config;
use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigInt;
use crate::perpetual::GemPerpetual;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::assets::icon::asset_icon;
use crate::services::balance::{GemAssetBalance, GemBalanceRequirement};
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::perpetual::rules::margin_amount_value;
use crate::services::stake::model::GemStakeAmountInput;
use crate::services::stake::rules as stake_rules;
use crate::services::transfer::rules as transfer_rules;
use crate::services::transfer::{GemRecipient, GemTransferData};
use gem_hypercore::perpetual_formatter::PerpetualFormatter;
use number_formatter::{BigNumberFormatter, CryptoFiatConverter};
use primitives::PerpetualProvider;
use primitives::TransactionInputType;

const USDC_SYMBOL: &str = "USDC";

#[uniffi::export]
impl GemAmountType {
    pub fn can_switch_input_type(&self) -> bool {
        matches!(self, Self::Transfer)
    }
}

impl GemAmountType {
    pub fn entry(&self, asset: &Asset, input: &GemAmountInput, price: Option<f64>, input_type: GemAmountInputType, text: String, currency: Currency) -> GemAmountEntry {
        let decimals = asset.decimals as u32;
        let (value, error) = match entry_value(&text, decimals, price, input_type) {
            Ok(Some(value)) => {
                let error = validate(asset, &value, &input.available_value, &minimum_value(self, asset)).err();
                (Some(value), error)
            }
            Ok(None) => (None, None),
            Err(error) => (None, Some(error)),
        };
        let is_max = value.as_ref() == Some(&input.max_value);
        GemAmountEntry {
            equivalent: equivalent(value.as_ref(), asset, price, input_type, currency),
            is_max,
            reserved_fee: input.reserved_fee.as_ref().filter(|_| is_max).map(|fee| GemLocalizedText::ReservedFees {
                fee: GemFormattedNumber::asset_amount(fee, asset, GemValueStyle::Auto),
            }),
            value,
            error,
        }
    }

    pub fn input(&self, asset: &Asset, balance: &GemAssetBalance) -> GemAmountInput {
        let available = self.available_value(asset, balance);
        let reserve = reserve_for_fee(self, asset);
        let max_after_fee = (&available - &reserve).max(BigInt::from(0));
        let reserved_fee = reserves_fee(self, &reserve, &max_after_fee, &minimum_value(self, asset)).then_some(reserve);
        let max_value = if reserved_fee.is_some() { max_after_fee } else { available.clone() };
        let can_change_value = can_change_value(self, asset);
        GemAmountInput {
            icon: asset_icon(&asset.id),
            balance: GemLocalizedText::AmountBalance {
                balance: GemFormattedNumber::asset_amount(&available, asset, GemValueStyle::Auto),
            },
            available_value: available,
            prefill: (!can_change_value).then(|| GemAmountMaxEntry {
                input_type: GemAmountInputType::Asset,
                value: max_value.clone(),
            }),
            max_value,
            reserved_fee,
            can_change_value,
            focuses_input: can_change_value,
            shows_asset_balance: shows_asset_balance(self, asset),
            uses_whole_amounts: uses_whole_amounts(self, asset),
        }
    }
}

impl GemAmountInput {
    pub fn max_entry(&self) -> GemAmountMaxEntry {
        GemAmountMaxEntry {
            input_type: GemAmountInputType::Asset,
            value: self.max_value.clone(),
        }
    }
}

fn uses_whole_amounts(amount_type: &GemAmountType, asset: &Asset) -> bool {
    let stakes_or_unstakes = matches!(
        amount_type,
        GemAmountType::Stake {
            stake_type: GemAmountStakeType::Stake | GemAmountStakeType::Unstake { .. }
        }
    );
    stakes_or_unstakes && stake_rules::uses_whole_amounts(asset.chain())
}

fn entry_value(text: &str, decimals: u32, price: Option<f64>, input_type: GemAmountInputType) -> Result<Option<BigInt>, GemAmountError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let amount = match input_type {
        GemAmountInputType::Asset => text.to_string(),
        GemAmountInputType::Fiat => {
            let price = valid_price(price).ok_or(GemAmountError::PriceMissing)?;
            CryptoFiatConverter::to_crypto_at_entry_precision(text, decimals, price).map_err(invalid_number)?
        }
    };
    let value = BigNumberFormatter::value_from_amount_truncated(&amount, decimals).map_err(invalid_number)?;
    BigInt::from_str(&value).map(Some).map_err(invalid_number)
}

fn invalid_number<E>(_: E) -> GemAmountError {
    GemAmountError::InvalidNumber
}

fn equivalent(value: Option<&BigInt>, asset: &Asset, price: Option<f64>, input_type: GemAmountInputType, currency: Currency) -> GemFormattedNumber {
    let value = value.cloned().unwrap_or_default();
    match input_type {
        GemAmountInputType::Asset => {
            let amount = valid_price(price)
                .and_then(|price| CryptoFiatConverter::to_fiat(&value.to_string(), asset.decimals as u32, price).ok())
                .and_then(|fiat| fiat.parse().ok())
                .unwrap_or(0.0);
            GemFormattedNumber::currency(amount, currency, GemCurrencyStyle::Currency)
        }
        GemAmountInputType::Fiat => GemFormattedNumber::asset_amount(&value, asset, GemValueStyle::Auto),
    }
}

fn valid_price(price: Option<f64>) -> Option<f64> {
    price.filter(|price| *price > 0.0)
}

pub fn perpetual_amount_type(action: &GemPerpetualPositionAction, leverage: u8) -> GemAmountType {
    let data = action.data();
    GemAmountType::Perpetual {
        position: match action {
            GemPerpetualPositionAction::Open { .. } => GemAmountPerpetualPosition::Open,
            GemPerpetualPositionAction::Increase { .. } => GemAmountPerpetualPosition::Increase,
            GemPerpetualPositionAction::Reduce { position, .. } => GemAmountPerpetualPosition::Reduce { available: margin_amount_value(position) },
        },
        direction: data.direction.clone(),
        price: data.price,
        leverage,
        size_decimals: data.asset.decimals,
    }
}

pub fn stake_amount_type(input: &GemStakeAmountInput) -> GemAmountType {
    let stake_type = match input {
        GemStakeAmountInput::Stake { .. } => GemAmountStakeType::Stake,
        GemStakeAmountInput::Unstake { delegation } => GemAmountStakeType::Unstake { delegation: delegation.clone() },
        GemStakeAmountInput::Redelegate { delegation, .. } => GemAmountStakeType::Redelegate { delegation: delegation.clone() },
        GemStakeAmountInput::Withdraw { delegation } => GemAmountStakeType::Withdraw { delegation: delegation.clone() },
        GemStakeAmountInput::Rewards { delegations, validator } => GemAmountStakeType::Rewards {
            delegations: delegations.iter().filter(|delegation| delegation.validator.id == validator.id).cloned().collect(),
        },
        GemStakeAmountInput::Freeze { resource } => GemAmountStakeType::Freeze { resource: *resource },
        GemStakeAmountInput::Unfreeze { resource } => GemAmountStakeType::Unfreeze { resource: *resource },
    };
    GemAmountType::Stake { stake_type }
}

pub fn earn_amount_type(earn_type: EarnType) -> GemAmountType {
    let (earn_type, validator) = match earn_type {
        EarnType::Deposit(validator) => (GemAmountEarnType::Deposit, validator),
        EarnType::Withdraw(delegation) => (GemAmountEarnType::Withdraw { delegation: delegation.clone() }, delegation.validator),
    };
    GemAmountType::Earn {
        earn_type,
        provider: stake_rules::validator_row(&validator),
    }
}

pub fn amount_title(amount_type: &GemAmountType) -> GemAmountTitle {
    match amount_type {
        GemAmountType::Transfer => GemAmountTitle::Send,
        GemAmountType::Deposit => GemAmountTitle::Deposit,
        GemAmountType::Withdraw => GemAmountTitle::Withdraw,
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake => GemAmountTitle::Stake,
            GemAmountStakeType::Unstake { .. } => GemAmountTitle::Unstake,
            GemAmountStakeType::Redelegate { .. } => GemAmountTitle::Redelegate,
            GemAmountStakeType::Withdraw { .. } => GemAmountTitle::Withdraw,
            GemAmountStakeType::Rewards { .. } => GemAmountTitle::Rewards,
            GemAmountStakeType::Freeze { .. } => GemAmountTitle::Freeze,
            GemAmountStakeType::Unfreeze { .. } => GemAmountTitle::Unfreeze,
        },
        GemAmountType::Earn { earn_type, .. } => match earn_type {
            GemAmountEarnType::Deposit => GemAmountTitle::Deposit,
            GemAmountEarnType::Withdraw { .. } => GemAmountTitle::Withdraw,
        },
        GemAmountType::Perpetual { position, direction, .. } => match position {
            GemAmountPerpetualPosition::Open => GemAmountTitle::PerpetualOpen { direction: direction.clone() },
            GemAmountPerpetualPosition::Increase => GemAmountTitle::PerpetualIncrease { direction: direction.clone() },
            GemAmountPerpetualPosition::Reduce { .. } => GemAmountTitle::PerpetualReduce { direction: direction.clone() },
        },
    }
}

pub fn transfer_amount_type(transfer: &GemAmountTransfer) -> GemAmountType {
    match transfer {
        GemAmountTransfer::Send { .. } => GemAmountType::Transfer,
        GemAmountTransfer::Deposit => GemAmountType::Deposit,
        GemAmountTransfer::Withdraw => GemAmountType::Withdraw,
    }
}

pub fn transfer_display_asset(transfer: &GemAmountTransfer, asset: Asset) -> Asset {
    match transfer {
        GemAmountTransfer::Withdraw => GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset(),
        GemAmountTransfer::Send { .. } | GemAmountTransfer::Deposit => asset,
    }
}

pub fn transfer_input(transfer: &GemAmountTransfer, asset: &Asset, balance: &GemAssetBalance) -> GemAmountInput {
    let input = transfer_amount_type(transfer).input(asset, balance);
    let requested = match transfer {
        GemAmountTransfer::Send { payment } => payment.amount.as_deref().and_then(|amount| BigNumberFormatter::value_from_amount(amount, asset.decimals as u32).ok()),
        GemAmountTransfer::Deposit | GemAmountTransfer::Withdraw => None,
    };
    let prefill = requested.and_then(|value| GemBigInt::from_str(&value).ok()).map(|value| GemAmountMaxEntry {
        input_type: GemAmountInputType::Asset,
        value,
    });
    GemAmountInput {
        icon: asset_icon(&transfer_display_asset(transfer, asset.clone()).id),
        prefill: prefill.or(input.prefill.clone()),
        ..input
    }
}

pub fn transfer_data(asset: Asset, transfer: GemAmountTransfer, owner: Option<GemRecipient>, value: GemBigInt, use_max_amount: bool) -> Result<GemTransferData, GemServiceError> {
    let (input_type, recipient) = match transfer {
        GemAmountTransfer::Send { payment } => (TransactionInputType::Transfer { asset }, payment.recipient),
        GemAmountTransfer::Deposit => (TransactionInputType::Deposit { asset }, GemPerpetual::new(PerpetualProvider::Hypercore).deposit_recipient()),
        GemAmountTransfer::Withdraw => {
            let owner = owner.ok_or_else(|| GemServiceError::NotFound {
                msg: format!("no {} account to withdraw to", asset.chain()),
            })?;
            (TransactionInputType::Withdrawal { asset }, owner)
        }
    };
    Ok(GemTransferData {
        input_type,
        recipient,
        value,
        use_max_amount,
    })
}

impl GemAmountType {
    fn available_value(&self, asset: &Asset, balance: &GemAssetBalance) -> BigInt {
        match self {
            Self::Transfer | Self::Deposit => BigInt::from(balance.available.clone()),
            Self::Withdraw => BigInt::from(balance.withdrawable.clone()),
            Self::Stake { stake_type } => match stake_type {
                GemAmountStakeType::Stake if asset.chain() == Chain::Tron => transfer_rules::tron_stake_available(asset, balance),
                GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => BigInt::from(balance.available.clone()),
                GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => BigInt::from(delegation.base.balance.clone()),
                GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
                GemAmountStakeType::Unfreeze { resource } => transfer_rules::unfreeze_available(resource, balance),
            },
            Self::Earn { earn_type, .. } => match earn_type {
                GemAmountEarnType::Deposit => BigInt::from(balance.available.clone()),
                GemAmountEarnType::Withdraw { delegation } => BigInt::from(delegation.base.balance.clone()),
            },
            Self::Perpetual { position, .. } => match position {
                GemAmountPerpetualPosition::Open | GemAmountPerpetualPosition::Increase => BigInt::from(balance.available.clone()),
                GemAmountPerpetualPosition::Reduce { available } => BigInt::from(available.clone()),
            },
        }
    }
}

pub fn perpetual_autoclose(action: &GemPerpetualPositionAction, leverage: u8, take_profit_percent: u8, stop_loss_percent: u8, decimal_separator: &str) -> GemPerpetualAutoclose {
    if !action.shows_autoclose() {
        return GemPerpetualAutoclose { take_profit: None, stop_loss: None };
    }
    let data = action.data();
    let estimator = AutocloseEstimator::for_open(data.price, 0.0, leverage, data.direction.clone());
    let perpetual = GemPerpetual::new(data.provider.clone());
    let target = |percent: u8, trigger_type: TpslType| (percent > 0).then(|| perpetual.format_input_price(estimator.target_price_from_roe(i32::from(percent), trigger_type), data.asset.decimals, decimal_separator.to_string()));
    GemPerpetualAutoclose {
        take_profit: target(take_profit_percent, TpslType::TakeProfit),
        stop_loss: target(stop_loss_percent, TpslType::StopLoss),
    }
}

pub fn validate(asset: &Asset, value: &BigInt, available: &BigInt, minimum: &BigInt) -> Result<(), GemAmountError> {
    if value <= &BigInt::from(0) {
        return Err(GemAmountError::Zero);
    }
    if value < minimum {
        return Err(GemAmountError::BelowMinimum {
            asset: asset.clone(),
            minimum: minimum.clone(),
        });
    }
    if value > available {
        return Err(GemAmountError::InsufficientBalance {
            asset: asset.clone(),
            requirement: GemBalanceRequirement::new(value.clone(), available.clone()),
        });
    }
    Ok(())
}

fn minimum_value(amount_type: &GemAmountType, asset: &Asset) -> BigInt {
    let stake_config = stake_chain(asset.chain()).map(get_stake_config);
    match amount_type {
        GemAmountType::Transfer | GemAmountType::Earn { .. } => BigInt::from(0),
        GemAmountType::Deposit => usdc_minimum(asset, MIN_DEPOSIT_AMOUNT),
        GemAmountType::Withdraw => usdc_minimum(asset, MIN_WITHDRAW_AMOUNT),
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => stake_config.map(|config| BigInt::from(config.min_amount)).unwrap_or_default(),
            GemAmountStakeType::Redelegate { .. } if asset.chain() == Chain::SmartChain => stake_config.map(|config| BigInt::from(config.min_amount)).unwrap_or_default(),
            GemAmountStakeType::Withdraw { .. } | GemAmountStakeType::Redelegate { .. } | GemAmountStakeType::Unstake { .. } | GemAmountStakeType::Unfreeze { .. } | GemAmountStakeType::Rewards { .. } => BigInt::from(0),
        },
        GemAmountType::Perpetual {
            position, price, leverage, size_decimals, ..
        } => {
            let minimum = BigInt::from(PerpetualFormatter::minimum_order_usd_amount(*price, *size_decimals, *leverage));
            match position {
                GemAmountPerpetualPosition::Open | GemAmountPerpetualPosition::Increase => minimum,
                GemAmountPerpetualPosition::Reduce { available } => minimum.min(BigInt::from(available.clone())),
            }
        }
    }
}

fn reserve_for_fee(amount_type: &GemAmountType, asset: &Asset) -> BigInt {
    let reserved = stake_chain(asset.chain()).map(|chain| BigInt::from(get_stake_config(chain).reserved_for_fees)).unwrap_or_default();
    match amount_type {
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake if asset.chain() != Chain::Tron => reserved,
            GemAmountStakeType::Freeze { .. } => reserved,
            _ => BigInt::from(0),
        },
        _ => BigInt::from(0),
    }
}

fn can_change_value(amount_type: &GemAmountType, asset: &Asset) -> bool {
    match amount_type {
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Unstake { .. } => stake_chain(asset.chain()).map(|chain| get_stake_config(chain).change_amount_on_unstake).unwrap_or(true),
            GemAmountStakeType::Withdraw { .. } | GemAmountStakeType::Rewards { .. } => false,
            GemAmountStakeType::Stake | GemAmountStakeType::Redelegate { .. } | GemAmountStakeType::Freeze { .. } | GemAmountStakeType::Unfreeze { .. } => true,
        },
        _ => true,
    }
}

fn shows_asset_balance(amount_type: &GemAmountType, asset: &Asset) -> bool {
    match amount_type {
        GemAmountType::Stake {
            stake_type: GemAmountStakeType::Rewards { .. },
        } => true,
        _ => can_change_value(amount_type, asset),
    }
}

fn reserves_fee(amount_type: &GemAmountType, reserve: &BigInt, max_after_fee: &BigInt, minimum: &BigInt) -> bool {
    match amount_type {
        GemAmountType::Stake {
            stake_type: GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. },
        } => reserve > &BigInt::from(0) && max_after_fee > minimum,
        _ => false,
    }
}

fn usdc_minimum(asset: &Asset, minimum: u64) -> BigInt {
    if asset.symbol == USDC_SYMBOL { BigInt::from(minimum) } else { BigInt::from(0) }
}

fn stake_chain(chain: Chain) -> Option<StakeChain> {
    StakeChain::from_str(chain.as_ref()).ok()
}

const SEPARATORS: [char; 2] = ['.', ','];
const GROUPING_SYMBOLS: [char; 5] = [' ', '\'', '\u{2019}', '\u{202F}', '\u{00A0}'];
const ARABIC_DECIMAL: char = '\u{066B}';
const DIGIT_ZEROS: [u32; 76] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50,
    0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10, 0x104A0, 0x10D30, 0x10D40, 0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650, 0x116C0, 0x116D0, 0x116DA, 0x11730, 0x118E0, 0x11950, 0x11BF0, 0x11C50,
    0x11D50, 0x11DA0, 0x11F50, 0x16130, 0x16A60, 0x16AC0, 0x16B50, 0x16D70, 0x1CCF0, 0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E4F0, 0x1E5F1, 0x1E950, 0x1FBF0,
];
const ARABIC_GROUPING: char = '\u{066C}';

pub fn sanitize_number_input(decimal_separator: &str, text: &str, maximum_fraction_digits: Option<u32>, maximum_integer_digits: Option<u32>) -> String {
    let is_separator = |character: &char| SEPARATORS.contains(character) || decimal_separator.contains(*character);
    let allows_fraction = maximum_fraction_digits != Some(0);
    let typed: String = text.chars().filter(|character| character.is_numeric() || allows_fraction && is_separator(character)).collect();
    let limit = |value: &str, maximum: Option<u32>| match maximum {
        Some(maximum) => value.chars().take(maximum as usize).collect::<String>(),
        None => value.to_string(),
    };
    match typed.chars().position(|character| is_separator(&character)) {
        None => limit(&typed, maximum_integer_digits),
        Some(position) => {
            let integer: String = typed.chars().take(position).collect();
            let fraction: String = typed.chars().skip(position + 1).filter(|character| !is_separator(character)).collect();
            format!("{}{}{}", limit(&integer, maximum_integer_digits), decimal_separator, limit(&fraction, maximum_fraction_digits))
        }
    }
}

pub fn value_from_input(decimal_separator: &str, text: &str, decimals: u32) -> Result<BigInt, GemAmountError> {
    let plain = plain_number(decimal_separator, text);
    let value = BigNumberFormatter::value_from_amount_truncated(&plain, decimals).map_err(invalid_number)?;

    BigInt::from_str(&value).map_err(invalid_number)
}

pub fn input_text(decimal_separator: &str, value: &str, decimals: u32) -> Option<String> {
    let plain = BigNumberFormatter::big_decimal_value(value, decimals).ok()?.normalized().to_plain_string();
    Some(plain.replace('.', decimal_separator))
}

pub fn value_text(decimal_separator: &str, value: f64) -> String {
    value.to_string().replace('.', decimal_separator)
}

pub fn plain_number(decimal_separator: &str, text: &str) -> String {
    let mut trimmed = latin_digits(text).trim().to_string();
    while let Some(last) = trimmed.chars().last() {
        if last.is_ascii_digit() || SEPARATORS.contains(&last) || GROUPING_SYMBOLS.contains(&last) {
            break;
        }
        trimmed.pop();
    }
    let without_grouping: String = trimmed.chars().filter(|character| !GROUPING_SYMBOLS.contains(character)).collect();
    if without_grouping.is_empty() {
        return String::new();
    }
    without_leading_zeros(&standard_decimal(decimal_separator, &without_grouping))
}

fn latin_digits(text: &str) -> String {
    text.chars()
        .filter(|character| *character != ARABIC_GROUPING)
        .map(|character| match character {
            ARABIC_DECIMAL => '.',
            _ => latin_digit(character).unwrap_or(character),
        })
        .collect()
}

fn latin_digit(character: char) -> Option<char> {
    let code = character as u32;
    let index = DIGIT_ZEROS.partition_point(|zero| *zero <= code);
    let digit = code - DIGIT_ZEROS.get(index.checked_sub(1)?)?;

    (digit <= 9).then(|| char::from_digit(digit, 10)).flatten()
}

fn standard_decimal(decimal_separator: &str, text: &str) -> String {
    let dots = text.matches('.').count();
    let commas = text.matches(',').count();
    match (dots > 0, commas > 0) {
        (true, true) if decimal_separator == "." => keep_last(&text.replace(',', ""), '.'),
        (true, true) => keep_last(&text.replace('.', ""), ',').replace(',', "."),
        (true, false) if dots > 1 || (decimal_separator == "," && is_grouping_dot(text)) => text.replace('.', ""),
        (true, false) => keep_last(text, '.'),
        (false, true) if commas > 1 || decimal_separator != "," => text.replace(',', ""),
        (false, true) => text.replace(',', "."),
        (false, false) => text.to_string(),
    }
}

fn is_grouping_dot(text: &str) -> bool {
    match text.split_once('.') {
        Some((before, after)) => !before.is_empty() && before.chars().all(|character| character.is_ascii_digit()) && after.len() == 3 && after.chars().all(|character| character.is_ascii_digit()),
        None => false,
    }
}

fn keep_last(text: &str, symbol: char) -> String {
    match text.rfind(symbol) {
        Some(position) => format!("{}{}", text[..position].replace(symbol, ""), &text[position..]),
        None => text.to_string(),
    }
}

fn without_leading_zeros(text: &str) -> String {
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text, None),
    };
    let trimmed = integer.trim_start_matches('0');
    let integer = if trimmed.is_empty() { "0" } else { trimmed };
    match fraction {
        Some(fraction) => format!("{integer}.{fraction}"),
        None => integer.to_string(),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_a_request_answers_its_own_amount_type_and_display_asset() {
        use super::super::model::{GemAmountRequest, GemAmountStakeType, GemAmountTransfer, GemAmountType};
        use crate::services::stake::model::GemStakeAmountInput;
        use primitives::{Asset, Chain, DelegationValidator};

        let usdc = Asset::from_chain(Chain::Arbitrum);
        let deposit = GemAmountRequest::Transfer { transfer: GemAmountTransfer::Deposit };
        assert_eq!(deposit.amount_type(), GemAmountType::Deposit);
        assert_eq!(deposit.display_asset(usdc.clone()), super::transfer_display_asset(&GemAmountTransfer::Deposit, usdc.clone()));

        let stake = GemAmountRequest::Stake {
            input: GemStakeAmountInput::Stake { validator: DelegationValidator::mock() },
        };
        assert_eq!(stake.amount_type(), GemAmountType::Stake { stake_type: GemAmountStakeType::Stake });
        assert_eq!(stake.display_asset(usdc.clone()), usdc, "only a transfer shows another asset");
    }

    #[test]
    fn test_a_value_echoes_into_the_field_without_grouping_or_trailing_zeros() {
        assert_eq!(super::value_text(".", 67000.0), "67000");
        assert_eq!(super::value_text(",", 1234.5), "1234,5");
        assert_eq!(super::value_text(".", 0.000001), "0.000001");
    }

    #[test]
    fn test_the_amount_screen_title_follows_the_amount_type() {
        assert_eq!(amount_title(&GemAmountType::Transfer), GemAmountTitle::Send);
        assert_eq!(
            amount_title(&earn_amount_type(EarnType::Withdraw(primitives::Delegation::mock()))),
            GemAmountTitle::Withdraw,
            "earning and staking withdrawals share the wallet's withdraw title"
        );
        assert_eq!(
            amount_title(&GemAmountType::Stake {
                stake_type: GemAmountStakeType::Rewards { delegations: vec![] }
            }),
            GemAmountTitle::Rewards
        );
        assert_eq!(
            amount_title(&GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Increase,
                direction: primitives::PerpetualDirection::Short,
                price: 1.0,
                leverage: 1,
                size_decimals: 2,
            }),
            GemAmountTitle::PerpetualIncrease {
                direction: primitives::PerpetualDirection::Short
            }
        );
    }

    #[test]
    fn test_an_amount_confirms_only_when_it_is_positive_and_has_no_error() {
        let entry = GemAmountEntry {
            value: Some(BigInt::from(1)),
            error: None,
            equivalent: GemFormattedNumber::asset_amount(&BigInt::from(1), &Asset::mock_hypercore_usdc(), GemValueStyle::Auto),
            is_max: false,
            reserved_fee: None,
        };

        assert!(entry.allows_confirm());
        assert!(!GemAmountEntry { value: None, ..entry.clone() }.allows_confirm(), "nothing typed is nothing to send");
        assert!(!GemAmountEntry { value: Some(BigInt::ZERO), ..entry.clone() }.allows_confirm(), "zero is not an amount");
        assert!(!GemAmountEntry { error: Some(GemAmountError::Zero), ..entry }.allows_confirm());
    }
    use super::*;
    use crate::formatted_number::GemNumberUnit;
    use crate::models::custom_types::GemBigUint;
    use crate::payment::GemPaymentRecipient;
    use crate::services::perpetual::GemPerpetualTransferData;
    use primitives::Resource;
    use primitives::asset_balance::BalanceMetadata;
    use primitives::contract_constants::HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS;
    use primitives::{Delegation, DelegationBase, DelegationValidator, PerpetualDirection};

    #[test]
    fn test_input_text_is_plain_digits_with_the_callers_separator() {
        assert_eq!(input_text(",", "9649000000000000", 18), Some("0,009649".to_string()));
        assert_eq!(input_text(".", "123456789012", 4), Some("12345678.9012".to_string()));
        assert_eq!(input_text(".", "100000", 3), Some("100".to_string()));
        assert_eq!(input_text(".", "1", 18), Some("0.000000000000000001".to_string()));
        assert_eq!(input_text(".", "0", 8), Some("0".to_string()));
        assert_eq!(input_text(".", "abc", 8), None);
    }

    #[test]
    fn test_sanitize_number_input_keeps_digits_and_the_first_separator() {
        assert_eq!(sanitize_number_input(".", "abc123.45xyz", None, None), "123.45");
        assert_eq!(sanitize_number_input(".", "12.", Some(0), None), "12");
        assert_eq!(sanitize_number_input(".", ".50", Some(0), None), "50");
        assert_eq!(sanitize_number_input(".", "123.45.67", None, None), "123.4567");
        assert_eq!(sanitize_number_input(".", " 1 000 ", None, None), "1000");
        assert_eq!(sanitize_number_input(".", "12", None, None), "12");
        assert_eq!(sanitize_number_input(".", ".", None, None), ".");
        assert_eq!(sanitize_number_input(".", "٣.٥", None, None), "٣.٥");
    }

    #[test]
    fn test_sanitize_number_input_answers_in_the_callers_separator() {
        assert_eq!(sanitize_number_input(",", "1.5", None, None), "1,5");
        assert_eq!(sanitize_number_input(".", "1,5", None, None), "1.5");
        assert_eq!(sanitize_number_input("٫", "٣٫٥", None, None), "٣٫٥");
    }

    #[test]
    fn test_sanitize_number_input_limits_each_part() {
        assert_eq!(sanitize_number_input(".", "0.111111", Some(2), None), "0.11");
        assert_eq!(sanitize_number_input(".", "12.5", Some(2), None), "12.5");
        assert_eq!(sanitize_number_input(".", "12", Some(2), None), "12");
        assert_eq!(sanitize_number_input(".", "33333312312", None, Some(2)), "33");
        assert_eq!(sanitize_number_input(".", "19.555", None, Some(2)), "19.555");
    }

    #[test]
    fn test_plain_number_strips_grouping_for_each_locale() {
        let cases = [
            (".", "1,234.56", "1234.56"),
            (".", " 1,234.56 ", "1234.56"),
            (".", "1,234.56$", "1234.56"),
            (".", "1,234.56 USD", "1234.56"),
            (".", "123,456.78BTC", "123456.78"),
            (".", "12,345,678,901.23456789", "12345678901.23456789"),
            (",", "1.234,56", "1234.56"),
            (",", " 1.234,56 ", "1234.56"),
            (",", "1.234,56 kr", "1234.56"),
            (",", "12.345.678.901,23456789", "12345678901.23456789"),
            (".", "1'234.56", "1234.56"),
            (",", "1 234,56 \u{20ac}", "1234.56"),
            (",", "1\u{202f}234,56", "1234.56"),
            (".", "1,234.56\u{5143}", "1234.56"),
        ];

        for (separator, input, expected) in cases {
            assert_eq!(plain_number(separator, input), expected, "{input} in {separator}");
        }
    }

    #[test]
    fn test_plain_number_reads_arabic_numerals() {
        let cases = [
            ("\u{66b}", "\u{661}\u{66c}\u{662}\u{663}\u{664}\u{66b}\u{665}\u{666}", "1234.56"),
            ("\u{66b}", " \u{661}\u{66c}\u{662}\u{663}\u{664}\u{66b}\u{665}\u{666} ", "1234.56"),
            ("\u{66b}", "\u{661}\u{66c}\u{662}\u{663}\u{664}\u{66b}\u{665}\u{666} SAR", "1234.56"),
        ];

        for (separator, input, expected) in cases {
            assert_eq!(plain_number(separator, input), expected);
        }
    }

    #[test]
    fn test_plain_number_reads_every_script_that_has_digits() {
        let cases = [
            ("\u{6f1}\u{6f2}\u{6f3}.\u{6f4}\u{6f5}", "123.45"),
            ("\u{967}\u{968}\u{969}.\u{96a}\u{96b}", "123.45"),
            ("\u{9e7}\u{9e8}\u{9e9}.\u{9ea}\u{9eb}", "123.45"),
            ("\u{e51}\u{e52}\u{e53}.\u{e54}\u{e55}", "123.45"),
            ("\u{17e1}\u{17e2}\u{17e3}.\u{17e4}\u{17e5}", "123.45"),
            ("\u{ff11}\u{ff12}\u{ff13}.\u{ff14}\u{ff15}", "123.45"),
            ("\u{1d7d9}\u{1d7da}\u{1d7db}.\u{1d7dc}\u{1d7dd}", "123.45"),
        ];

        for (input, expected) in cases {
            assert_eq!(plain_number(".", input), expected);
        }
    }

    #[test]
    fn test_value_from_input_scales_by_decimals() {
        assert_eq!(value_from_input(".", "1,234.56", 2).unwrap(), BigInt::from(123_456));
        assert_eq!(value_from_input(",", "1.234,56", 6).unwrap(), BigInt::from(1_234_560_000u64));
        assert_eq!(value_from_input(".", "0.1234567", 4).unwrap(), BigInt::from(1234));
        assert_eq!(value_from_input(".", "\u{661}\u{662}\u{663}\u{66b}\u{665}", 2).unwrap(), BigInt::from(12_350));
        assert!(value_from_input(".", "abc", 8).is_err());
        assert!(value_from_input(".", "-5", 8).is_err());
    }

    #[test]
    fn test_plain_number_tells_a_grouping_mark_from_a_decimal_point() {
        let cases = [
            (".", "1.234", "1.234"),
            (",", "1.234", "1234"),
            (".", "1.000.000", "1000000"),
            (",", "1,000,000", "1000000"),
            (".", "1,000,000", "1000000"),
            (",", "1.000.000", "1000000"),
            (",", "1,5", "1.5"),
            (".", "1.5", "1.5"),
            (".", "1,234", "1234"),
            (".", "1,234 567.89", "1234567.89"),
            (",", "1 234 567,89 \u{20ac}", "1234567.89"),
        ];

        for (separator, input, expected) in cases {
            assert_eq!(plain_number(separator, input), expected, "{input} in {separator}");
        }
    }

    #[test]
    fn test_plain_number_answers_nothing_when_no_number_was_typed() {
        for input in ["", "   ", "non-digit"] {
            assert_eq!(plain_number(".", input), "");
        }
    }

    #[test]
    fn test_plain_number_drops_leading_zeros_and_trailing_symbols() {
        let cases = [
            (".", "0.12317", "0.12317"),
            (".", "00.12317", "0.12317"),
            (".", "0001234.56", "1234.56"),
            (",", "0,12317", "0.12317"),
            (".", "123,456.78!!!!", "123456.78"),
            (".", "1,2 34.5'6", "1234.56"),
            (".", "1234.56", "1234.56"),
            (",", "1234.56", "1234.56"),
        ];

        for (separator, input, expected) in cases {
            assert_eq!(plain_number(separator, input), expected, "{input} in {separator}");
        }
    }

    #[test]
    fn test_stake_rules_reserve_fees_and_minimums() {
        let cosmos = Asset::from_chain(Chain::Cosmos);
        let config = get_stake_config(StakeChain::Cosmos);
        assert_eq!(minimum_value(&GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }, &cosmos), BigInt::from(config.min_amount));

        let stake_input = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&cosmos, &GemAssetBalance::mock_with_available(config.reserved_for_fees * 10 + config.min_amount * 10));
        assert_eq!(stake_input.reserved_fee, Some(BigInt::from(config.reserved_for_fees)));
        assert!(stake_input.can_change_value);
        assert!(stake_input.focuses_input);
        assert_eq!(stake_input.prefill, None);
        assert_eq!(stake_input.max_value, BigInt::from(config.reserved_for_fees * 9 + config.min_amount * 10));

        let tron = Asset::from_chain(Chain::Tron);
        let tron_config = get_stake_config(StakeChain::Tron);
        assert_eq!(reserve_for_fee(&GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }, &tron), BigInt::ZERO);
        assert_eq!(
            GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.available_value(
                &tron,
                &GemAssetBalance {
                    frozen: BigUint::from(5000000u64),
                    locked: BigUint::from(3000000u64),
                    metadata: Some(BalanceMetadata { votes: 2, ..BalanceMetadata::default() }),
                    ..GemAssetBalance::mock_with_available(1)
                }
            ),
            BigInt::from(6_000_000)
        );
        let freeze = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Freeze { resource: Resource::Bandwidth },
        };
        assert!(tron_config.reserved_for_fees > 0);
        let freeze_input = freeze.input(
            &tron,
            &GemAssetBalance {
                frozen: BigUint::from(99u64),
                locked: BigUint::from(98u64),
                ..GemAssetBalance::mock_with_available(tron_config.reserved_for_fees + tron_config.min_amount + 1)
            },
        );
        assert_eq!(freeze_input.available_value, BigInt::from(tron_config.reserved_for_fees + tron_config.min_amount + 1));
        assert_eq!(freeze_input.reserved_fee, Some(BigInt::from(tron_config.reserved_for_fees)));
        assert_eq!(freeze_input.max_value, BigInt::from(tron_config.min_amount + 1));

        let smart_chain = Asset::from_chain(Chain::SmartChain);
        let redelegate = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Redelegate {
                delegation: Delegation::mock_base(DelegationBase::mock_with_balance(50, 0)),
            },
        };
        let smart_chain_minimum = get_stake_config(StakeChain::SmartChain).min_amount;
        assert!(smart_chain_minimum > 0);
        assert_eq!(minimum_value(&redelegate, &smart_chain), BigInt::from(smart_chain_minimum));
        assert_eq!(minimum_value(&redelegate, &cosmos), BigInt::ZERO);

        let unstake = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Unstake {
                delegation: Delegation::mock_base(DelegationBase::mock_with_balance(50, 0)),
            },
        };
        let solana_unstake = unstake.input(&Asset::from_chain(Chain::Solana), &GemAssetBalance::mock_with_available(1));
        assert!(!solana_unstake.can_change_value);
        assert!(!solana_unstake.shows_asset_balance);
        assert!(!solana_unstake.focuses_input, "a fixed amount takes no typing");
        assert_eq!(solana_unstake.prefill, Some(solana_unstake.max_entry()), "a fixed amount fills itself");
        let cosmos_unstake = unstake.input(&cosmos, &GemAssetBalance::mock_with_available(1));
        assert!(cosmos_unstake.can_change_value);
        assert!(cosmos_unstake.shows_asset_balance);

        let rewards = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Rewards { delegations: vec![] },
        }
        .input(&cosmos, &GemAssetBalance::mock_with_available(1));
        assert!(!rewards.can_change_value);
        assert!(rewards.shows_asset_balance);
        assert_eq!(
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Rewards {
                    delegations: vec![Delegation::mock_base(DelegationBase::mock_with_balance(10, 3)), Delegation::mock_base(DelegationBase::mock_with_balance(20, 4))]
                }
            }
            .available_value(&cosmos, &GemAssetBalance::mock_with_available(1)),
            BigInt::from(7)
        );
        assert_eq!(
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Unstake {
                    delegation: Delegation::mock_base(DelegationBase::mock_with_balance(50, 0))
                }
            }
            .available_value(&cosmos, &GemAssetBalance::mock_with_available(1)),
            BigInt::from(50)
        );
        assert_eq!(
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Unfreeze { resource: Resource::Energy }
            }
            .available_value(
                &tron,
                &GemAssetBalance {
                    frozen: BigUint::from(2u64),
                    locked: BigUint::from(3u64),
                    ..GemAssetBalance::mock_with_available(1)
                }
            ),
            BigInt::from(3)
        );
        assert_eq!(
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Unfreeze { resource: Resource::Bandwidth }
            }
            .available_value(
                &tron,
                &GemAssetBalance {
                    frozen: BigUint::from(2u64),
                    locked: BigUint::from(3u64),
                    ..GemAssetBalance::mock_with_available(1)
                }
            ),
            BigInt::from(2)
        );
    }

    #[test]
    fn test_only_a_stake_or_unstake_on_a_whole_unit_chain_uses_whole_amounts() {
        let tron = Asset::from_chain(Chain::Tron);
        let funded = GemAssetBalance::mock_with_available(10_000_000);
        assert!(GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&tron, &funded).uses_whole_amounts);
        assert!(
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Unstake {
                    delegation: Delegation::mock_base(DelegationBase::mock_with_balance(50, 0))
                }
            }
            .input(&tron, &funded)
            .uses_whole_amounts
        );
        assert!(
            !GemAmountType::Stake {
                stake_type: GemAmountStakeType::Rewards { delegations: vec![] }
            }
            .input(&tron, &funded)
            .uses_whole_amounts
        );
        assert!(!GemAmountType::Transfer.input(&tron, &funded).uses_whole_amounts);
        assert!(!GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&Asset::from_chain(Chain::Cosmos), &funded).uses_whole_amounts);
    }

    #[test]
    fn test_limits_reserve_boundary() {
        let solana = Asset::from_chain(Chain::Solana);
        let config = get_stake_config(StakeChain::Solana);
        let reserve = config.reserved_for_fees;
        let minimum = config.min_amount;
        assert!(reserve > 0 && minimum > 0);

        let at_boundary = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&solana, &GemAssetBalance::mock_with_available(reserve + minimum));
        assert_eq!(at_boundary.reserved_fee, None);
        assert_eq!(at_boundary.max_value, BigInt::from(reserve + minimum));

        let above_boundary = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&solana, &GemAssetBalance::mock_with_available(reserve + minimum + 1));
        assert_eq!(above_boundary.reserved_fee, Some(BigInt::from(reserve)));
        assert_eq!(above_boundary.max_value, BigInt::from(minimum + 1));

        let below_reserve = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.input(&solana, &GemAssetBalance::mock_with_available(reserve - 1));
        assert_eq!(below_reserve.reserved_fee, None);
        assert_eq!(below_reserve.max_value, BigInt::from(reserve - 1));
        assert_eq!(below_reserve.available_value, BigInt::from(reserve - 1));
    }

    #[test]
    fn test_earn_perpetual_and_deposit_sources() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        assert_eq!(
            earn_amount_type(EarnType::Deposit(DelegationValidator::mock())).available_value(&ethereum, &GemAssetBalance::mock_with_available(11)),
            BigInt::from(11)
        );
        assert_eq!(
            earn_amount_type(EarnType::Withdraw(Delegation::mock_base(DelegationBase::mock_with_balance(33, 0)))).available_value(&ethereum, &GemAssetBalance::mock_with_available(11)),
            BigInt::from(33)
        );
        assert_eq!(GemAmountType::Deposit.available_value(&Asset::mock_hypercore_usdc(), &GemAssetBalance::mock_with_available(5)), BigInt::from(5));
        assert_eq!(
            GemAmountType::Withdraw.available_value(
                &Asset::mock_hypercore_usdc(),
                &GemAssetBalance {
                    withdrawable: BigUint::from(7u32),
                    ..GemAssetBalance::mock_with_available(5)
                }
            ),
            BigInt::from(7)
        );

        assert_eq!(
            minimum_value(
                &GemAmountType::Perpetual {
                    position: GemAmountPerpetualPosition::Open,
                    direction: PerpetualDirection::Long,
                    price: 4.0,
                    leverage: 1,
                    size_decimals: 0,
                },
                &Asset::mock_hypercore_usdc()
            ),
            BigInt::from(12_000_000)
        );
        assert_eq!(
            minimum_value(
                &GemAmountType::Perpetual {
                    position: GemAmountPerpetualPosition::Open,
                    direction: PerpetualDirection::Long,
                    price: 4.0,
                    leverage: 1,
                    size_decimals: 1,
                },
                &Asset::mock_hypercore_usdc()
            ),
            BigInt::from(10_000_000)
        );
        assert_eq!(
            minimum_value(
                &GemAmountType::Perpetual {
                    position: GemAmountPerpetualPosition::Open,
                    direction: PerpetualDirection::Long,
                    price: 4.0,
                    leverage: 2,
                    size_decimals: 0,
                },
                &Asset::mock_hypercore_usdc()
            ),
            BigInt::from(6_000_000)
        );
        assert_eq!(
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Open,
                direction: PerpetualDirection::Long,
                price: 4.0,
                leverage: 1,
                size_decimals: 0,
            }
            .available_value(&Asset::mock_hypercore_usdc(), &GemAssetBalance::mock_with_available(9)),
            BigInt::from(9)
        );
    }

    #[test]
    fn test_entry_reads_the_text_in_the_input_units() {
        let usdc = Asset::mock_hypercore_usdc();
        let funded = GemAmountType::Transfer.input(&usdc, &GemAssetBalance::mock_with_available(100_000_000_000));
        let hundred = GemAmountType::Transfer.input(&usdc, &GemAssetBalance::mock_with_available(100_000_000));
        let typed = GemAmountType::Transfer.entry(&usdc, &funded, Some(2.0), GemAmountInputType::Asset, "1000.123456".to_string(), Currency::USD);
        assert_eq!(typed.value, Some(BigInt::from(1_000_123_456)));
        assert_eq!(typed.error, None);
        assert_eq!(typed.equivalent, GemFormattedNumber::currency(2000.246912, Currency::USD, GemCurrencyStyle::Currency));
        assert!(!typed.is_max);
        assert_eq!(
            GemAmountType::Transfer.entry(&usdc, &hundred, None, GemAmountInputType::Asset, "1.5".to_string(), Currency::USD).equivalent,
            GemFormattedNumber::currency(0.0, Currency::USD, GemCurrencyStyle::Currency),
            "without a price the screen still shows a zero equivalent rather than nothing"
        );

        let fiat = GemAmountType::Transfer.entry(&usdc, &funded, Some(1.0), GemAmountInputType::Fiat, "1000.123456".to_string(), Currency::USD);
        assert_eq!(fiat.value, Some(BigInt::from(1_000_120_000)));
        assert_eq!(
            fiat.equivalent,
            GemFormattedNumber::asset_amount(&BigInt::from(1_000_120_000), &usdc, GemValueStyle::Auto),
            "the asset equivalent arrives formatted, so neither app picks a style for it"
        );
        assert_eq!(
            GemAmountType::Transfer.entry(&usdc, &funded, Some(1.0), GemAmountInputType::Fiat, "1000".to_string(), Currency::USD).value,
            Some(BigInt::from(1_000_000_000))
        );
        assert_eq!(
            GemAmountType::Transfer.entry(&usdc, &hundred, None, GemAmountInputType::Fiat, "10".to_string(), Currency::USD).error,
            Some(GemAmountError::PriceMissing)
        );
        assert_eq!(
            GemAmountType::Transfer.entry(&usdc, &hundred, Some(0.0), GemAmountInputType::Fiat, "10".to_string(), Currency::USD).error,
            Some(GemAmountError::PriceMissing)
        );

        let ether = Asset::from_chain(Chain::Ethereum);
        let one_ether = GemAmountType::Transfer.input(&ether, &GemAssetBalance::mock_with_available(1_000_000_000_000_000_000));
        let whole = GemAmountType::Transfer.entry(&ether, &one_ether, Some(2.0), GemAmountInputType::Asset, "1".to_string(), Currency::USD);
        assert_eq!(whole.value, Some(BigInt::from(1_000_000_000_000_000_000u64)));
        assert!(whole.is_max);
        let tiny = GemAmountType::Transfer.entry(&ether, &one_ether, Some(4000.0), GemAmountInputType::Fiat, "0.0000001".to_string(), Currency::USD);
        assert_eq!(tiny.value, Some(BigInt::from(25_000_000u64)));
        assert_eq!(tiny.error, None);
    }

    #[test]
    fn test_entry_rejects_empty_zero_and_invalid_text() {
        let usdc = Asset::mock_hypercore_usdc();
        let input = GemAmountType::Transfer.input(&usdc, &GemAssetBalance::mock_with_available(100_000_000));
        let entry = |input_type, text: &str| GemAmountType::Transfer.entry(&usdc, &input, Some(1.0), input_type, text.to_string(), Currency::USD);

        let empty = entry(GemAmountInputType::Asset, " ");
        assert_eq!(empty.value, None);
        assert_eq!(empty.error, None);
        assert_eq!(empty.equivalent, GemFormattedNumber::currency(0.0, Currency::USD, GemCurrencyStyle::Currency));

        assert_eq!(entry(GemAmountInputType::Asset, "abc").error, Some(GemAmountError::InvalidNumber));
        assert_eq!(entry(GemAmountInputType::Asset, "-1").error, Some(GemAmountError::InvalidNumber));
        assert_eq!(entry(GemAmountInputType::Fiat, "0").error, Some(GemAmountError::Zero));
        assert_eq!(entry(GemAmountInputType::Asset, "0.0000001").error, Some(GemAmountError::Zero));
        assert_eq!(entry(GemAmountInputType::Fiat, "0.0000001").error, Some(GemAmountError::Zero));
        assert_eq!(
            entry(GemAmountInputType::Asset, "200").error,
            Some(GemAmountError::InsufficientBalance {
                asset: usdc.clone(),
                requirement: GemBalanceRequirement::new(BigInt::from(200_000_000), BigInt::from(100_000_000))
            })
        );

        let bnb = Asset::from_chain(Chain::SmartChain);
        let stake = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake };
        let input = stake.input(&bnb, &GemAssetBalance::mock_with_available(5_000_000_000_000_000_000));
        assert_eq!(
            stake.entry(&bnb, &input, Some(1.0), GemAmountInputType::Asset, "0.99".to_string(), Currency::USD).error,
            Some(GemAmountError::BelowMinimum {
                asset: bnb.clone(),
                minimum: BigInt::from(1_000_000_000_000_000_000u64)
            })
        );
    }

    #[test]
    fn test_the_input_carries_the_balance_already_formatted() {
        let ether = Asset::from_chain(Chain::Ethereum);
        let input = GemAmountType::Transfer.input(&ether, &GemAssetBalance::mock_with_available(1_500_000_000_000_000_000));
        assert_eq!(input.available_value, BigInt::from(1_500_000_000_000_000_000u64), "the raw value stays, because entry reads it back");
        let balance = GemFormattedNumber::asset_amount(&BigInt::from(1_500_000_000_000_000_000u64), &ether, GemValueStyle::Auto);
        assert_eq!(balance.unit, GemNumberUnit::Symbol { symbol: ether.symbol.clone() }, "the symbol travels with the number");
        assert_eq!(balance.value, 1.5);
        assert_eq!(input.balance, GemLocalizedText::AmountBalance { balance }, "the screen reads Balance: with the number");
    }

    #[test]
    fn test_entry_max_carries_the_reserved_fee() {
        let cosmos = Asset::from_chain(Chain::Cosmos);
        let config = get_stake_config(StakeChain::Cosmos);
        let available = config.reserved_for_fees * 10 + config.min_amount * 10;
        let stake = GemAmountType::Stake { stake_type: GemAmountStakeType::Stake };
        let input = stake.input(&cosmos, &GemAssetBalance::mock_with_available(available));
        let max = input.max_entry();
        assert_eq!(max.input_type, GemAmountInputType::Asset);
        assert_eq!(max.value, BigInt::from(available - config.reserved_for_fees));

        let max_text = BigNumberFormatter::value(&max.value.to_string(), cosmos.decimals).unwrap();
        let at_max = stake.entry(&cosmos, &input, Some(10.0), GemAmountInputType::Asset, max_text, Currency::USD);
        assert!(at_max.is_max);
        assert_eq!(
            at_max.reserved_fee,
            Some(GemLocalizedText::ReservedFees {
                fee: GemFormattedNumber::asset_amount(&BigInt::from(config.reserved_for_fees), &cosmos, GemValueStyle::Auto)
            })
        );

        let below_max = stake.entry(&cosmos, &input, Some(10.0), GemAmountInputType::Asset, "1".to_string(), Currency::USD);
        assert!(!below_max.is_max);
        assert_eq!(below_max.reserved_fee, None);
    }

    #[test]
    fn test_perpetual_reduce_minimum_allows_only_the_full_small_position() {
        for direction in [PerpetualDirection::Long, PerpetualDirection::Short] {
            for leverage in [1, 3] {
                let minimum = 10_170_000 / u64::from(leverage);
                let available = 6_000_000 / u64::from(leverage);
                let balance = GemAssetBalance::mock_with_available(100_000_000);
                let check = |amount: &GemAmountType, value: BigInt| {
                    validate(
                        &Asset::mock_hypercore_usdc(),
                        &value,
                        &amount.available_value(&Asset::mock_hypercore_usdc(), &balance),
                        &minimum_value(amount, &Asset::mock_hypercore_usdc()),
                    )
                };
                let reduce = GemAmountType::Perpetual {
                    position: GemAmountPerpetualPosition::Reduce { available: available.into() },
                    direction: direction.clone(),
                    price: 3390.0,
                    leverage,
                    size_decimals: 4,
                };
                assert_eq!(check(&reduce, available.into()), Ok(()));
                assert_eq!(check(&reduce, BigInt::ZERO), Err(GemAmountError::Zero));
                assert_eq!(check(&reduce, (-1).into()), Err(GemAmountError::Zero));
                assert_eq!(
                    check(&reduce, (available - 1).into()),
                    Err(GemAmountError::BelowMinimum {
                        asset: Asset::mock_hypercore_usdc(),
                        minimum: available.into()
                    })
                );
                assert_eq!(
                    check(&reduce, (available + 1).into()),
                    Err(GemAmountError::InsufficientBalance {
                        asset: Asset::mock_hypercore_usdc(),
                        requirement: GemBalanceRequirement::new((available + 1).into(), available.into())
                    })
                );
                for position in [GemAmountPerpetualPosition::Open, GemAmountPerpetualPosition::Increase, GemAmountPerpetualPosition::Reduce { available: 20_000_000u64.into() }] {
                    let amount = GemAmountType::Perpetual {
                        position,
                        direction: direction.clone(),
                        price: 3390.0,
                        leverage,
                        size_decimals: 4,
                    };
                    assert_eq!(check(&amount, minimum.into()), Ok(()));
                    assert_eq!(
                        check(&amount, (minimum - 1).into()),
                        Err(GemAmountError::BelowMinimum {
                            asset: Asset::mock_hypercore_usdc(),
                            minimum: minimum.into()
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn test_stake_withdraw_has_no_minimum() {
        let withdraw = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Withdraw {
                delegation: Delegation::mock_base(DelegationBase::mock_with_balance(700, 0)),
            },
        };
        assert_eq!(minimum_value(&withdraw, &Asset::mock_hypercore_usdc()), BigInt::ZERO);
        assert!(!withdraw.input(&Asset::mock_hypercore_usdc(), &GemAssetBalance::mock_with_available(1)).can_change_value);
    }

    #[test]
    fn test_transfer_deposit_withdraw_rules() {
        assert_eq!(minimum_value(&GemAmountType::Transfer, &Asset::from_chain(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(minimum_value(&GemAmountType::Deposit, &Asset::mock_hypercore_usdc()), BigInt::from(MIN_DEPOSIT_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Withdraw, &Asset::mock_hypercore_usdc()), BigInt::from(MIN_WITHDRAW_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Deposit, &Asset::from_chain(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(
            GemAmountType::Withdraw.available_value(
                &Asset::mock_hypercore_usdc(),
                &GemAssetBalance {
                    withdrawable: BigUint::from(7u32),
                    ..GemAssetBalance::mock_with_available(1)
                }
            ),
            BigInt::from(7)
        );
        assert_eq!(
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Reduce { available: BigUint::from(42u32) },
                direction: PerpetualDirection::Long,
                price: 1.0,
                leverage: 1,
                size_decimals: 0
            }
            .available_value(&Asset::mock_hypercore_usdc(), &GemAssetBalance::mock_with_available(1)),
            BigInt::from(42)
        );
    }

    #[test]
    fn test_perpetual_autoclose_follows_the_preference_percents() {
        let open = |direction: PerpetualDirection| GemPerpetualPositionAction::Open {
            data: GemPerpetualTransferData {
                direction,
                ..GemPerpetualTransferData::mock()
            },
        };
        let long = perpetual_autoclose(&open(PerpetualDirection::Long), 10, 50, 20, ",");
        assert_eq!(long.take_profit.as_deref(), Some("105"));
        assert_eq!(long.stop_loss.as_deref(), Some("98"));

        let short = perpetual_autoclose(&open(PerpetualDirection::Short), 10, 50, 20, ",");
        assert_eq!(short.take_profit.as_deref(), Some("95"));
        assert_eq!(short.stop_loss.as_deref(), Some("102"));

        let off = perpetual_autoclose(&open(PerpetualDirection::Long), 10, 0, 20, ".");
        assert_eq!(off.take_profit, None);
        assert_eq!(off.stop_loss.as_deref(), Some("98"));
        let empty = GemPerpetualAutoclose { take_profit: None, stop_loss: None };
        assert_eq!(perpetual_autoclose(&open(PerpetualDirection::Long), 10, 0, 0, "."), empty);
        let increase = GemPerpetualPositionAction::Increase { data: GemPerpetualTransferData::mock() };
        assert_eq!(perpetual_autoclose(&increase, 10, 50, 20, "."), empty, "only an open position takes defaults");
    }

    #[test]
    fn test_validate() {
        let bnb = Asset::from_chain(Chain::SmartChain);
        assert_eq!(validate(&bnb, &BigInt::from(0), &BigInt::from(10), &BigInt::from(0)), Err(GemAmountError::Zero));
        assert_eq!(
            validate(&bnb, &BigInt::from(1), &BigInt::from(10), &BigInt::from(2)),
            Err(GemAmountError::BelowMinimum {
                asset: bnb.clone(),
                minimum: BigInt::from(2)
            })
        );
        assert_eq!(
            validate(&bnb, &BigInt::from(11), &BigInt::from(10), &BigInt::from(0)),
            Err(GemAmountError::InsufficientBalance {
                asset: bnb.clone(),
                requirement: GemBalanceRequirement::new(BigInt::from(11), BigInt::from(10))
            })
        );
        assert_eq!(validate(&bnb, &BigInt::from(5), &BigInt::from(10), &BigInt::from(2)), Ok(()));
        assert_eq!(validate(&bnb, &BigInt::from(0), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));
        assert_eq!(validate(&bnb, &BigInt::from(-1), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));
    }

    #[test]
    fn test_transfer_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemAssetBalance) -> (GemBigUint, GemBigUint, GemBigUint, GemBigUint) = |balance| (balance.available, balance.frozen, balance.locked, balance.withdrawable);
        assert_eq!(GemAmountType::Transfer.available_value(&Asset::from_chain(Chain::Ethereum), &GemAssetBalance::mock_with_available(500)), BigInt::from(500));
    }

    #[test]
    fn test_only_a_transfer_switches_the_input_type() {
        assert!(GemAmountType::Transfer.can_switch_input_type());
        assert!(!GemAmountType::Deposit.can_switch_input_type());
        assert!(!GemAmountType::Withdraw.can_switch_input_type());
        assert!(!GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }.can_switch_input_type());
    }

    #[test]
    fn test_perpetual_open_amount_type_uses_the_selected_leverage() {
        let data = crate::services::perpetual::GemPerpetualTransferData {
            provider: PerpetualProvider::Hypercore,
            direction: PerpetualDirection::Short,
            asset: Asset::mock_hypercore_usdc(),
            base_asset: Asset::mock_hypercore_usdc(),
            asset_index: 1,
            price: 120.5,
            leverage: 3,
            margin_type: primitives::PerpetualMarginType::Cross,
        };

        let open = perpetual_amount_type(&GemPerpetualPositionAction::Open { data }, 10);
        assert_eq!(
            open,
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Open,
                direction: PerpetualDirection::Short,
                price: 120.5,
                leverage: 10,
                size_decimals: 6,
            }
        );
    }

    #[test]
    fn test_stake_amount_type_mirrors_the_input_and_keeps_only_the_confirmed_validator_rewards() {
        let delegation = Delegation::mock_base(DelegationBase::mock_with_balance(100, 5));
        let other = Delegation {
            base: DelegationBase {
                validator_id: "other".into(),
                ..delegation.base.clone()
            },
            validator: DelegationValidator {
                id: "other".into(),
                ..delegation.validator.clone()
            },
            price: None,
        };
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Stake { validator: delegation.validator.clone() }),
            GemAmountType::Stake { stake_type: GemAmountStakeType::Stake }
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Unstake { delegation: delegation.clone() }),
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Unstake { delegation: delegation.clone() }
            }
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Redelegate {
                delegation: delegation.clone(),
                validator: other.validator.clone(),
            }),
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Redelegate { delegation: delegation.clone() }
            }
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Rewards {
                delegations: vec![other.clone(), delegation.clone()],
                validator: other.validator.clone(),
            }),
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Rewards { delegations: vec![other.clone()] }
            }
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Rewards {
                delegations: vec![other.clone(), delegation.clone()],
                validator: delegation.validator.clone(),
            }),
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Rewards { delegations: vec![delegation] }
            }
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Freeze { resource: Resource::Energy }),
            GemAmountType::Stake {
                stake_type: GemAmountStakeType::Freeze { resource: Resource::Energy }
            }
        );
    }

    #[test]
    fn test_earn_amount_type_keeps_the_withdrawn_delegation_and_names_its_provider() {
        let delegation = Delegation::mock_base(DelegationBase::mock_with_balance(100, 0));
        let provider = stake_rules::validator_row(&delegation.validator);

        assert_eq!(
            earn_amount_type(EarnType::Deposit(delegation.validator.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Deposit,
                provider: provider.clone()
            }
        );
        assert_eq!(
            earn_amount_type(EarnType::Withdraw(delegation.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation },
                provider
            },
            "a withdrawal names the validator it is leaving"
        );
    }

    #[test]
    fn test_transfer_data_addresses_a_send_a_deposit_and_a_withdrawal() {
        let recipient = GemRecipient::named("to".into(), "friend".into());
        let owner = GemRecipient::named("owner".into(), "wallet".into());

        let payment = GemPaymentRecipient {
            recipient: recipient.clone(),
            amount: Some("1.5".into()),
        };
        let send = transfer_data(Asset::mock_hypercore_usdc(), GemAmountTransfer::Send { payment: payment.clone() }, None, GemBigInt::from(1), false).unwrap();
        assert!(matches!(send.input_type, TransactionInputType::Transfer { .. }));
        assert_eq!(send.recipient, recipient);

        let deposit = transfer_data(Asset::mock_hypercore_usdc(), GemAmountTransfer::Deposit, None, GemBigInt::from(2), true).unwrap();
        assert!(matches!(deposit.input_type, TransactionInputType::Deposit { .. }));
        assert_eq!(deposit.recipient.address, HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS);
        assert!(deposit.use_max_amount);

        let withdraw = transfer_data(Asset::mock_hypercore_usdc(), GemAmountTransfer::Withdraw, Some(owner.clone()), GemBigInt::from(3), false).unwrap();
        assert!(matches!(withdraw.input_type, TransactionInputType::Withdrawal { .. }));
        assert_eq!(withdraw.recipient, owner);

        assert!(transfer_data(Asset::mock_hypercore_usdc(), GemAmountTransfer::Withdraw, None, GemBigInt::from(3), false).is_err());
    }

    #[test]
    fn test_each_transfer_kind_names_its_amount_type_display_asset_and_prefill() {
        let send = GemAmountTransfer::Send {
            payment: GemPaymentRecipient {
                recipient: GemRecipient::named("to".into(), "friend".into()),
                amount: Some("2".into()),
            },
        };

        assert_eq!(transfer_amount_type(&send), GemAmountType::Transfer);
        assert_eq!(transfer_amount_type(&GemAmountTransfer::Deposit), GemAmountType::Deposit);
        assert_eq!(transfer_amount_type(&GemAmountTransfer::Withdraw), GemAmountType::Withdraw);

        assert_eq!(transfer_display_asset(&send, Asset::mock_hypercore_usdc()), Asset::mock_hypercore_usdc());
        assert_eq!(transfer_display_asset(&GemAmountTransfer::Deposit, Asset::mock_hypercore_usdc()), Asset::mock_hypercore_usdc());
        assert_eq!(
            transfer_display_asset(&GemAmountTransfer::Withdraw, Asset::mock_hypercore_usdc()),
            GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset()
        );

        let balance = GemAssetBalance::mock();
        assert_eq!(
            transfer_input(&send, &Asset::mock_hypercore_usdc(), &balance).prefill,
            Some(GemAmountMaxEntry {
                input_type: GemAmountInputType::Asset,
                value: GemBigInt::from(2_000_000),
            })
        );
        assert_eq!(transfer_input(&GemAmountTransfer::Withdraw, &Asset::mock_hypercore_usdc(), &balance).prefill, None);
    }
}
