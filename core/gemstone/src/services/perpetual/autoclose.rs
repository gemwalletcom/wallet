use crate::models::custom_types::GemBigInt;
use crate::perpetual::GemPerpetual;
use crate::services::transfer::GemTransferData;
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::{CancelOrderData, PerpetualModifyConfirmData, PerpetualModifyPositionType, TPSLOrderData};
use primitives::{Asset, PerpetualDirection, PerpetualProvider, PerpetualType};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseField {
    pub price: Option<f64>,
    pub original_price: Option<f64>,
    pub formatted_price: Option<String>,
    pub is_valid: bool,
    pub order_id: Option<u64>,
}

#[uniffi::export]
impl GemAutocloseField {
    pub fn has_pending_change(&self) -> bool {
        self.is_cleared() || (self.price.is_some() && self.has_changed())
    }
}

impl GemAutocloseField {
    fn has_changed(&self) -> bool {
        self.price != self.original_price
    }

    fn is_cleared(&self) -> bool {
        self.price.is_none() && self.original_price.is_some()
    }

    fn should_set(&self) -> bool {
        self.is_valid && self.has_changed()
    }

    fn should_update(&self) -> bool {
        self.should_set() || self.is_cleared()
    }

    fn should_cancel(&self) -> bool {
        self.is_cleared() || (self.should_set() && self.original_price.is_some())
    }

    fn is_acceptable(&self) -> bool {
        self.price.is_none() || self.is_valid
    }

    fn cancel(&self, asset_index: i32) -> Option<CancelOrderData> {
        self.should_cancel()
            .then_some(())
            .and(self.order_id)
            .map(|order_id| CancelOrderData { asset_index, order_id })
    }

    fn set_price(&self) -> Option<String> {
        self.should_set().then(|| self.formatted_price.clone()).flatten()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseModify {
    pub direction: PerpetualDirection,
    pub asset_index: i32,
    pub take_profit: GemAutocloseField,
    pub stop_loss: GemAutocloseField,
}

#[uniffi::export]
impl GemAutocloseModify {
    pub fn can_build(&self) -> bool {
        self.take_profit.is_acceptable() && self.stop_loss.is_acceptable() && (self.take_profit.should_update() || self.stop_loss.should_update())
    }

    pub fn transfer(&self, provider: PerpetualProvider, asset: Asset) -> GemTransferData {
        let data = PerpetualModifyConfirmData {
            base_asset: HYPERCORE_PERPETUAL_USDC.clone(),
            asset_index: self.asset_index,
            modify_types: self.build(),
            take_profit_order_id: self.take_profit.order_id,
            stop_loss_order_id: self.stop_loss.order_id,
        };
        GemPerpetual::new(provider).transfer_data(asset, PerpetualType::Modify(data), GemBigInt::ZERO, false)
    }

    pub fn build(&self) -> Vec<PerpetualModifyPositionType> {
        let cancels: Vec<CancelOrderData> = [&self.take_profit, &self.stop_loss]
            .into_iter()
            .filter_map(|field| field.cancel(self.asset_index))
            .collect();
        let mut result = Vec::new();
        if !cancels.is_empty() {
            result.push(PerpetualModifyPositionType::Cancel(cancels));
        }
        if self.take_profit.should_set() || self.stop_loss.should_set() {
            result.push(PerpetualModifyPositionType::Tpsl(TPSLOrderData {
                direction: self.direction.clone(),
                take_profit: self.take_profit.set_price(),
                stop_loss: self.stop_loss.set_price(),
                size: "0".to_string(),
            }));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(price: Option<f64>, original_price: Option<f64>, is_valid: bool, order_id: Option<u64>) -> GemAutocloseField {
        GemAutocloseField {
            price,
            original_price,
            formatted_price: price.map(|price| format!("{price:.1}")),
            is_valid,
            order_id,
        }
    }

    fn modify(take_profit: GemAutocloseField, stop_loss: GemAutocloseField) -> GemAutocloseModify {
        GemAutocloseModify {
            direction: PerpetualDirection::Long,
            asset_index: 5,
            take_profit,
            stop_loss,
        }
    }

    #[test]
    fn test_field_rules() {
        let empty = field(None, None, false, None);
        assert!(!empty.has_pending_change());
        assert!(!field(Some(100.0), Some(100.0), true, None).has_pending_change());
        assert!(field(Some(110.0), Some(100.0), true, None).has_pending_change());
        assert!(field(None, Some(100.0), false, Some(1)).has_pending_change());

        let updated = field(Some(120.0), Some(100.0), true, Some(1));
        assert!(updated.should_set() && updated.should_update() && updated.should_cancel());
        let new = field(Some(120.0), None, true, None);
        assert!(new.should_set() && !new.should_cancel());
        let cleared = field(None, Some(100.0), false, Some(1));
        assert!(!cleared.should_set() && cleared.should_update() && cleared.should_cancel());
        let invalid = field(Some(120.0), Some(100.0), false, Some(1));
        assert!(!invalid.should_set() && !invalid.should_update() && !invalid.is_acceptable());
    }

    #[test]
    fn test_can_build() {
        let none = field(None, None, false, None);
        assert!(modify(field(Some(110.0), Some(100.0), true, None), none.clone()).can_build());
        assert!(!modify(field(Some(100.0), Some(100.0), true, None), field(Some(90.0), Some(90.0), true, None)).can_build());
        assert!(!modify(field(Some(110.0), Some(100.0), false, None), none.clone()).can_build());
        assert!(modify(field(None, Some(100.0), false, None), none.clone()).can_build());
        assert!(modify(none.clone(), field(Some(90.0), None, true, None)).can_build());
        assert!(!modify(none.clone(), none.clone()).can_build());
        assert!(!modify(field(Some(110.0), Some(100.0), false, None), field(Some(80.0), Some(90.0), false, None)).can_build());
        assert!(!modify(field(Some(110.0), Some(100.0), true, None), field(Some(80.0), Some(90.0), false, None)).can_build());
    }

    #[test]
    fn test_build_sets_and_cancels() {
        let none = field(None, None, false, None);
        let set_only = modify(field(Some(110.0), None, true, None), none.clone()).build();
        assert!(matches!(&set_only[..], [PerpetualModifyPositionType::Tpsl(order)] if order.take_profit.as_deref() == Some("110.0") && order.stop_loss.is_none()));

        let cancel_only = modify(field(None, Some(100.0), false, Some(12345)), none.clone()).build();
        assert!(matches!(&cancel_only[..], [PerpetualModifyPositionType::Cancel(cancels)] if cancels.len() == 1 && cancels[0].order_id == 12345 && cancels[0].asset_index == 5));

        let both = modify(field(Some(120.0), Some(100.0), true, Some(12345)), field(Some(80.0), Some(90.0), true, Some(67890))).build();
        assert_eq!(both.len(), 2);
        assert!(matches!(&both[0], PerpetualModifyPositionType::Cancel(cancels) if cancels.len() == 2));
        assert!(
            matches!(&both[1], PerpetualModifyPositionType::Tpsl(order) if order.take_profit.as_deref() == Some("120.0") && order.stop_loss.as_deref() == Some("80.0") && order.size == "0")
        );

        let unchanged_stop_loss = modify(field(Some(120.0), Some(100.0), true, Some(12345)), field(Some(90.0), Some(90.0), true, Some(67890))).build();
        assert!(matches!(&unchanged_stop_loss[1], PerpetualModifyPositionType::Tpsl(order) if order.stop_loss.is_none()));
    }

    #[test]
    fn test_transfer_carries_the_modify_and_the_order_ids_it_replaces() {
        let modify = modify(field(Some(120.0), Some(100.0), true, Some(7)), field(None, None, false, None));
        let transfer = modify.transfer(PerpetualProvider::Hypercore, Asset::from_chain(primitives::Chain::HyperCore));

        let primitives::TransactionInputType::Perpetual {
            perpetual_type: PerpetualType::Modify(data),
            ..
        } = &transfer.input_type
        else {
            panic!("expected a modify transfer")
        };
        assert_eq!((data.asset_index, data.take_profit_order_id, data.stop_loss_order_id), (5, Some(7), None));
        assert_eq!(data.modify_types.len(), 2, "a cancel of the old order and the new tp/sl");
        assert_eq!(transfer.recipient, GemPerpetual::new(PerpetualProvider::Hypercore).recipient());
    }
}
