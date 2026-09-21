use crate::SUI_COIN_TYPE;
#[cfg(feature = "rpc")]
use crate::models::{BalanceChange, Digest, Effect, Event, GasObject, GasUsed, Owner, OwnerObject, STATUS_SUCCESS, Status};
use crate::models::{Coin, Object, OwnedCoins};
#[cfg(feature = "rpc")]
use num_bigint::{BigInt, BigUint};
#[cfg(feature = "rpc")]
use serde_json::Value;

#[cfg(feature = "rpc")]
pub const TEST_OWNER_ADDRESS: &str = "0x1930a5e729ad95a48e4d9dc2ca8a001f8ed18b20077c083cd6b1d3355a7972a5";

impl Object {
    pub fn mock() -> Self {
        Self {
            object_id: "0xabcdef1234567890abcdef1234567890abcdef12".parse().unwrap(),
            digest: "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8".parse().unwrap(),
            version: 100,
        }
    }
}

impl Coin {
    pub fn mock_sui() -> Self {
        Self {
            coin_type: SUI_COIN_TYPE.to_string(),
            balance: 5_000_000_000,
            object: Object::mock(),
        }
    }
}

impl OwnedCoins<Coin> {
    pub fn mock_sui() -> Self {
        Self::new(SUI_COIN_TYPE.to_string(), vec![Coin::mock_sui()], 0)
    }
}

#[cfg(feature = "rpc")]
impl Owner {
    pub fn mock(address: &str) -> Self {
        Owner::OwnerObject(OwnerObject { address_owner: Some(address.to_string()) })
    }
}

#[cfg(feature = "rpc")]
impl Event {
    pub fn mock(event_type: impl Into<String>, parsed_json: Value) -> Self {
        Self {
            event_type: event_type.into(),
            parsed_json: Some(parsed_json),
            package_id: String::new(),
        }
    }
}

#[cfg(feature = "rpc")]
impl BalanceChange {
    pub fn mock(address: &str, coin_type: &str, amount: i64) -> Self {
        Self {
            owner: Owner::mock(address),
            coin_type: coin_type.to_string(),
            amount: BigInt::from(amount),
        }
    }
}

#[cfg(feature = "rpc")]
impl Digest {
    pub fn mock(events: Vec<Event>, balance_changes: Vec<BalanceChange>) -> Self {
        Self {
            digest: "test".to_string(),
            effects: Effect {
                gas_used: GasUsed {
                    computation_cost: BigUint::from(0u32),
                    storage_cost: BigUint::from(0u32),
                    storage_rebate: BigUint::from(0u32),
                    non_refundable_storage_fee: BigUint::from(0u32),
                },
                status: Status { status: STATUS_SUCCESS.to_string() },
                gas_object: GasObject { owner: Owner::mock(TEST_OWNER_ADDRESS) },
            },
            move_call_packages: Vec::new(),
            balance_changes: Some(balance_changes),
            events,
            timestamp_ms: 1778964551487,
        }
    }
}
