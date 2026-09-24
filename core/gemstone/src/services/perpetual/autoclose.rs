use crate::formatted_number::{GemFormattedNumber, GemValueTone, value_tone};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::percentage::GemPercentageStyle;
use crate::perpetual::GemAutocloseEstimator;
use crate::perpetual::GemPerpetual;
use crate::precision::GemCurrencyStyle;
use crate::services::amount::rules::plain_number;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::perpetual::model::GemPerpetualPositionRow;
use crate::services::transfer::GemTransferData;
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, TPSLOrderData};
use primitives::{Asset, AutocloseValidation, AutocloseValidator, Currency, Perpetual, PerpetualDirection, PerpetualPosition, PerpetualProvider, PerpetualType, TpslType};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseField {
    pub tpsl_type: TpslType,
    pub price: Option<f64>,
    pub original_price: Option<f64>,
    pub formatted_price: Option<String>,
    pub validation: AutocloseValidation,
    pub order_id: Option<u64>,
}

impl GemAutocloseField {
    fn has_pending_change(&self) -> bool {
        self.is_cleared() || (self.price.is_some() && self.has_changed())
    }

    fn is_valid(&self) -> bool {
        self.price.is_some() && self.validation == AutocloseValidation::Valid
    }

    fn has_changed(&self) -> bool {
        self.price != self.original_price
    }

    fn is_cleared(&self) -> bool {
        self.price.is_none() && self.original_price.is_some()
    }

    fn should_set(&self) -> bool {
        self.is_valid() && self.has_changed()
    }

    fn should_update(&self) -> bool {
        self.should_set() || self.is_cleared()
    }

    fn should_cancel(&self) -> bool {
        self.is_cleared() || (self.should_set() && self.original_price.is_some())
    }

    fn is_acceptable(&self) -> bool {
        self.price.is_none() || self.is_valid()
    }

    fn cancel(&self, asset_index: i32) -> Option<CancelOrderData> {
        self.should_cancel().then_some(()).and(self.order_id).map(|order_id| CancelOrderData { asset_index, order_id })
    }

    fn set_price(&self) -> Option<String> {
        self.should_set().then(|| self.formatted_price.clone()).flatten()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseFieldState {
    pub tpsl_type: TpslType,
    pub is_profit: bool,
    pub estimate: Option<GemLocalizedText>,
    pub tone: GemValueTone,
    pub suggestions: Vec<GemFormattedNumber>,
    pub validation: AutocloseValidation,
}

fn autoclose_field_state(field: GemAutocloseField, estimator: &GemAutocloseEstimator, shows_errors: bool) -> GemAutocloseFieldState {
    let estimate = field.price.map(|price| {
        let percent = GemFormattedNumber::percentage(estimator.roe(price), GemPercentageStyle::Signed);
        match estimator.has_size() {
            true => GemLocalizedText::Pnl {
                amount: GemFormattedNumber::signed_currency(estimator.pnl(price), Currency::USD, GemCurrencyStyle::Currency),
                percent,
            },
            false => GemLocalizedText::Number { number: percent },
        }
    });
    GemAutocloseFieldState {
        tpsl_type: field.tpsl_type,
        is_profit: estimator.is_profit(field.price, field.tpsl_type),
        estimate,
        tone: field.price.map(|price| value_tone(estimator.roe(price))).unwrap_or(GemValueTone::Neutral),
        suggestions: estimator
            .percent_suggestions()
            .into_iter()
            .map(|percent| GemFormattedNumber::percentage(percent as f64, GemPercentageStyle::UnsignedCompact))
            .collect(),
        validation: match shows_errors {
            true => field.validation,
            false => AutocloseValidation::Valid,
        },
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseModify {
    pub direction: PerpetualDirection,
    pub asset_index: Option<i32>,
    pub take_profit: GemAutocloseField,
    pub stop_loss: GemAutocloseField,
}

#[uniffi::export]
impl GemAutocloseModify {
    pub fn transfer(&self, provider: PerpetualProvider, asset: Asset) -> Result<GemTransferData, GemServiceError> {
        let asset_index = self.asset_index.ok_or_else(|| GemServiceError::Core {
            msg: "perpetual has no asset index".to_string(),
        })?;
        let data = PerpetualModifyConfirmData {
            base_asset: HYPERCORE_PERPETUAL_USDC.clone(),
            asset_index,
            modify_types: self.build(asset_index),
            take_profit_order_id: self.take_profit.order_id,
            stop_loss_order_id: self.stop_loss.order_id,
        };
        Ok(GemPerpetual::new(provider).transfer_data(asset, PerpetualType::Modify { data }, GemBigInt::ZERO, false))
    }
}

impl GemAutocloseModify {
    fn is_complete(&self) -> bool {
        self.take_profit.is_acceptable() && self.stop_loss.is_acceptable() && (self.take_profit.should_update() || self.stop_loss.should_update())
    }

    fn build(&self, asset_index: i32) -> Vec<PerpetualModifyPositionType> {
        let cancels: Vec<CancelOrderData> = [&self.take_profit, &self.stop_loss].into_iter().filter_map(|field| field.cancel(asset_index)).collect();
        let mut result = Vec::new();
        if !cancels.is_empty() {
            result.push(PerpetualModifyPositionType::Cancel { orders: cancels });
        }
        if self.take_profit.should_set() || self.stop_loss.should_set() {
            result.push(PerpetualModifyPositionType::Tpsl {
                order: TPSLOrderData {
                    direction: self.direction.clone(),
                    take_profit: self.take_profit.set_price(),
                    stop_loss: self.stop_loss.set_price(),
                    size: "0".to_string(),
                },
            });
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAutocloseConfirmPolicy {
    WhenBuildable,
    UntilSubmitted,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutoclosePrices {
    pub entry: Option<f64>,
    pub market: f64,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseEstimate {
    pub entry_price: f64,
    pub size: f64,
    pub leverage: u8,
    pub is_open: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseViewState {
    pub confirm_enabled: bool,
    pub take_profit: GemAutocloseFieldState,
    pub stop_loss: GemAutocloseFieldState,
    pub price_rows: Vec<GemListRow>,
    pub position_row: Option<GemPerpetualPositionRow>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseSession {
    pub modify: GemAutocloseModify,
    pub policy: GemAutocloseConfirmPolicy,
    pub submit_attempted: bool,
    pub prices: GemAutoclosePrices,
    pub provider: PerpetualProvider,
    pub decimals: i32,
    pub position_row: Option<GemPerpetualPositionRow>,
    pub estimate: GemAutocloseEstimate,
}

fn price_row(title: GemListRowTitle, price: f64) -> GemListRow {
    GemListRow::Amount {
        title,
        amount: GemFormattedNumber::currency(price, Currency::USD, GemCurrencyStyle::Currency),
        info: None,
    }
}

#[uniffi::export]
impl GemAutocloseSession {
    pub fn on_submit_attempt(&self) -> Self {
        Self { submit_attempted: true, ..self.clone() }
    }

    pub fn on_price(&self, tpsl_type: TpslType, price: Option<f64>) -> Self {
        let field = self.priced(self.field(tpsl_type).clone(), price);
        let modify = match tpsl_type {
            TpslType::TakeProfit => GemAutocloseModify { take_profit: field, ..self.modify.clone() },
            TpslType::StopLoss => GemAutocloseModify { stop_loss: field, ..self.modify.clone() },
        };
        Self {
            modify,
            submit_attempted: false,
            ..self.clone()
        }
    }

    pub fn on_percent_selected(&self, tpsl_type: TpslType, percent: i32) -> Self {
        let price = self.estimator().target_price_from_roe(percent, tpsl_type);
        self.on_price(tpsl_type, Some(price))
    }

    pub fn input_text(&self, tpsl_type: TpslType, decimal_separator: String) -> Option<String> {
        self.field(tpsl_type).price.map(|price| GemPerpetual::new(self.provider.clone()).format_input_price(price, self.decimals, decimal_separator))
    }

    pub fn view_state(&self) -> GemAutocloseViewState {
        let estimator = self.estimator();
        GemAutocloseViewState {
            take_profit: autoclose_field_state(self.modify.take_profit.clone(), &estimator, self.submit_attempted),
            stop_loss: autoclose_field_state(self.modify.stop_loss.clone(), &estimator, self.submit_attempted),
            confirm_enabled: match (self.policy, self.submit_attempted) {
                (GemAutocloseConfirmPolicy::WhenBuildable, _) => self.modify.is_complete(),
                (GemAutocloseConfirmPolicy::UntilSubmitted, true) => self.modify.is_complete(),
                (GemAutocloseConfirmPolicy::UntilSubmitted, false) => self.modify.take_profit.has_pending_change() || self.modify.stop_loss.has_pending_change(),
            },
            price_rows: [self.prices.entry.map(|price| price_row(GemListRowTitle::EntryPrice, price)), Some(price_row(GemListRowTitle::MarketPrice, self.prices.market))]
                .into_iter()
                .flatten()
                .collect(),
            position_row: self.position_row.clone(),
        }
    }
}

impl GemAutocloseSession {
    pub fn new(modify: GemAutocloseModify, policy: GemAutocloseConfirmPolicy, prices: GemAutoclosePrices, provider: PerpetualProvider, decimals: i32, position_row: Option<GemPerpetualPositionRow>, estimate: GemAutocloseEstimate) -> Self {
        Self {
            estimate,
            modify,
            policy,
            submit_attempted: false,
            prices,
            provider,
            decimals,
            position_row,
        }
    }

    fn estimator(&self) -> GemAutocloseEstimator {
        let direction = self.modify.direction.clone();
        match self.estimate.is_open {
            true => GemAutocloseEstimator::for_open(self.estimate.entry_price, self.estimate.size, self.estimate.leverage, direction),
            false => GemAutocloseEstimator::new(self.estimate.entry_price, self.estimate.size, direction, self.estimate.leverage),
        }
    }

    fn field(&self, tpsl_type: TpslType) -> &GemAutocloseField {
        match tpsl_type {
            TpslType::TakeProfit => &self.modify.take_profit,
            TpslType::StopLoss => &self.modify.stop_loss,
        }
    }

    fn priced(&self, field: GemAutocloseField, price: Option<f64>) -> GemAutocloseField {
        GemAutocloseField {
            price,
            formatted_price: price.map(|price| GemPerpetual::new(self.provider.clone()).format_price(price, self.decimals)),
            validation: AutocloseValidator::new(field.tpsl_type, self.modify.direction.clone(), self.prices.market).validate_optional(price),
            ..field
        }
    }
}

#[uniffi::export]
pub fn autoclose_session(perpetual: Perpetual, asset: Asset, position: PerpetualPosition) -> GemAutocloseSession {
    let trigger = |tpsl_type: TpslType, order: Option<&primitives::PerpetualTriggerOrder>| GemAutocloseField {
        tpsl_type,
        price: order.map(|order| order.price),
        original_price: order.map(|order| order.price),
        formatted_price: None,
        validation: AutocloseValidation::Valid,
        order_id: order.and_then(|order| order.order_id.parse().ok()),
    };
    let modify = GemAutocloseModify {
        direction: position.direction.clone(),
        asset_index: super::rules::asset_index(&perpetual).ok(),
        take_profit: trigger(TpslType::TakeProfit, position.take_profit.as_ref()),
        stop_loss: trigger(TpslType::StopLoss, position.stop_loss.as_ref()),
    };
    GemAutocloseSession::new(
        modify,
        GemAutocloseConfirmPolicy::UntilSubmitted,
        GemAutoclosePrices {
            entry: Some(position.entry_price),
            market: perpetual.price,
        },
        perpetual.provider.clone(),
        asset.decimals,
        Some(super::rules::position_row(&perpetual, &asset, &position)),
        GemAutocloseEstimate {
            entry_price: position.entry_price,
            size: position.size,
            leverage: position.leverage,
            is_open: false,
        },
    )
}

#[uniffi::export]
pub fn autoclose_open_session(direction: PerpetualDirection, market_price: f64, size: f64, leverage: u8, decimals: i32, provider: PerpetualProvider) -> GemAutocloseSession {
    let empty = |tpsl_type: TpslType| GemAutocloseField {
        tpsl_type,
        price: None,
        original_price: None,
        formatted_price: None,
        validation: AutocloseValidation::Valid,
        order_id: None,
    };
    GemAutocloseSession::new(
        GemAutocloseModify {
            direction,
            asset_index: None,
            take_profit: empty(TpslType::TakeProfit),
            stop_loss: empty(TpslType::StopLoss),
        },
        GemAutocloseConfirmPolicy::WhenBuildable,
        GemAutoclosePrices { entry: None, market: market_price },
        provider,
        decimals,
        None,
        GemAutocloseEstimate {
            entry_price: market_price,
            size,
            leverage,
            is_open: true,
        },
    )
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseDraftField {
    pub value: Option<String>,
    pub is_edited: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseDraft {
    pub take_profit: GemAutocloseDraftField,
    pub stop_loss: GemAutocloseDraftField,
}

#[uniffi::export]
pub fn autoclose_draft(take_profit: Option<String>, stop_loss: Option<String>) -> GemAutocloseDraft {
    GemAutocloseDraft {
        take_profit: GemAutocloseDraftField { value: take_profit, is_edited: false },
        stop_loss: GemAutocloseDraftField { value: stop_loss, is_edited: false },
    }
}

#[uniffi::export]
impl GemAutocloseDraft {
    pub fn on_defaults(&self, take_profit: Option<String>, stop_loss: Option<String>) -> GemAutocloseDraft {
        GemAutocloseDraft {
            take_profit: self.take_profit.on_default(take_profit),
            stop_loss: self.stop_loss.on_default(stop_loss),
        }
    }

    pub fn on_edited(&self, tpsl_type: TpslType, value: Option<String>) -> GemAutocloseDraft {
        match tpsl_type {
            TpslType::TakeProfit => GemAutocloseDraft {
                take_profit: self.take_profit.on_edited(value),
                ..self.clone()
            },
            TpslType::StopLoss => GemAutocloseDraft {
                stop_loss: self.stop_loss.on_edited(value),
                ..self.clone()
            },
        }
    }
}

impl GemAutocloseDraft {
    pub fn prices(&self, decimal_separator: &str) -> (Option<f64>, Option<f64>) {
        (self.take_profit.price(decimal_separator), self.stop_loss.price(decimal_separator))
    }
}

impl GemAutocloseDraftField {
    fn price(&self, decimal_separator: &str) -> Option<f64> {
        self.value.as_ref().and_then(|text| plain_number(decimal_separator, text).parse().ok())
    }

    fn on_default(&self, value: Option<String>) -> GemAutocloseDraftField {
        match self.is_edited {
            true => self.clone(),
            false => GemAutocloseDraftField { value, is_edited: false },
        }
    }

    fn on_edited(&self, value: Option<String>) -> GemAutocloseDraftField {
        let value = value.filter(|text| !text.trim().is_empty());
        match value == self.value {
            true => self.clone(),
            false => GemAutocloseDraftField { value, is_edited: true },
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_an_open_position_starts_empty_and_prices_against_the_snapshot_it_was_given() {
        let session = autoclose_open_session(PerpetualDirection::Long, 100.0, 1.0, 5, 2, PerpetualProvider::Hypercore);
        let state = session.view_state();

        assert!(!state.confirm_enabled, "nothing has been entered yet");
        assert_eq!(state.price_rows, vec![price_row(GemListRowTitle::MarketPrice, 100.0)], "an unopened position has no entry price");
        assert_eq!(session.input_text(TpslType::TakeProfit, ".".to_string()), None);

        let above = session.on_price(TpslType::TakeProfit, Some(120.0));
        assert_eq!(above.modify.take_profit.validation, AutocloseValidation::Valid);
        assert!(above.view_state().confirm_enabled);

        let below = session.on_price(TpslType::TakeProfit, Some(80.0));
        assert_ne!(below.modify.take_profit.validation, AutocloseValidation::Valid, "a long takes profit above the market it was opened against");
        assert!(!below.view_state().confirm_enabled);
    }

    #[test]
    fn test_each_platform_gates_confirm_the_way_its_policy_says() {
        let changed = GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), GemAutocloseField::mock(None, None, true, None));
        let prices = GemAutoclosePrices { entry: Some(100.0), market: 110.0 };
        let ios = GemAutocloseSession::new(changed.clone(), GemAutocloseConfirmPolicy::WhenBuildable, prices.clone(), PerpetualProvider::Hypercore, 2, None, GemAutocloseEstimate::mock());
        let android = GemAutocloseSession::new(changed, GemAutocloseConfirmPolicy::UntilSubmitted, prices, PerpetualProvider::Hypercore, 2, None, GemAutocloseEstimate::mock());

        assert_eq!(ios.view_state().confirm_enabled, ios.modify.is_complete());
        assert_eq!(
            ios.view_state().price_rows,
            vec![
                GemListRow::Amount {
                    title: GemListRowTitle::EntryPrice,
                    amount: GemFormattedNumber::currency(100.0, Currency::USD, GemCurrencyStyle::Currency),
                    info: None,
                },
                GemListRow::Amount {
                    title: GemListRowTitle::MarketPrice,
                    amount: GemFormattedNumber::currency(110.0, Currency::USD, GemCurrencyStyle::Currency),
                    info: None,
                },
            ]
        );
        assert!(android.view_state().confirm_enabled, "a pending change is enough before a submit");
        assert_eq!(android.on_submit_attempt().view_state().confirm_enabled, android.modify.is_complete());
    }

    #[test]
    fn test_a_position_without_an_asset_index_confirms_and_the_transfer_says_why() {
        let modify = GemAutocloseModify {
            asset_index: None,
            ..GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), GemAutocloseField::mock(None, None, true, None))
        };
        let session = GemAutocloseSession::new(
            modify,
            GemAutocloseConfirmPolicy::UntilSubmitted,
            GemAutoclosePrices { entry: Some(100.0), market: 105.0 },
            PerpetualProvider::Hypercore,
            2,
            None,
            GemAutocloseEstimate::mock(),
        )
        .on_submit_attempt();

        assert!(session.view_state().confirm_enabled, "the tap reaches the transfer, which reports the problem");
        let error = session.modify.transfer(PerpetualProvider::Hypercore, Asset::mock()).unwrap_err();
        assert_eq!(error.text(), crate::services::error_text::GemErrorText::Unknown, "the user sees the generic error, not the internal reason");
    }

    #[test]
    fn test_a_leverage_change_refreshes_untouched_defaults_and_keeps_edited_prices() {
        let draft = autoclose_draft(Some("110".to_string()), Some("90".to_string()));

        let edited = draft.on_edited(TpslType::TakeProfit, Some("120".to_string()));
        let refreshed = edited.on_defaults(Some("130".to_string()), Some("80".to_string()));

        assert_eq!(refreshed.take_profit.value.as_deref(), Some("120"), "an edited price survives a leverage change");
        assert_eq!(refreshed.stop_loss.value.as_deref(), Some("80"), "an untouched default follows the leverage");
        assert_eq!(draft.on_edited(TpslType::StopLoss, Some("90".to_string())), draft, "confirming the same price is not an edit");
        assert_eq!(draft.on_edited(TpslType::StopLoss, Some(" ".to_string())).stop_loss, GemAutocloseDraftField { value: None, is_edited: true });
    }

    #[test]
    fn test_a_percent_pick_prices_the_field_and_its_state_comes_with_the_view() {
        let session = autoclose_open_session(PerpetualDirection::Long, 100.0, 1.0, 5, 2, PerpetualProvider::Hypercore);

        let picked = session.on_percent_selected(TpslType::TakeProfit, 50);
        assert!(picked.modify.take_profit.price.is_some_and(|price| price > 100.0));
        assert!(picked.input_text(TpslType::TakeProfit, ".".to_string()).is_some());
        let state = picked.view_state();
        assert!(state.take_profit.estimate.is_some());
        assert!(state.stop_loss.estimate.is_none());
        assert_eq!(state.take_profit.tpsl_type, TpslType::TakeProfit);
    }

    #[test]
    fn test_a_draft_reads_its_prices_in_the_user_locale() {
        let draft = autoclose_draft(Some("1 234,5".to_string()), None).on_edited(TpslType::StopLoss, Some("98,25".to_string()));

        assert_eq!(draft.prices(","), (Some(1234.5), Some(98.25)));
        assert_eq!(autoclose_draft(Some("abc".to_string()), None).prices("."), (None, None));
    }

    #[test]
    fn test_errors_only_show_after_a_submit_attempt() {
        let session = GemAutocloseSession::new(
            GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), None, false, None), GemAutocloseField::mock(None, None, true, None)),
            GemAutocloseConfirmPolicy::UntilSubmitted,
            GemAutoclosePrices { entry: None, market: 110.0 },
            PerpetualProvider::Hypercore,
            2,
            None,
            GemAutocloseEstimate::mock(),
        );

        assert_eq!(session.view_state().take_profit.validation, AutocloseValidation::Valid, "an invalid price stays quiet before a submit attempt");
        assert_eq!(session.view_state().price_rows.len(), 1, "no entry price without a position");
        assert_eq!(session.on_submit_attempt().view_state().take_profit.validation, AutocloseValidation::InvalidAmount);
    }
    use super::*;

    #[test]
    fn test_a_field_estimates_from_any_typed_price_and_names_its_outcome() {
        let estimator = GemAutocloseEstimator::new(100.0, 2.0, PerpetualDirection::Long, 5);
        let profit = autoclose_field_state(GemAutocloseField::mock(Some(120.0), None, true, None), &estimator, true);
        let loss = autoclose_field_state(GemAutocloseField::mock(Some(80.0), None, false, None), &estimator, true);
        let empty = autoclose_field_state(GemAutocloseField::mock(None, None, false, None), &estimator, true);

        assert!(profit.is_profit);
        assert_eq!(profit.tone, GemValueTone::Positive);
        assert!(matches!(profit.estimate, Some(GemLocalizedText::Pnl { .. })), "a sized position names the amount and the percent");
        assert!(!loss.is_profit, "a take profit under the market still estimates, as a loss");
        assert_eq!(loss.tone, GemValueTone::Negative);
        assert_eq!(empty.estimate, None);
        assert_eq!(empty.tone, GemValueTone::Neutral);
        assert!(profit.suggestions.iter().all(|value| value.unit == crate::formatted_number::GemNumberUnit::Percent));
        assert_eq!(
            autoclose_field_state(GemAutocloseField::mock(Some(80.0), None, false, None), &estimator, false).validation,
            AutocloseValidation::Valid,
            "errors wait for a submit attempt"
        );
    }

    #[test]
    fn test_field_rules() {
        let empty = GemAutocloseField::mock(None, None, false, None);
        assert!(!empty.has_pending_change());
        assert!(!GemAutocloseField::mock(Some(100.0), Some(100.0), true, None).has_pending_change());
        assert!(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None).has_pending_change());
        assert!(GemAutocloseField::mock(None, Some(100.0), false, Some(1)).has_pending_change());

        let updated = GemAutocloseField::mock(Some(120.0), Some(100.0), true, Some(1));
        assert!(updated.should_set() && updated.should_update() && updated.should_cancel());
        let new = GemAutocloseField::mock(Some(120.0), None, true, None);
        assert!(new.should_set() && !new.should_cancel());
        let cleared = GemAutocloseField::mock(None, Some(100.0), false, Some(1));
        assert!(!cleared.should_set() && cleared.should_update() && cleared.should_cancel());
        let invalid = GemAutocloseField::mock(Some(120.0), Some(100.0), false, Some(1));
        assert!(!invalid.should_set() && !invalid.should_update() && !invalid.is_acceptable());
    }

    #[test]
    fn test_is_complete() {
        let none = GemAutocloseField::mock(None, None, false, None);
        assert!(GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), none.clone()).is_complete());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(100.0), Some(100.0), true, None), GemAutocloseField::mock(Some(90.0), Some(90.0), true, None)).is_complete());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), false, None), none.clone()).is_complete());
        assert!(GemAutocloseModify::mock(GemAutocloseField::mock(None, Some(100.0), false, None), none.clone()).is_complete());
        assert!(GemAutocloseModify::mock(none.clone(), GemAutocloseField::mock(Some(90.0), None, true, None)).is_complete());
        assert!(!GemAutocloseModify::mock(none.clone(), none.clone()).is_complete());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), false, None), GemAutocloseField::mock(Some(80.0), Some(90.0), false, None)).is_complete());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), GemAutocloseField::mock(Some(80.0), Some(90.0), false, None)).is_complete());
    }

    #[test]
    fn test_build_sets_and_cancels() {
        let none = GemAutocloseField::mock(None, None, false, None);
        let set_only = GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), None, true, None), none.clone()).build(5);
        assert!(matches!(&set_only[..], [PerpetualModifyPositionType::Tpsl { order }] if order.take_profit.as_deref() == Some("110.0") && order.stop_loss.is_none()));

        let cancel_only = GemAutocloseModify::mock(GemAutocloseField::mock(None, Some(100.0), false, Some(12345)), none.clone()).build(5);
        assert!(matches!(&cancel_only[..], [PerpetualModifyPositionType::Cancel { orders: cancels }] if cancels.len() == 1 && cancels[0].order_id == 12345 && cancels[0].asset_index == 5));

        let both = GemAutocloseModify::mock(GemAutocloseField::mock(Some(120.0), Some(100.0), true, Some(12345)), GemAutocloseField::mock(Some(80.0), Some(90.0), true, Some(67890))).build(5);
        assert_eq!(both.len(), 2);
        assert!(matches!(&both[0], PerpetualModifyPositionType::Cancel { orders: cancels } if cancels.len() == 2));
        assert!(matches!(&both[1], PerpetualModifyPositionType::Tpsl { order } if order.take_profit.as_deref() == Some("120.0") && order.stop_loss.as_deref() == Some("80.0") && order.size == "0"));

        let unchanged_stop_loss = GemAutocloseModify::mock(GemAutocloseField::mock(Some(120.0), Some(100.0), true, Some(12345)), GemAutocloseField::mock(Some(90.0), Some(90.0), true, Some(67890))).build(5);
        assert!(matches!(&unchanged_stop_loss[1], PerpetualModifyPositionType::Tpsl { order } if order.stop_loss.is_none()));
    }

    #[test]
    fn test_transfer_carries_the_modify_and_the_order_ids_it_replaces() {
        let modify = GemAutocloseModify::mock(GemAutocloseField::mock(Some(120.0), Some(100.0), true, Some(7)), GemAutocloseField::mock(None, None, false, None));
        let transfer = modify.transfer(PerpetualProvider::Hypercore, Asset::from_chain(primitives::Chain::HyperCore)).unwrap();

        let primitives::TransactionInputType::Perpetual {
            perpetual_type: PerpetualType::Modify { data },
            ..
        } = &transfer.input_type
        else {
            panic!("expected a modify transfer")
        };
        assert_eq!((data.asset_index, data.take_profit_order_id, data.stop_loss_order_id), (5, Some(7), None));
        assert_eq!(data.modify_types.len(), 2, "a cancel of the old order and the new tp/sl");
        assert_eq!(
            data.base_asset.id,
            super::super::rules::collateral_asset_id(primitives::Chain::HyperCore).unwrap(),
            "a modify moves no funds but still names its collateral, which is the balance row the confirm load reads"
        );
        assert_eq!(transfer.recipient, GemPerpetual::new(PerpetualProvider::Hypercore).recipient());
    }

    #[test]
    fn test_a_modify_without_an_asset_index_never_becomes_a_transfer() {
        let modify = GemAutocloseModify {
            asset_index: None,
            ..GemAutocloseModify::mock(GemAutocloseField::mock(Some(120.0), Some(100.0), true, Some(7)), GemAutocloseField::mock(None, None, false, None))
        };

        assert!(
            modify.transfer(PerpetualProvider::Hypercore, Asset::from_chain(primitives::Chain::HyperCore)).is_err(),
            "an unparsed perpetual identifier must never fall back to market 0"
        );
    }

    #[test]
    fn test_a_session_seeded_from_a_position_carries_its_triggers_and_market() {
        let perpetual = Perpetual {
            identifier: "42".to_string(),
            price: 110.0,
            ..Perpetual::mock()
        };
        let position = PerpetualPosition {
            take_profit: Some(primitives::PerpetualTriggerOrder {
                price: 120.0,
                order_type: primitives::PerpetualOrderType::Market,
                order_id: "7".to_string(),
            }),
            stop_loss: None,
            ..PerpetualPosition::mock()
        };

        let session = autoclose_session(perpetual, Asset::from_chain(primitives::Chain::HyperCore), position.clone());

        assert_eq!(session.modify.asset_index, Some(42));
        assert_eq!(session.modify.take_profit.original_price, Some(120.0));
        assert_eq!(session.modify.take_profit.order_id, Some(7));
        assert_eq!(session.modify.stop_loss.original_price, None);
        assert_eq!(session.prices.market, 110.0);
        assert_eq!(session.prices.entry, Some(position.entry_price));
        assert!(!session.view_state().confirm_enabled, "an untouched form has nothing to submit");
        assert!(session.view_state().position_row.is_some(), "the modify screen shows the position it edits");
        assert!(autoclose_open_session(PerpetualDirection::Long, 100.0, 1.0, 5, 2, PerpetualProvider::Hypercore).view_state().position_row.is_none());
    }

    #[test]
    fn test_a_perpetual_without_an_asset_index_never_builds_an_order() {
        let perpetual = Perpetual {
            identifier: "BTC".to_string(),
            ..Perpetual::mock()
        };

        let session = autoclose_session(perpetual, Asset::from_chain(primitives::Chain::HyperCore), PerpetualPosition::mock()).on_price(TpslType::TakeProfit, Some(500.0));

        assert_eq!(session.modify.asset_index, None);
        assert!(
            session.modify.transfer(PerpetualProvider::Hypercore, Asset::from_chain(primitives::Chain::HyperCore)).is_err(),
            "an unparsed identifier must never reach a market"
        );
    }

    #[test]
    fn test_typing_a_price_revalidates_the_field_in_core() {
        let session = autoclose_session(
            Perpetual {
                identifier: "42".to_string(),
                price: 110.0,
                ..Perpetual::mock()
            },
            Asset::from_chain(primitives::Chain::HyperCore),
            PerpetualPosition {
                direction: PerpetualDirection::Long,
                take_profit: None,
                stop_loss: None,
                ..PerpetualPosition::mock()
            },
        );

        let below = session.on_price(TpslType::TakeProfit, Some(90.0));
        assert_eq!(below.modify.take_profit.validation, AutocloseValidation::TriggerMustBeHigher);
        assert!(!below.on_submit_attempt().view_state().confirm_enabled);

        let above = session.on_price(TpslType::TakeProfit, Some(120.0));
        assert_eq!(above.modify.take_profit.validation, AutocloseValidation::Valid);
        assert_eq!(above.modify.take_profit.formatted_price.as_deref(), Some("120"));
        assert!(above.on_submit_attempt().view_state().confirm_enabled);

        assert_eq!(
            session.on_price(TpslType::TakeProfit, None).modify.take_profit.validation,
            AutocloseValidation::Valid,
            "an empty field is not an invalid amount"
        );
    }
}
