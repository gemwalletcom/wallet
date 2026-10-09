use crate::formatted_number::GemFormattedNumber;
use crate::models::GemEarnType;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::list::GemInfoTopic;
use crate::payment::GemPaymentRecipient;
use crate::precision::GemValueStyle;
use crate::services::assets::model::GemAssetItemRow;
use crate::services::balance::{GemAssetBalance, GemBalanceRequirement};
use crate::services::localization::GemLocalizedText;
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::perpetual::autoclose::{GemAutocloseDraft, GemAutocloseSession, autoclose_draft, autoclose_open_session};
use crate::services::perpetual::model::perpetual_open_row;
use crate::services::stake::model::{GemStakeAmountInput, GemValidatorRow};
use primitives::{Asset, AssetData, Currency, Delegation, PerpetualDirection, Resource, TpslType};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountType {
    Transfer,
    Deposit,
    Withdraw,
    Stake {
        stake_type: GemAmountStakeType,
    },
    Earn {
        earn_type: GemAmountEarnType,
        provider: GemValidatorRow,
    },
    Perpetual {
        position: GemAmountPerpetualPosition,
        direction: PerpetualDirection,
        price: f64,
        leverage: u8,
        size_decimals: u32,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountTitle {
    Send,
    Deposit,
    Withdraw,
    Stake,
    Unstake,
    Redelegate,
    Rewards,
    Freeze,
    Unfreeze,
    PerpetualOpen { direction: PerpetualDirection },
    PerpetualIncrease { direction: PerpetualDirection },
    PerpetualReduce { direction: PerpetualDirection },
}

#[uniffi::export]
impl GemAmountType {
    pub fn title(&self) -> GemAmountTitle {
        super::rules::amount_title(self)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountTransfer {
    Send { payment: GemPaymentRecipient },
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemAmountRequest {
    Transfer { transfer: GemAmountTransfer },
    Stake { input: GemStakeAmountInput },
    Earn { earn_type: GemEarnType },
    Perpetual { session: GemPerpetualAmountSession },
}

#[uniffi::export]
impl GemAmountRequest {
    pub fn amount_type(&self) -> GemAmountType {
        match self {
            Self::Transfer { transfer } => super::rules::transfer_amount_type(transfer),
            Self::Stake { input } => input.amount_type(),
            Self::Earn { earn_type } => super::rules::earn_amount_type(earn_type.clone()),
            Self::Perpetual { session } => super::rules::perpetual_amount_type(&session.action, session.leverage()),
        }
    }

    pub fn display_asset(&self, asset: Asset) -> Asset {
        match self {
            Self::Transfer { transfer } => super::rules::transfer_display_asset(transfer, asset),
            Self::Stake { .. } | Self::Earn { .. } | Self::Perpetual { .. } => asset,
        }
    }

    pub fn input(&self, data: AssetData) -> GemAmountInput {
        let balance = GemAssetBalance::from(&data);
        match self {
            Self::Transfer { transfer } => super::rules::transfer_input(transfer, &data.asset, &balance),
            Self::Stake { .. } | Self::Earn { .. } | Self::Perpetual { .. } => self.amount_type().input(&data.asset, &balance),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountStakeType {
    Stake,
    Unstake { delegation: Delegation },
    Redelegate { delegation: Delegation },
    Withdraw { delegation: Delegation },
    Rewards { delegations: Vec<Delegation> },
    Freeze { resource: Resource },
    Unfreeze { resource: Resource },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemAmountEarnType {
    Deposit,
    Withdraw { delegation: Delegation },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountPerpetualPosition {
    Open,
    Increase,
    Reduce { available: GemBigUint },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountInput {
    pub icon: crate::services::assets::icon::GemAssetIcon,
    pub available_value: GemBigInt,
    pub balance: GemLocalizedText,
    pub max_value: GemBigInt,
    pub reserved_fee: Option<GemBigInt>,
    pub can_change_value: bool,
    pub shows_asset_balance: bool,
    pub uses_whole_amounts: bool,
    pub prefill: Option<GemAmountMaxEntry>,
    pub focuses_input: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemAmountInputType {
    Asset,
    Fiat,
}

#[uniffi::export]
impl GemAmountInputType {
    pub fn toggled(&self) -> Self {
        match self {
            Self::Asset => Self::Fiat,
            Self::Fiat => Self::Asset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountField {
    pub symbol: GemAmountSymbol,
    pub placement: GemAmountSymbolPlacement,
    pub keyboard: GemAmountKeyboard,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountSymbol {
    Asset { symbol: String },
    Currency { currency: Currency },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAmountSymbolPlacement {
    Leading,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAmountKeyboard {
    Decimal,
    Whole,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountEntry {
    pub value: Option<GemBigInt>,
    pub error: Option<GemAmountError>,
    pub equivalent: GemFormattedNumber,
    pub is_max: bool,
    pub reserved_fee: Option<GemLocalizedText>,
}

#[uniffi::export]
impl GemAmountEntry {
    pub fn allows_confirm(&self) -> bool {
        self.error.is_none() && self.value.as_ref().is_some_and(|value| value.sign() == num_bigint::Sign::Plus)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountMaxEntry {
    pub input_type: GemAmountInputType,
    pub value: GemBigInt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemPerpetualAutoclose {
    pub take_profit: Option<String>,
    pub stop_loss: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemAmountError {
    InvalidNumber,
    PriceMissing,
    Zero,
    BelowMinimum { asset: Asset, minimum: GemBigInt },
    InsufficientBalance { asset: Asset, requirement: GemBalanceRequirement },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountErrorDisplay {
    None,
    InvalidAmount,
    BelowMinimum { minimum: GemFormattedNumber, topic: GemInfoTopic },
    InsufficientBalance { title: String },
}

#[uniffi::export]
impl GemAmountError {
    pub fn display(&self) -> GemAmountErrorDisplay {
        match self {
            Self::Zero => GemAmountErrorDisplay::None,
            Self::InvalidNumber | Self::PriceMissing => GemAmountErrorDisplay::InvalidAmount,
            Self::BelowMinimum { asset, minimum } => GemAmountErrorDisplay::BelowMinimum {
                minimum: GemFormattedNumber::asset_amount(minimum, asset, GemValueStyle::Auto),
                topic: GemInfoTopic::MinimumAmount {
                    asset: asset.clone(),
                    minimum: minimum.clone(),
                },
            },
            Self::InsufficientBalance { asset, .. } => GemAmountErrorDisplay::InsufficientBalance { title: asset.display_title() },
        }
    }
}

#[uniffi::export]
impl GemAmountErrorDisplay {
    pub fn info(&self) -> Option<GemInfoTopic> {
        match self {
            Self::BelowMinimum { topic, .. } => Some(topic.clone()),
            Self::None | Self::InvalidAmount | Self::InsufficientBalance { .. } => None,
        }
    }
}

impl std::fmt::Display for GemAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber => write!(f, "amount is not a number"),
            Self::PriceMissing => write!(f, "amount needs a price to convert from fiat"),
            Self::Zero => write!(f, "amount must be positive"),
            Self::BelowMinimum { asset, minimum } => write!(f, "amount is below the minimum {minimum} {}", asset.symbol),
            Self::InsufficientBalance { asset, requirement } => write!(f, "amount exceeds the available {} balance {}", asset.symbol, requirement.available),
        }
    }
}

impl std::error::Error for GemAmountError {}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNumberFormat {
    pub decimal_separator: String,
}

#[uniffi::export]
impl GemNumberFormat {
    pub fn sanitize(&self, input: String, maximum_fraction_digits: Option<u32>, maximum_integer_digits: Option<u32>) -> String {
        super::rules::sanitize_number_input(&self.decimal_separator, &input, maximum_fraction_digits, maximum_integer_digits)
    }

    pub fn input_text(&self, value: String, decimals: u32) -> Option<String> {
        super::rules::input_text(&self.decimal_separator, &value, decimals)
    }

    pub fn plain(&self, input: String) -> String {
        super::rules::plain_number(&self.decimal_separator, &input)
    }
}

impl GemNumberFormat {
    pub fn value_text(&self, value: f64) -> String {
        super::rules::value_text(&self.decimal_separator, value)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountSession {
    pub text: String,
    pub input_type: GemAmountInputType,
    pub format: GemNumberFormat,
}

#[uniffi::export]
pub fn new_amount_session(format: GemNumberFormat) -> GemAmountSession {
    GemAmountSession {
        text: String::new(),
        input_type: GemAmountInputType::Asset,
        format,
    }
}

#[uniffi::export]
impl GemAmountSession {
    pub fn on_text(&self, text: String) -> Self {
        Self { text, ..self.clone() }
    }

    pub fn on_toggle(&self) -> Self {
        Self {
            input_type: self.input_type.toggled(),
            text: String::new(),
            ..self.clone()
        }
    }

    pub fn on_clear(&self) -> Self {
        self.on_text(String::new())
    }

    pub fn on_max(&self, input: GemAmountInput, asset: Asset) -> Self {
        self.filled(input.max_entry(), &asset)
    }

    pub fn on_prefill(&self, input: GemAmountInput, asset: Asset) -> Self {
        match input.prefill {
            Some(prefill) => self.filled(prefill, &asset),
            None => self.clone(),
        }
    }

    pub fn field(&self, asset: Asset, input: GemAmountInput, currency: Currency) -> GemAmountField {
        let (symbol, placement) = match self.input_type {
            GemAmountInputType::Asset => (GemAmountSymbol::Asset { symbol: asset.symbol }, GemAmountSymbolPlacement::Trailing),
            GemAmountInputType::Fiat => (GemAmountSymbol::Currency { currency }, GemAmountSymbolPlacement::Leading),
        };
        GemAmountField {
            symbol,
            placement,
            keyboard: match input.uses_whole_amounts {
                true => GemAmountKeyboard::Whole,
                false => GemAmountKeyboard::Decimal,
            },
        }
    }

    pub fn entry(&self, amount_type: GemAmountType, asset: Asset, input: GemAmountInput, price: Option<f64>, currency: Currency) -> GemAmountEntry {
        amount_type.entry(&asset, &input, price, self.input_type, self.format.plain(self.text.clone()), currency)
    }
}

impl GemAmountSession {
    fn filled(&self, entry: GemAmountMaxEntry, asset: &Asset) -> Self {
        match self.format.input_text(entry.value.to_string(), asset.decimals) {
            Some(text) => Self {
                text,
                input_type: entry.input_type,
                ..self.clone()
            },
            None => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountExtras {
    None,
    Validator {
        row: GemValidatorRow,
        can_select: bool,
    },
    Resources {
        options: Vec<primitives::Resource>,
        selected: primitives::Resource,
    },
    Provider {
        row: GemValidatorRow,
    },
    Perpetual {
        leverage: Option<GemAmountLeverage>,
        autoclose: Option<crate::models::list::GemListRow>,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountLeverage {
    pub selection: GemLeverageSelection,
    pub direction: PerpetualDirection,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemLeverageSelection {
    pub options: Vec<crate::services::settings::rules::GemPickerOption>,
    pub selected: crate::services::settings::rules::GemPickerOption,
}

impl GemLeverageSelection {
    pub(super) fn picked(self, leverage: u8) -> Self {
        let selected = self.options.iter().find(|option| option.value == leverage).cloned().unwrap_or(self.selected);
        Self { selected, ..self }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualAmountSession {
    pub action: GemPerpetualPositionAction,
    pub leverage: Option<GemLeverageSelection>,
    pub take_profit_percent: u8,
    pub stop_loss_percent: u8,
    pub autoclose: GemAutocloseDraft,
    pub format: GemNumberFormat,
}

#[uniffi::export]
impl GemPerpetualAmountSession {
    pub fn on_leverage(&self, leverage: u8) -> Self {
        Self {
            leverage: self.leverage.clone().map(|selection| selection.picked(leverage)),
            ..self.clone()
        }
        .with_defaults()
    }

    pub fn on_autoclose(&self, take_profit: String, stop_loss: String) -> Self {
        Self {
            autoclose: self.autoclose.on_edited(TpslType::TakeProfit, Some(take_profit)).on_edited(TpslType::StopLoss, Some(stop_loss)),
            ..self.clone()
        }
    }

    pub fn autoclose_session(&self, amount: String) -> GemAutocloseSession {
        let data = self.action.data();
        autoclose_open_session(data.direction.clone(), data.price, self.size(amount), self.leverage(), data.asset.decimals, data.provider.clone(), self.format.clone())
            .on_input(TpslType::TakeProfit, self.autoclose.take_profit.value.clone().unwrap_or_default())
            .on_input(TpslType::StopLoss, self.autoclose.stop_loss.value.clone().unwrap_or_default())
    }

    pub fn open_row(&self, amount: String) -> GemAssetItemRow {
        let data = self.action.data();
        perpetual_open_row(data.asset.id.clone(), data.asset.symbol.clone(), data.direction.clone(), self.leverage(), self.size(amount))
    }
}

impl GemPerpetualAmountSession {
    pub fn new(action: GemPerpetualPositionAction, preferred_leverage: u8, take_profit_percent: u8, stop_loss_percent: u8, format: GemNumberFormat) -> Self {
        Self {
            leverage: match &action {
                GemPerpetualPositionAction::Open { data } => super::rules::perpetual_leverage_selection(data.leverage, preferred_leverage),
                GemPerpetualPositionAction::Increase { .. } | GemPerpetualPositionAction::Reduce { .. } => None,
            },
            action,
            take_profit_percent,
            stop_loss_percent,
            autoclose: autoclose_draft(None, None),
            format,
        }
        .with_defaults()
    }

    pub fn leverage(&self) -> u8 {
        self.leverage.as_ref().map_or(self.action.data().leverage, |selection| selection.selected.value)
    }

    fn size(&self, amount: String) -> f64 {
        self.format.plain(amount).parse().unwrap_or(0.0)
    }

    fn with_defaults(self) -> Self {
        let defaults = super::rules::perpetual_autoclose(&self.action, self.leverage(), self.take_profit_percent, self.stop_loss_percent, &self.format.decimal_separator);
        Self {
            autoclose: self.autoclose.on_defaults(defaults.take_profit, defaults.stop_loss),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatted_number::GemNumberUnit;
    use crate::services::perpetual::model::GemPerpetualTransferData;
    use primitives::Chain;

    #[test]
    fn test_the_amount_field_fills_max_and_prefill_in_the_callers_separator_and_a_toggle_clears_it() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let input = GemAmountType::Transfer.input(&asset, &GemAssetBalance::mock_with_available(1_500_000_000_000_000_000));
        let session = new_amount_session(GemNumberFormat { decimal_separator: ",".to_string() });

        let max = session.on_text("0,1".to_string()).on_max(input.clone(), asset.clone());
        assert_eq!(max.text, "1,5");
        assert_eq!(max.input_type, GemAmountInputType::Asset);
        assert_eq!(
            max.entry(GemAmountType::Transfer, asset.clone(), input.clone(), None, primitives::Currency::USD).value,
            Some(GemBigInt::from(1_500_000_000_000_000_000u64))
        );

        let toggled = max.on_toggle();
        assert_eq!((toggled.text.as_str(), toggled.input_type), ("", GemAmountInputType::Fiat), "switching sides starts over");
        assert_eq!(session.on_prefill(input, asset).text, "", "a transfer the user types has nothing to prefill");
    }

    #[test]
    fn test_the_field_puts_the_asset_symbol_after_and_the_currency_before_the_number() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let input = GemAmountType::Transfer.input(&asset, &GemAssetBalance::mock_with_available(1));
        let session = new_amount_session(GemNumberFormat { decimal_separator: ".".to_string() });

        assert_eq!(
            session.field(asset.clone(), input.clone(), primitives::Currency::EUR),
            GemAmountField {
                symbol: GemAmountSymbol::Asset { symbol: "ETH".to_string() },
                placement: GemAmountSymbolPlacement::Trailing,
                keyboard: GemAmountKeyboard::Decimal,
            }
        );
        let fiat = session.on_toggle().field(asset.clone(), input.clone(), primitives::Currency::EUR);
        assert_eq!((fiat.symbol, fiat.placement), (GemAmountSymbol::Currency { currency: primitives::Currency::EUR }, GemAmountSymbolPlacement::Leading));

        let whole = GemAmountInput { uses_whole_amounts: true, ..input };
        assert_eq!(session.field(asset, whole, primitives::Currency::EUR).keyboard, GemAmountKeyboard::Whole, "a whole-unit asset takes no decimal point");
    }

    #[test]
    fn test_a_leverage_selection_keeps_the_picked_leverage_it_offers() {
        let option = crate::services::settings::rules::leverage_option;
        let selection = GemLeverageSelection {
            options: vec![option(1), option(5), option(10)],
            selected: option(5),
        };

        assert_eq!(selection.clone().picked(10).selected, option(10));
        assert_eq!(selection.picked(40).selected, option(5), "a leverage the market no longer offers falls back to the default");
    }

    fn perpetual_session(action: GemPerpetualPositionAction, decimal_separator: &str) -> GemPerpetualAmountSession {
        GemPerpetualAmountSession::new(
            action,
            10,
            50,
            20,
            GemNumberFormat {
                decimal_separator: decimal_separator.to_string(),
            },
        )
    }

    fn open(max_leverage: u8) -> GemPerpetualPositionAction {
        GemPerpetualPositionAction::Open {
            data: GemPerpetualTransferData {
                leverage: max_leverage,
                ..GemPerpetualTransferData::mock()
            },
        }
    }

    #[test]
    fn test_a_perpetual_open_offers_leverage_up_to_the_market_and_refills_the_autoclose_defaults_on_a_change() {
        let session = perpetual_session(open(20), ",");
        let selection = session.leverage.clone().unwrap();
        assert_eq!(selection.options.last().map(|option| option.value), Some(20));
        assert_eq!(session.leverage(), 10);
        assert_eq!(session.autoclose.take_profit.value.as_deref(), Some("105"));
        assert_eq!(session.autoclose.stop_loss.value.as_deref(), Some("98"));

        let five = session.on_leverage(5);
        assert_eq!(five.leverage(), 5);
        assert_eq!(five.autoclose.take_profit.value.as_deref(), Some("110"));
        assert_eq!(five.autoclose.stop_loss.value.as_deref(), Some("96"));
        assert_eq!(five.on_leverage(40).leverage(), 5, "a leverage the market does not offer keeps the current one");
    }

    #[test]
    fn test_an_autoclose_price_the_user_set_survives_a_leverage_change() {
        let session = perpetual_session(open(20), ".").on_autoclose("120".to_string(), " ".to_string()).on_leverage(5);

        assert_eq!(session.autoclose.take_profit.value.as_deref(), Some("120"));
        assert_eq!(session.autoclose.stop_loss.value, None, "a cleared field stays cleared");
        assert_eq!(session.autoclose.prices("."), (Some(120.0), None));
    }

    #[test]
    fn test_increase_and_reduce_keep_the_position_leverage_and_take_no_autoclose() {
        let increase = perpetual_session(GemPerpetualPositionAction::Increase { data: GemPerpetualTransferData::mock() }, ".");

        assert_eq!(increase.leverage, None);
        assert_eq!(increase.leverage(), 3);
        assert_eq!(increase.autoclose, autoclose_draft(None, None));
        assert_eq!(increase.on_leverage(10).leverage(), 3);
    }

    #[test]
    fn test_the_autoclose_sheet_opens_at_the_order_price_with_the_typed_size_and_the_draft() {
        let session = perpetual_session(open(20), ",");
        let sheet = session.autoclose_session("1 000,5".to_string());

        assert_eq!(sheet.prices.market, 100.0);
        assert_eq!(sheet.estimate.entry_price, 100.0);
        assert_eq!(sheet.estimate.size, 1000.5);
        assert_eq!(sheet.estimate.leverage, 10);
        assert_eq!(sheet.take_profit_text, "105");
        assert_eq!(sheet.stop_loss_text, "98");
        assert_eq!(sheet.modify.take_profit.price, Some(105.0));
        assert_eq!(session.autoclose_session("abc".to_string()).estimate.size, 0.0);
    }

    #[test]
    fn test_the_open_row_shows_the_picked_leverage_and_the_typed_size() {
        let session = perpetual_session(open(20), ".").on_leverage(5);

        assert_eq!(session.open_row("250".to_string()), perpetual_open_row(Asset::mock().id, "ETH".to_string(), PerpetualDirection::Long, 5, 250.0));
        assert_eq!(session.open_row(String::new()).trailing, crate::services::assets::model::GemAssetItemTrailing::None);
    }

    #[test]
    fn test_the_input_type_toggles_between_the_asset_and_the_fiat_side() {
        assert_eq!(GemAmountInputType::Asset.toggled(), GemAmountInputType::Fiat);
        assert_eq!(GemAmountInputType::Fiat.toggled(), GemAmountInputType::Asset);
        assert_eq!(GemAmountInputType::Asset.toggled().toggled(), GemAmountInputType::Asset);
    }

    #[test]
    fn test_a_zero_amount_shows_nothing_and_a_short_balance_names_the_asset() {
        assert_eq!(GemAmountError::Zero.display(), GemAmountErrorDisplay::None);
        assert_eq!(GemAmountError::InvalidNumber.display(), GemAmountErrorDisplay::InvalidAmount);
        assert_eq!(GemAmountError::PriceMissing.display(), GemAmountErrorDisplay::InvalidAmount);

        let asset = Asset::from_chain(Chain::Ethereum);
        let requirement = GemBalanceRequirement::new(GemBigInt::from(1), GemBigInt::ZERO);
        assert_eq!(
            GemAmountError::InsufficientBalance {
                asset: asset.clone(),
                requirement: requirement.clone(),
            }
            .display(),
            GemAmountErrorDisplay::InsufficientBalance {
                title: format!("{} ({})", asset.name, asset.symbol)
            }
        );

        let same = Asset { name: asset.symbol.clone(), ..asset };
        assert_eq!(
            GemAmountError::InsufficientBalance { asset: same.clone(), requirement }.display(),
            GemAmountErrorDisplay::InsufficientBalance { title: same.symbol }
        );
    }

    #[test]
    fn test_only_a_below_minimum_amount_offers_an_info_sheet() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let minimum = GemBigInt::from(1000);

        assert_eq!(
            GemAmountError::BelowMinimum {
                asset: asset.clone(),
                minimum: minimum.clone(),
            }
            .display()
            .info(),
            Some(GemInfoTopic::MinimumAmount { asset: asset.clone(), minimum })
        );
        let GemAmountErrorDisplay::BelowMinimum { minimum: formatted, .. } = GemAmountError::BelowMinimum {
            asset,
            minimum: GemBigInt::from(10u64.pow(16)),
        }
        .display() else {
            panic!("a below-minimum error displays its minimum");
        };
        assert_eq!((formatted.value, formatted.unit), (0.01, GemNumberUnit::Symbol { symbol: "ETH".to_string() }));
        assert_eq!(GemAmountErrorDisplay::None.info(), None);
        assert_eq!(GemAmountErrorDisplay::InvalidAmount.info(), None);
        assert_eq!(GemAmountErrorDisplay::InsufficientBalance { title: "ETH".to_string() }.info(), None);
    }
}
