use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, AutocloseEstimator, Chain, EarnType, PerpetualDirection, StakeChain, TpslType};

use super::model::{
    GemAmountEarnType, GemAmountEntry, GemAmountEquivalent, GemAmountError, GemAmountInput, GemAmountInputType, GemAmountMaxEntry, GemAmountPerpetualPosition, GemAmountStakeType,
    GemAmountTransfer, GemAmountType, GemPerpetualAutoclose,
};
use crate::config::perpetual_config::{MIN_DEPOSIT_AMOUNT, MIN_WITHDRAW_AMOUNT};
use crate::config::stake::get_stake_config;
use crate::models::custom_types::GemBigInt;
use crate::perpetual::GemPerpetual;
use crate::services::balance::{GemAssetBalance, GemBalanceRequirement};
use crate::services::error::GemServiceError;
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
    pub fn input(&self, asset: &Asset, balance: &GemAssetBalance) -> GemAmountInput {
        let available = self.available_value(asset, balance);
        let reserve = reserve_for_fee(self, asset);
        let max_after_fee = (&available - &reserve).max(BigInt::from(0));
        let reserved_fee = reserves_fee(self, &reserve, &max_after_fee, &minimum_value(self, asset)).then_some(reserve);
        GemAmountInput {
            available_value: available.clone(),
            max_value: if reserved_fee.is_some() { max_after_fee } else { available },
            reserved_fee,
            can_change_value: can_change_value(self, asset),
            shows_asset_balance: shows_asset_balance(self, asset),
            uses_whole_amounts: uses_whole_amounts(self, asset),
        }
    }

    pub fn can_switch_input_type(&self) -> bool {
        matches!(self, Self::Transfer)
    }

    pub fn entry(&self, asset: &Asset, input: &GemAmountInput, price: Option<f64>, input_type: GemAmountInputType, text: String) -> GemAmountEntry {
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
            equivalent: equivalent(value.as_ref(), decimals, price, input_type),
            is_max,
            reserved_fee: if is_max { input.reserved_fee.clone() } else { None },
            value,
            error,
        }
    }
}

#[uniffi::export]
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

fn equivalent(value: Option<&BigInt>, decimals: u32, price: Option<f64>, input_type: GemAmountInputType) -> Option<GemAmountEquivalent> {
    let price = valid_price(price)?;
    let value = value.cloned().unwrap_or_default();
    match input_type {
        GemAmountInputType::Asset => {
            let amount = CryptoFiatConverter::to_fiat(&value.to_string(), decimals, price).ok()?.parse().ok()?;
            Some(GemAmountEquivalent::Fiat { amount })
        }
        GemAmountInputType::Fiat => Some(GemAmountEquivalent::Asset { value }),
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
            GemPerpetualPositionAction::Reduce { position, .. } => GemAmountPerpetualPosition::Reduce {
                available: margin_amount_value(position),
            },
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
            delegations: match stake_rules::rewards_validator(delegations, validator) {
                Some(validator) => delegations.iter().filter(|delegation| delegation.validator.id == validator.id).cloned().collect(),
                None => vec![],
            },
        },
        GemStakeAmountInput::Freeze { resource } => GemAmountStakeType::Freeze { resource: *resource },
        GemStakeAmountInput::Unfreeze { resource } => GemAmountStakeType::Unfreeze { resource: *resource },
    };
    GemAmountType::Stake { stake_type }
}

pub fn earn_amount_type(earn_type: EarnType) -> GemAmountType {
    GemAmountType::Earn {
        earn_type: match earn_type {
            EarnType::Deposit(_) => GemAmountEarnType::Deposit,
            EarnType::Withdraw(delegation) => GemAmountEarnType::Withdraw { delegation },
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

pub fn transfer_prefilled_amount(transfer: &GemAmountTransfer) -> Option<String> {
    match transfer {
        GemAmountTransfer::Send { payment } => payment.amount.clone(),
        GemAmountTransfer::Deposit | GemAmountTransfer::Withdraw => None,
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
                GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => {
                    BigInt::from(delegation.base.balance.clone())
                }
                GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
                GemAmountStakeType::Unfreeze { resource } => transfer_rules::unfreeze_available(resource, balance),
            },
            Self::Earn { earn_type } => match earn_type {
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

pub fn perpetual_autoclose(price: f64, direction: PerpetualDirection, leverage: u8, take_profit_percent: u8, stop_loss_percent: u8) -> GemPerpetualAutoclose {
    let estimator = AutocloseEstimator::for_open(price, 0.0, leverage, direction);
    let target = |percent: u8, trigger_type: TpslType| (percent > 0).then(|| estimator.target_price_from_roe(i32::from(percent), trigger_type));
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
            GemAmountStakeType::Withdraw { .. }
            | GemAmountStakeType::Redelegate { .. }
            | GemAmountStakeType::Unstake { .. }
            | GemAmountStakeType::Unfreeze { .. }
            | GemAmountStakeType::Rewards { .. } => BigInt::from(0),
        },
        GemAmountType::Perpetual {
            position,
            price,
            leverage,
            size_decimals,
            ..
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
    let reserved = stake_chain(asset.chain())
        .map(|chain| BigInt::from(get_stake_config(chain).reserved_for_fees))
        .unwrap_or_default();
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
const DIGIT_ZEROS: [char; 2] = ['\u{0660}', '\u{06F0}'];
const ARABIC_GROUPING: char = '\u{066C}';

pub fn sanitize_number_input(decimal_separator: &str, text: &str, maximum_fraction_digits: Option<u32>, maximum_integer_digits: Option<u32>) -> String {
    let is_separator = |character: &char| SEPARATORS.contains(character) || decimal_separator.contains(*character);
    let typed: String = text.chars().filter(|character| character.is_numeric() || is_separator(character)).collect();
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
    if let Some(digit) = character.to_digit(10) {
        return char::from_digit(digit, 10);
    }
    DIGIT_ZEROS.iter().find_map(|zero| {
        let offset = (character as u32).checked_sub(*zero as u32)?;
        (offset < 10).then(|| char::from_digit(offset, 10)).flatten()
    })
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
        Some((before, after)) => {
            !before.is_empty()
                && before.chars().all(|character| character.is_ascii_digit())
                && after.len() == 3
                && after.chars().all(|character| character.is_ascii_digit())
        }
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
    use super::*;
    use crate::config::perpetual_config::HYPERLIQUID_DEPOSIT_ADDRESS;
    use crate::models::custom_types::GemBigUint;
    use crate::payment::GemPaymentRecipient;
    use primitives::Resource;
    use primitives::asset_balance::BalanceMetadata;
    use primitives::{AssetId, AssetType, Delegation, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};

    #[test]
    fn test_sanitize_number_input_keeps_digits_and_the_first_separator() {
        assert_eq!(sanitize_number_input(".", "abc123.45xyz", None, None), "123.45");
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

    fn asset(chain: Chain) -> Asset {
        Asset::from_chain(chain)
    }

    fn usdc() -> Asset {
        Asset::new(AssetId::from(Chain::HyperCore, Some("usdc".into())), "USDC".into(), "USDC".into(), 6, AssetType::TOKEN)
    }

    fn balance(available: u64, frozen: u64, locked: u64, votes: u32) -> GemAssetBalance {
        GemAssetBalance {
            available: BigUint::from(available),
            frozen: BigUint::from(frozen),
            locked: BigUint::from(locked),
            withdrawable: BigUint::from(7u32),
            metadata: Some(BalanceMetadata {
                votes,
                ..BalanceMetadata::default()
            }),
            ..GemAssetBalance::mock()
        }
    }

    fn delegation(balance: u64, rewards: u64) -> Delegation {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Cosmos),
                state: DelegationState::Active,
                balance: BigUint::from(balance),
                shares: BigUint::default(),
                rewards: BigUint::from(rewards),
                completion_date: None,
                delegation_id: "delegation".into(),
                validator_id: "validator".into(),
            },
            validator: DelegationValidator {
                chain: Chain::Cosmos,
                id: "validator".into(),
                name: "validator".into(),
                is_active: true,
                commission: 0.0,
                apr: 0.0,
                provider_type: StakeProviderType::Stake,
            },
        }
    }

    fn stake(stake_type: GemAmountStakeType) -> GemAmountType {
        GemAmountType::Stake { stake_type }
    }

    #[test]
    fn test_stake_rules_reserve_fees_and_minimums() {
        let cosmos = asset(Chain::Cosmos);
        let config = get_stake_config(StakeChain::Cosmos);
        assert_eq!(minimum_value(&stake(GemAmountStakeType::Stake), &cosmos), BigInt::from(config.min_amount));

        let stake_input = stake(GemAmountStakeType::Stake).input(&cosmos, &balance(config.reserved_for_fees * 10 + config.min_amount * 10, 0, 0, 0));
        assert_eq!(stake_input.reserved_fee, Some(BigInt::from(config.reserved_for_fees)));
        assert!(stake_input.can_change_value);
        assert_eq!(stake_input.max_value, BigInt::from(config.reserved_for_fees * 9 + config.min_amount * 10));

        let tron = asset(Chain::Tron);
        let tron_config = get_stake_config(StakeChain::Tron);
        assert_eq!(reserve_for_fee(&stake(GemAmountStakeType::Stake), &tron), BigInt::ZERO);
        assert_eq!(
            stake(GemAmountStakeType::Stake).available_value(&tron, &balance(1, 5_000_000, 3_000_000, 2)),
            BigInt::from(6_000_000)
        );
        let freeze = stake(GemAmountStakeType::Freeze { resource: Resource::Bandwidth });
        assert!(tron_config.reserved_for_fees > 0);
        let freeze_input = freeze.input(&tron, &balance(tron_config.reserved_for_fees + tron_config.min_amount + 1, 99, 98, 0));
        assert_eq!(freeze_input.available_value, BigInt::from(tron_config.reserved_for_fees + tron_config.min_amount + 1));
        assert_eq!(freeze_input.reserved_fee, Some(BigInt::from(tron_config.reserved_for_fees)));
        assert_eq!(freeze_input.max_value, BigInt::from(tron_config.min_amount + 1));

        let smart_chain = asset(Chain::SmartChain);
        let redelegate = stake(GemAmountStakeType::Redelegate { delegation: delegation(50, 0) });
        let smart_chain_minimum = get_stake_config(StakeChain::SmartChain).min_amount;
        assert!(smart_chain_minimum > 0);
        assert_eq!(minimum_value(&redelegate, &smart_chain), BigInt::from(smart_chain_minimum));
        assert_eq!(minimum_value(&redelegate, &cosmos), BigInt::ZERO);

        let unstake = stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) });
        let solana_unstake = unstake.input(&asset(Chain::Solana), &balance(1, 0, 0, 0));
        assert!(!solana_unstake.can_change_value);
        assert!(!solana_unstake.shows_asset_balance);
        let cosmos_unstake = unstake.input(&cosmos, &balance(1, 0, 0, 0));
        assert!(cosmos_unstake.can_change_value);
        assert!(cosmos_unstake.shows_asset_balance);

        let rewards = stake(GemAmountStakeType::Rewards { delegations: vec![] }).input(&cosmos, &balance(1, 0, 0, 0));
        assert!(!rewards.can_change_value);
        assert!(rewards.shows_asset_balance);
        assert_eq!(
            stake(GemAmountStakeType::Rewards {
                delegations: vec![delegation(10, 3), delegation(20, 4)]
            })
            .available_value(&cosmos, &balance(1, 0, 0, 0)),
            BigInt::from(7)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) }).available_value(&cosmos, &balance(1, 0, 0, 0)),
            BigInt::from(50)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Energy }).available_value(&tron, &balance(1, 2, 3, 0)),
            BigInt::from(3)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Bandwidth }).available_value(&tron, &balance(1, 2, 3, 0)),
            BigInt::from(2)
        );
    }

    #[test]
    fn test_only_a_stake_or_unstake_on_a_whole_unit_chain_uses_whole_amounts() {
        let tron = asset(Chain::Tron);
        let funded = balance(10_000_000, 0, 0, 0);
        assert!(stake(GemAmountStakeType::Stake).input(&tron, &funded).uses_whole_amounts);
        assert!(
            stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) })
                .input(&tron, &funded)
                .uses_whole_amounts
        );
        assert!(!stake(GemAmountStakeType::Rewards { delegations: vec![] }).input(&tron, &funded).uses_whole_amounts);
        assert!(!GemAmountType::Transfer.input(&tron, &funded).uses_whole_amounts);
        assert!(!stake(GemAmountStakeType::Stake).input(&asset(Chain::Cosmos), &funded).uses_whole_amounts);
    }

    #[test]
    fn test_limits_reserve_boundary() {
        let solana = asset(Chain::Solana);
        let config = get_stake_config(StakeChain::Solana);
        let reserve = config.reserved_for_fees;
        let minimum = config.min_amount;
        assert!(reserve > 0 && minimum > 0);

        let at_boundary = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve + minimum, 0, 0, 0));
        assert_eq!(at_boundary.reserved_fee, None);
        assert_eq!(at_boundary.max_value, BigInt::from(reserve + minimum));

        let above_boundary = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve + minimum + 1, 0, 0, 0));
        assert_eq!(above_boundary.reserved_fee, Some(BigInt::from(reserve)));
        assert_eq!(above_boundary.max_value, BigInt::from(minimum + 1));

        let below_reserve = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve - 1, 0, 0, 0));
        assert_eq!(below_reserve.reserved_fee, None);
        assert_eq!(below_reserve.max_value, BigInt::from(reserve - 1));
        assert_eq!(below_reserve.available_value, BigInt::from(reserve - 1));
    }

    #[test]
    fn test_earn_perpetual_and_deposit_sources() {
        let ethereum = asset(Chain::Ethereum);
        assert_eq!(
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Deposit
            }
            .available_value(&ethereum, &balance(11, 0, 0, 0)),
            BigInt::from(11)
        );
        assert_eq!(
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation: delegation(33, 0) }
            }
            .available_value(&ethereum, &balance(11, 0, 0, 0)),
            BigInt::from(33)
        );
        assert_eq!(GemAmountType::Deposit.available_value(&usdc(), &balance(5, 0, 0, 0)), BigInt::from(5));
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(5, 0, 0, 0)), BigInt::from(7));

        let perpetual = |leverage: u8, size_decimals: i32| GemAmountType::Perpetual {
            position: GemAmountPerpetualPosition::Open,
            direction: PerpetualDirection::Long,
            price: 4.0,
            leverage,
            size_decimals,
        };
        assert_eq!(minimum_value(&perpetual(1, 0), &usdc()), BigInt::from(12_000_000));
        assert_eq!(minimum_value(&perpetual(1, 1), &usdc()), BigInt::from(10_000_000));
        assert_eq!(minimum_value(&perpetual(2, 0), &usdc()), BigInt::from(6_000_000));
        assert_eq!(perpetual(1, 0).available_value(&usdc(), &balance(9, 0, 0, 0)), BigInt::from(9));
    }

    fn transfer_entry(asset: &Asset, available: u64, price: Option<f64>, input_type: GemAmountInputType, text: &str) -> GemAmountEntry {
        let input = GemAmountType::Transfer.input(asset, &balance(available, 0, 0, 0));
        GemAmountType::Transfer.entry(asset, &input, price, input_type, text.to_string())
    }

    #[test]
    fn test_entry_reads_the_text_in_the_input_units() {
        let usdc = usdc();
        let typed = transfer_entry(&usdc, 100_000_000_000, Some(2.0), GemAmountInputType::Asset, "1000.123456");
        assert_eq!(typed.value, Some(BigInt::from(1_000_123_456)));
        assert_eq!(typed.error, None);
        assert_eq!(typed.equivalent, Some(GemAmountEquivalent::Fiat { amount: 2000.246912 }));
        assert!(!typed.is_max);
        assert_eq!(transfer_entry(&usdc, 100_000_000, None, GemAmountInputType::Asset, "1.5").equivalent, None);

        let fiat = transfer_entry(&usdc, 100_000_000_000, Some(1.0), GemAmountInputType::Fiat, "1000.123456");
        assert_eq!(fiat.value, Some(BigInt::from(1_000_120_000)));
        assert_eq!(
            fiat.equivalent,
            Some(GemAmountEquivalent::Asset {
                value: BigInt::from(1_000_120_000)
            })
        );
        assert_eq!(
            transfer_entry(&usdc, 100_000_000_000, Some(1.0), GemAmountInputType::Fiat, "1000").value,
            Some(BigInt::from(1_000_000_000))
        );
        assert_eq!(
            transfer_entry(&usdc, 100_000_000, None, GemAmountInputType::Fiat, "10").error,
            Some(GemAmountError::PriceMissing)
        );
        assert_eq!(
            transfer_entry(&usdc, 100_000_000, Some(0.0), GemAmountInputType::Fiat, "10").error,
            Some(GemAmountError::PriceMissing)
        );

        let ether = asset(Chain::Ethereum);
        let whole = transfer_entry(&ether, 1_000_000_000_000_000_000, Some(2.0), GemAmountInputType::Asset, "1");
        assert_eq!(whole.value, Some(BigInt::from(1_000_000_000_000_000_000u64)));
        assert!(whole.is_max);
        let tiny = transfer_entry(&ether, 1_000_000_000_000_000_000, Some(4000.0), GemAmountInputType::Fiat, "0.0000001");
        assert_eq!(tiny.value, Some(BigInt::from(25_000_000u64)));
        assert_eq!(tiny.error, None);
    }

    #[test]
    fn test_entry_rejects_empty_zero_and_invalid_text() {
        let usdc = usdc();
        let entry = |input_type, text| transfer_entry(&usdc, 100_000_000, Some(1.0), input_type, text);

        let empty = entry(GemAmountInputType::Asset, " ");
        assert_eq!(empty.value, None);
        assert_eq!(empty.error, None);
        assert_eq!(empty.equivalent, Some(GemAmountEquivalent::Fiat { amount: 0.0 }));

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

        let bnb = asset(Chain::SmartChain);
        let stake = stake(GemAmountStakeType::Stake);
        let input = stake.input(&bnb, &balance(5_000_000_000_000_000_000, 0, 0, 0));
        assert_eq!(
            stake.entry(&bnb, &input, Some(1.0), GemAmountInputType::Asset, "0.99".to_string()).error,
            Some(GemAmountError::BelowMinimum {
                asset: bnb.clone(),
                minimum: BigInt::from(1_000_000_000_000_000_000u64)
            })
        );
    }

    #[test]
    fn test_entry_max_carries_the_reserved_fee() {
        let cosmos = asset(Chain::Cosmos);
        let config = get_stake_config(StakeChain::Cosmos);
        let available = config.reserved_for_fees * 10 + config.min_amount * 10;
        let stake = stake(GemAmountStakeType::Stake);
        let input = stake.input(&cosmos, &balance(available, 0, 0, 0));
        let max = input.max_entry();
        assert_eq!(max.input_type, GemAmountInputType::Asset);
        assert_eq!(max.value, BigInt::from(available - config.reserved_for_fees));

        let max_text = BigNumberFormatter::value(&max.value.to_string(), cosmos.decimals).unwrap();
        let at_max = stake.entry(&cosmos, &input, Some(10.0), GemAmountInputType::Asset, max_text);
        assert!(at_max.is_max);
        assert_eq!(at_max.reserved_fee, Some(BigInt::from(config.reserved_for_fees)));

        let below_max = stake.entry(&cosmos, &input, Some(10.0), GemAmountInputType::Asset, "1".to_string());
        assert!(!below_max.is_max);
        assert_eq!(below_max.reserved_fee, None);
    }

    #[test]
    fn test_perpetual_reduce_minimum_allows_only_the_full_small_position() {
        for direction in [PerpetualDirection::Long, PerpetualDirection::Short] {
            for leverage in [1, 3] {
                let minimum = 10_170_000 / u64::from(leverage);
                let available = 6_000_000 / u64::from(leverage);
                let amount_type = |position| GemAmountType::Perpetual {
                    position,
                    direction: direction.clone(),
                    price: 3390.0,
                    leverage,
                    size_decimals: 4,
                };
                let balance = balance(100_000_000, 0, 0, 0);
                let check = |amount: &GemAmountType, value: BigInt| validate(&usdc(), &value, &amount.available_value(&usdc(), &balance), &minimum_value(amount, &usdc()));
                let reduce = amount_type(GemAmountPerpetualPosition::Reduce { available: available.into() });
                assert_eq!(check(&reduce, available.into()), Ok(()));
                assert_eq!(check(&reduce, BigInt::ZERO), Err(GemAmountError::Zero));
                assert_eq!(check(&reduce, (-1).into()), Err(GemAmountError::Zero));
                assert_eq!(
                    check(&reduce, (available - 1).into()),
                    Err(GemAmountError::BelowMinimum {
                        asset: usdc(),
                        minimum: available.into()
                    })
                );
                assert_eq!(
                    check(&reduce, (available + 1).into()),
                    Err(GemAmountError::InsufficientBalance {
                        asset: usdc(),
                        requirement: GemBalanceRequirement::new((available + 1).into(), available.into())
                    })
                );
                for position in [
                    GemAmountPerpetualPosition::Open,
                    GemAmountPerpetualPosition::Increase,
                    GemAmountPerpetualPosition::Reduce { available: 20_000_000u64.into() },
                ] {
                    let amount = amount_type(position);
                    assert_eq!(check(&amount, minimum.into()), Ok(()));
                    assert_eq!(
                        check(&amount, (minimum - 1).into()),
                        Err(GemAmountError::BelowMinimum {
                            asset: usdc(),
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
            stake_type: GemAmountStakeType::Withdraw { delegation: delegation(700, 0) },
        };
        assert_eq!(minimum_value(&withdraw, &usdc()), BigInt::ZERO);
        assert!(!withdraw.input(&usdc(), &balance(1, 0, 0, 0)).can_change_value);
    }

    #[test]
    fn test_transfer_deposit_withdraw_rules() {
        assert_eq!(minimum_value(&GemAmountType::Transfer, &asset(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(minimum_value(&GemAmountType::Deposit, &usdc()), BigInt::from(MIN_DEPOSIT_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Withdraw, &usdc()), BigInt::from(MIN_WITHDRAW_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Deposit, &asset(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(1, 0, 0, 0)), BigInt::from(7));
        assert_eq!(
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Reduce { available: BigUint::from(42u32) },
                direction: PerpetualDirection::Long,
                price: 1.0,
                leverage: 1,
                size_decimals: 0
            }
            .available_value(&usdc(), &balance(1, 0, 0, 0)),
            BigInt::from(42)
        );
    }

    #[test]
    fn test_perpetual_autoclose_follows_the_preference_percents() {
        let long = perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 50, 20);
        assert_eq!(long.take_profit, Some(105.0));
        assert_eq!(long.stop_loss, Some(98.0));

        let short = perpetual_autoclose(100.0, PerpetualDirection::Short, 10, 50, 20);
        assert_eq!(short.take_profit, Some(95.0));
        assert_eq!(short.stop_loss, Some(102.0));

        let off = perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 0, 20);
        assert_eq!(off.take_profit, None);
        assert_eq!(off.stop_loss, Some(98.0));
        assert_eq!(
            perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 0, 0),
            GemPerpetualAutoclose {
                take_profit: None,
                stop_loss: None
            }
        );
    }

    #[test]
    fn test_validate() {
        let bnb = asset(Chain::SmartChain);
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
        assert_eq!(GemAmountType::Transfer.available_value(&asset(Chain::Ethereum), &balance(500, 0, 0, 0)), BigInt::from(500));
    }

    #[test]
    fn test_only_a_transfer_switches_the_input_type() {
        assert!(GemAmountType::Transfer.can_switch_input_type());
        assert!(!GemAmountType::Deposit.can_switch_input_type());
        assert!(!GemAmountType::Withdraw.can_switch_input_type());
        assert!(!stake(GemAmountStakeType::Stake).can_switch_input_type());
    }

    #[test]
    fn test_perpetual_open_amount_type_uses_the_selected_leverage() {
        let data = crate::services::perpetual::GemPerpetualTransferData {
            provider: PerpetualProvider::Hypercore,
            direction: PerpetualDirection::Short,
            asset: usdc(),
            base_asset: usdc(),
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
        let delegation = delegation(100, 5);
        let other = Delegation {
            base: DelegationBase {
                validator_id: "other".into(),
                ..delegation.base.clone()
            },
            validator: DelegationValidator {
                id: "other".into(),
                ..delegation.validator.clone()
            },
        };
        let validators = vec![delegation.validator.clone(), other.validator.clone()];

        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Stake {
                validators: validators.clone(),
                validator: None
            }),
            stake(GemAmountStakeType::Stake)
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Unstake { delegation: delegation.clone() }),
            stake(GemAmountStakeType::Unstake { delegation: delegation.clone() })
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Redelegate {
                validators,
                delegation: delegation.clone(),
                validator: Some(other.validator.clone()),
            }),
            stake(GemAmountStakeType::Redelegate { delegation: delegation.clone() })
        );
        let rewards = |validator| GemStakeAmountInput::Rewards {
            delegations: vec![other.clone(), delegation.clone()],
            validator,
        };
        assert_eq!(stake_amount_type(&rewards(None)), stake(GemAmountStakeType::Rewards { delegations: vec![other.clone()] }));
        assert_eq!(
            stake_amount_type(&rewards(Some(delegation.validator.clone()))),
            stake(GemAmountStakeType::Rewards { delegations: vec![delegation] })
        );
        assert_eq!(
            stake_amount_type(&GemStakeAmountInput::Freeze { resource: Resource::Energy }),
            stake(GemAmountStakeType::Freeze { resource: Resource::Energy })
        );
    }

    #[test]
    fn test_earn_amount_type_keeps_the_withdrawn_delegation() {
        let delegation = delegation(100, 0);

        assert_eq!(
            earn_amount_type(EarnType::Deposit(delegation.validator.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Deposit
            }
        );
        assert_eq!(
            earn_amount_type(EarnType::Withdraw(delegation.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation }
            }
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
        let send = transfer_data(usdc(), GemAmountTransfer::Send { payment: payment.clone() }, None, GemBigInt::from(1), false).unwrap();
        assert!(matches!(send.input_type, TransactionInputType::Transfer { .. }));
        assert_eq!(send.recipient, recipient);

        let deposit = transfer_data(usdc(), GemAmountTransfer::Deposit, None, GemBigInt::from(2), true).unwrap();
        assert!(matches!(deposit.input_type, TransactionInputType::Deposit { .. }));
        assert_eq!(deposit.recipient.address, HYPERLIQUID_DEPOSIT_ADDRESS);
        assert!(deposit.use_max_amount);

        let withdraw = transfer_data(usdc(), GemAmountTransfer::Withdraw, Some(owner.clone()), GemBigInt::from(3), false).unwrap();
        assert!(matches!(withdraw.input_type, TransactionInputType::Withdrawal { .. }));
        assert_eq!(withdraw.recipient, owner);

        assert!(transfer_data(usdc(), GemAmountTransfer::Withdraw, None, GemBigInt::from(3), false).is_err());
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

        assert_eq!(transfer_display_asset(&send, usdc()), usdc());
        assert_eq!(transfer_display_asset(&GemAmountTransfer::Deposit, usdc()), usdc());
        assert_eq!(
            transfer_display_asset(&GemAmountTransfer::Withdraw, usdc()),
            GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset()
        );

        assert_eq!(transfer_prefilled_amount(&send).as_deref(), Some("2"));
        assert_eq!(transfer_prefilled_amount(&GemAmountTransfer::Withdraw), None);
    }
}
