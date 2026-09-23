#![allow(clippy::result_large_err)]

pub mod model;
pub mod rules;

use std::sync::Arc;

use primitives::{Asset, Currency, PerpetualDirection};

pub use model::{GemAmountEarnType, GemAmountEntry, GemAmountError, GemAmountInput, GemAmountInputType, GemAmountMaxEntry, GemAmountPerpetualPosition, GemAmountStakeType, GemAmountTransfer, GemAmountType, GemPerpetualAutoclose};

use crate::config::perpetual_config::{leverage_options, select_leverage};

use crate::models::GemEarnType;
use crate::models::custom_types::GemBigInt;
use crate::models::list::GemListRow;
use crate::services::error::{GemServiceError, required_account};
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::perpetual::rules as perpetual_rules;
use crate::services::preferences::GemPreferencesService;
use crate::services::settings::rules::{GemPickerOption, leverage_option};
use crate::services::stake::GemStakeService;
use crate::services::transfer::rules as transfer_rules;
use crate::services::transfer::{GemRecipient, GemTransferData};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAmountService {
    stake: Arc<GemStakeService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemAmountService {
    #[uniffi::constructor]
    pub fn new(stake: Arc<GemStakeService>, preferences: Arc<GemPreferencesService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { stake, preferences, session }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn perpetual_leverage(&self, max_leverage: u8) -> u8 {
        select_leverage(self.preferences.get_perpetual_leverage(), &leverage_options(max_leverage))
    }

    pub fn perpetual_leverage_options(&self, max_leverage: u8) -> Vec<GemPickerOption> {
        leverage_options(max_leverage).into_iter().map(leverage_option).collect()
    }

    pub fn perpetual_autoclose(&self, price: f64, direction: PerpetualDirection, leverage: u8) -> GemPerpetualAutoclose {
        rules::perpetual_autoclose(price, direction, leverage, self.preferences.get_perpetual_take_profit_percent(), self.preferences.get_perpetual_stop_loss_percent())
    }

    pub fn perpetual_autoclose_row(&self, take_profit: Option<f64>, stop_loss: Option<f64>) -> GemListRow {
        perpetual_rules::amount_autoclose_row(take_profit, stop_loss)
    }

    pub fn perpetual_transfer_data(&self, action: GemPerpetualPositionAction, value: GemBigInt, use_max_amount: bool, leverage: u8, take_profit: Option<f64>, stop_loss: Option<f64>) -> GemTransferData {
        perpetual_rules::order_transfer(action, value, use_max_amount, leverage, take_profit, stop_loss)
    }

    pub fn perpetual_amount_type(&self, action: GemPerpetualPositionAction, leverage: u8) -> GemAmountType {
        rules::perpetual_amount_type(&action, leverage)
    }

    pub fn earn_amount_type(&self, earn_type: GemEarnType) -> GemAmountType {
        rules::earn_amount_type(earn_type)
    }

    pub async fn transfer_data(&self, asset: Asset, transfer: GemAmountTransfer, value: GemBigInt, use_max_amount: bool) -> Result<GemTransferData, GemServiceError> {
        let owner = match transfer {
            GemAmountTransfer::Withdraw => {
                let wallet = self.session.require_current_wallet().await?;
                let account = required_account(&wallet, asset.chain())?;
                Some(GemRecipient::named(account.address.clone(), wallet.name.clone()))
            }
            GemAmountTransfer::Send { .. } | GemAmountTransfer::Deposit => None,
        };
        rules::transfer_data(asset, transfer, owner, value, use_max_amount)
    }

    pub async fn earn_transfer_data(&self, asset: Asset, earn_type: GemEarnType, value: GemBigInt, use_max_amount: bool) -> Result<GemTransferData, GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        let account = required_account(&wallet, asset.chain())?;
        let data = self.stake.get_earn_data(asset.id.clone(), account.address.clone(), value.to_string(), earn_type.clone()).await?;
        Ok(transfer_rules::earn_transfer_data(asset, earn_type, data, value, use_max_amount))
    }
}
