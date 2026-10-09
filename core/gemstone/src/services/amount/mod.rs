#![allow(clippy::result_large_err)]

pub mod model;
pub mod rules;

use std::sync::Arc;

use primitives::{Asset, Currency};

pub use model::{
    GemAmountEarnType, GemAmountEntry, GemAmountError, GemAmountExtras, GemAmountInput, GemAmountInputType, GemAmountLeverage, GemAmountMaxEntry, GemAmountPerpetualPosition, GemAmountRequest, GemAmountStakeType, GemAmountTransfer,
    GemAmountType, GemLeverageSelection,
};
use model::{GemNumberFormat, GemPerpetualAmountSession};

use crate::models::custom_types::GemBigInt;
use crate::models::list::GemListRow;
use crate::services::error::{GemServiceError, required_account};
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::perpetual::rules as perpetual_rules;
use crate::services::preferences::GemPreferencesService;
use crate::services::stake::{GemStakeAmountSelection, GemStakeService};
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

    pub fn new_perpetual_session(&self, action: GemPerpetualPositionAction, format: GemNumberFormat) -> GemPerpetualAmountSession {
        GemPerpetualAmountSession::new(
            action,
            self.preferences.get_perpetual_leverage(),
            self.preferences.get_perpetual_take_profit_percent(),
            self.preferences.get_perpetual_stop_loss_percent(),
            format,
        )
    }

    pub fn extras(&self, request: GemAmountRequest, asset: Asset) -> GemAmountExtras {
        match &request {
            GemAmountRequest::Transfer { .. } => GemAmountExtras::None,
            GemAmountRequest::Stake { input } => match self.stake.stake_amount_selection(asset.chain(), input.clone()) {
                GemStakeAmountSelection::Validator { validator, can_select } => GemAmountExtras::Validator { row: validator, can_select },
                GemStakeAmountSelection::Resource { options, selected } => GemAmountExtras::Resources { options, selected },
            },
            GemAmountRequest::Earn { .. } => match request.amount_type() {
                GemAmountType::Earn { provider, .. } => GemAmountExtras::Provider { row: provider },
                _ => GemAmountExtras::None,
            },
            GemAmountRequest::Perpetual { session } => GemAmountExtras::Perpetual {
                leverage: session.leverage.clone().map(|selection| GemAmountLeverage {
                    selection,
                    direction: session.action.data().direction.clone(),
                }),
                autoclose: session.action.shows_autoclose().then(|| self.perpetual_autoclose_row(&session.action.data().asset, session)),
            },
        }
    }

    pub async fn transfer_data(&self, asset: Asset, request: GemAmountRequest, value: GemBigInt, use_max_amount: bool) -> Result<GemTransferData, GemServiceError> {
        match request {
            GemAmountRequest::Transfer { transfer } => {
                let owner = match transfer {
                    GemAmountTransfer::Withdraw | GemAmountTransfer::Deposit => {
                        let wallet = self.session.require_current_wallet().await?;
                        let account = required_account(&wallet, asset.chain())?;
                        Some(GemRecipient::named(account.address, wallet.name))
                    }
                    GemAmountTransfer::Send { .. } => None,
                };
                rules::transfer_data(asset, transfer, owner, value, use_max_amount)
            }
            GemAmountRequest::Stake { input } => Ok(transfer_rules::stake_transfer_data(asset, input.stake_type(), value, use_max_amount)),
            GemAmountRequest::Earn { earn_type } => {
                let wallet = self.session.require_current_wallet().await?;
                let account = required_account(&wallet, asset.chain())?;
                let data = self.stake.get_earn_data(asset.id.clone(), account.address.clone(), value.to_string(), earn_type.clone()).await?;
                Ok(transfer_rules::earn_transfer_data(asset, earn_type, data, value, use_max_amount))
            }
            GemAmountRequest::Perpetual { session } => {
                let (take_profit, stop_loss) = match session.action.shows_autoclose() {
                    true => session.autoclose.prices(&session.format.decimal_separator),
                    false => (None, None),
                };
                let leverage = session.leverage();
                Ok(perpetual_rules::order_transfer(session.action, value, use_max_amount, leverage, take_profit, stop_loss))
            }
        }
    }
}

impl GemAmountService {
    fn perpetual_autoclose_row(&self, asset: &Asset, session: &GemPerpetualAmountSession) -> GemListRow {
        let (take_profit, stop_loss) = session.autoclose.prices(&session.format.decimal_separator);
        perpetual_rules::amount_autoclose_row(asset, take_profit, stop_loss)
    }
}
