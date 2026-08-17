pub mod address;
pub use address::validate_address;
pub mod coin_type;
pub use coin_type::{coin_type_matches, full_coin_type, is_sui_coin};
mod constants;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "rpc")]
pub use rpc::SuiClient;

#[cfg(feature = "rpc")]
pub mod provider;

pub mod models;

#[cfg(feature = "rpc")]
pub mod transfer_builder;
#[cfg(feature = "rpc")]
pub use transfer_builder::*;

pub mod error;
pub mod gas_budget;
pub mod tx_builder;

#[cfg(feature = "signer")]
pub mod signer;

pub use constants::*;
pub use error::SuiError;
pub use models::ObjectId;
use models::{Coin, OwnedCoins};
use std::error::Error;
use sui_transaction_builder::ObjectInput;
use sui_types::Address;
pub use tx_builder::{decode_transaction, stake::*, transfer::*, validate_and_hash};

pub fn sui_framework_package_address() -> Address {
    ObjectId::from(SUI_FRAMEWORK_PACKAGE_ID).into()
}

pub fn sui_system_package_address() -> Address {
    ObjectId::from(SUI_SYSTEM_PACKAGE_ID).into()
}

pub fn sui_system_state_object_id() -> Address {
    ObjectId::from(SUI_SYSTEM_STATE_OBJECT_ID).into()
}

pub fn sui_clock_object_id() -> Address {
    ObjectId::from(SUI_CLOCK_OBJECT_ID).into()
}

pub fn sui_system_state_object_input() -> ObjectInput {
    ObjectInput::shared(sui_system_state_object_id(), 1, true)
}

pub fn sui_clock_object_input() -> ObjectInput {
    ObjectInput::shared(sui_clock_object_id(), 1, false)
}

pub fn validate_enough_balance(coins: &OwnedCoins<Coin>, amount: u64) -> Option<Box<dyn Error + Send + Sync>> {
    let total = coins.total();
    if total == 0 {
        return Some("no spendable coin objects or address balance".into());
    }
    if total < amount {
        return Some(format!("total amount ({}) is less than amount to send ({})", total, amount).into());
    }
    None
}
