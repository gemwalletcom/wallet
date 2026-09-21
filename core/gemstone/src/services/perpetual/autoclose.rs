use std::sync::Arc;

use crate::formatted_number::{GemFormattedNumber, GemValueTone, value_tone};
use crate::models::custom_types::GemBigInt;
use crate::percentage::GemPercentageStyle;
use crate::perpetual::GemAutocloseEstimator;
use crate::perpetual::GemPerpetual;
use crate::precision::GemCurrencyStyle;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
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

#[uniffi::export]
pub fn autoclose_field_state(field: GemAutocloseField, estimator: Arc<GemAutocloseEstimator>, shows_errors: bool) -> GemAutocloseFieldState {
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
        let asset_index = self.asset_index.ok_or_else(|| GemServiceError::InvalidInput {
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
    fn can_build(&self) -> bool {
        self.asset_index.is_some() && self.take_profit.is_acceptable() && self.stop_loss.is_acceptable() && (self.take_profit.should_update() || self.stop_loss.should_update())
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
pub struct GemAutocloseViewState {
    pub confirm_enabled: bool,
    pub shows_errors: bool,
    pub entry_price: Option<GemFormattedNumber>,
    pub market_price: GemFormattedNumber,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseSession {
    pub modify: GemAutocloseModify,
    pub policy: GemAutocloseConfirmPolicy,
    pub submit_attempted: bool,
    pub prices: GemAutoclosePrices,
    pub provider: PerpetualProvider,
    pub decimals: i32,
}

fn price_text(price: f64) -> GemFormattedNumber {
    GemFormattedNumber::currency(price, Currency::USD, GemCurrencyStyle::Currency)
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

    pub fn initial_text(&self, tpsl_type: TpslType, decimal_separator: String) -> Option<String> {
        self.field(tpsl_type)
            .original_price
            .map(|price| GemPerpetual::new(self.provider.clone()).format_input_price(price, self.decimals, decimal_separator))
    }

    pub fn view_state(&self) -> GemAutocloseViewState {
        let can_build = self.modify.can_build();
        GemAutocloseViewState {
            confirm_enabled: match (self.policy, self.submit_attempted) {
                (GemAutocloseConfirmPolicy::WhenBuildable, _) | (GemAutocloseConfirmPolicy::UntilSubmitted, true) => can_build,
                (GemAutocloseConfirmPolicy::UntilSubmitted, false) => self.modify.take_profit.has_pending_change() || self.modify.stop_loss.has_pending_change(),
            },
            shows_errors: self.submit_attempted,
            entry_price: self.prices.entry.map(price_text),
            market_price: price_text(self.prices.market),
        }
    }
}

impl GemAutocloseSession {
    pub fn new(modify: GemAutocloseModify, policy: GemAutocloseConfirmPolicy, prices: GemAutoclosePrices, provider: PerpetualProvider, decimals: i32) -> Self {
        Self {
            modify,
            policy,
            submit_attempted: false,
            prices,
            provider,
            decimals,
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
        perpetual.provider,
        asset.decimals,
    )
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_each_platform_gates_confirm_the_way_its_policy_says() {
        let changed = GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), GemAutocloseField::mock(None, None, true, None));
        let prices = GemAutoclosePrices { entry: Some(100.0), market: 110.0 };
        let ios = GemAutocloseSession::new(changed.clone(), GemAutocloseConfirmPolicy::WhenBuildable, prices.clone(), PerpetualProvider::Hypercore, 2);
        let android = GemAutocloseSession::new(changed, GemAutocloseConfirmPolicy::UntilSubmitted, prices, PerpetualProvider::Hypercore, 2);

        assert_eq!(ios.view_state().confirm_enabled, ios.modify.can_build());
        assert_eq!(ios.view_state().entry_price, Some(GemFormattedNumber::currency(100.0, Currency::USD, GemCurrencyStyle::Currency)));
        assert_eq!(ios.view_state().market_price, GemFormattedNumber::currency(110.0, Currency::USD, GemCurrencyStyle::Currency));
        assert!(android.view_state().confirm_enabled, "a pending change is enough before a submit");
        assert_eq!(android.on_submit_attempt().view_state().confirm_enabled, android.modify.can_build());
    }

    #[test]
    fn test_errors_only_show_after_a_submit_attempt() {
        let session = GemAutocloseSession::new(
            GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), None, false, None), GemAutocloseField::mock(None, None, true, None)),
            GemAutocloseConfirmPolicy::UntilSubmitted,
            GemAutoclosePrices { entry: None, market: 110.0 },
            PerpetualProvider::Hypercore,
            2,
        );

        assert!(!session.view_state().shows_errors);
        assert_eq!(session.view_state().entry_price, None);
        assert!(session.on_submit_attempt().view_state().shows_errors);
    }
    use super::*;

    #[test]
    fn test_a_field_estimates_from_any_typed_price_and_names_its_outcome() {
        let estimator = Arc::new(GemAutocloseEstimator::new(100.0, 2.0, PerpetualDirection::Long, 5));
        let profit = autoclose_field_state(GemAutocloseField::mock(Some(120.0), None, true, None), estimator.clone(), true);
        let loss = autoclose_field_state(GemAutocloseField::mock(Some(80.0), None, false, None), estimator.clone(), true);
        let empty = autoclose_field_state(GemAutocloseField::mock(None, None, false, None), estimator.clone(), true);

        assert!(profit.is_profit);
        assert_eq!(profit.tone, GemValueTone::Positive);
        assert!(matches!(profit.estimate, Some(GemLocalizedText::Pnl { .. })), "a sized position names the amount and the percent");
        assert!(!loss.is_profit, "a take profit under the market still estimates, as a loss");
        assert_eq!(loss.tone, GemValueTone::Negative);
        assert_eq!(empty.estimate, None);
        assert_eq!(empty.tone, GemValueTone::Neutral);
        assert!(profit.suggestions.iter().all(|value| value.unit == crate::formatted_number::GemNumberUnit::Percent));
        assert_eq!(
            autoclose_field_state(GemAutocloseField::mock(Some(80.0), None, false, None), estimator, false).validation,
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
    fn test_can_build() {
        let none = GemAutocloseField::mock(None, None, false, None);
        assert!(GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), none.clone()).can_build());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(100.0), Some(100.0), true, None), GemAutocloseField::mock(Some(90.0), Some(90.0), true, None)).can_build());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), false, None), none.clone()).can_build());
        assert!(GemAutocloseModify::mock(GemAutocloseField::mock(None, Some(100.0), false, None), none.clone()).can_build());
        assert!(GemAutocloseModify::mock(none.clone(), GemAutocloseField::mock(Some(90.0), None, true, None)).can_build());
        assert!(!GemAutocloseModify::mock(none.clone(), none.clone()).can_build());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), false, None), GemAutocloseField::mock(Some(80.0), Some(90.0), false, None)).can_build());
        assert!(!GemAutocloseModify::mock(GemAutocloseField::mock(Some(110.0), Some(100.0), true, None), GemAutocloseField::mock(Some(80.0), Some(90.0), false, None)).can_build());
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
    }

    #[test]
    fn test_a_perpetual_without_an_asset_index_can_never_confirm() {
        let perpetual = Perpetual {
            identifier: "BTC".to_string(),
            ..Perpetual::mock()
        };

        let session = autoclose_session(perpetual, Asset::from_chain(primitives::Chain::HyperCore), PerpetualPosition::mock()).on_price(TpslType::TakeProfit, Some(500.0));

        assert_eq!(session.modify.asset_index, None);
        assert!(!session.on_submit_attempt().view_state().confirm_enabled, "an unparsed identifier must never reach a market");
        assert!(session.modify.transfer(PerpetualProvider::Hypercore, Asset::from_chain(primitives::Chain::HyperCore)).is_err());
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
