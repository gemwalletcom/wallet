use num_bigint::BigUint;
use serde_json::Value;

use crate::AccountAddress;
use crate::models::{DelegationPoolStake, RawTransaction};
use crate::signer::{EntryFunctionPayload, build_raw_transaction};

impl RawTransaction {
    pub fn mock() -> Self {
        let address = "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef";
        let payload = EntryFunctionPayload {
            payload_type: "entry_function_payload".to_string(),
            function: "0x1::aptos_account::transfer".to_string(),
            type_arguments: Vec::new(),
            arguments: vec![Value::String(address.to_string()), Value::String("100".to_string())],
        };
        let entry_function = payload.to_entry_function(Some(&["address", "u64"])).unwrap();
        build_raw_transaction(AccountAddress::from_hex(address).unwrap(), 1, entry_function, 1500, 100, 1700000000, 1)
    }
}

impl DelegationPoolStake {
    pub fn mock(active: u32, inactive: u32, pending_inactive: u32) -> Self {
        Self {
            active: BigUint::from(active),
            inactive: BigUint::from(inactive),
            pending_inactive: BigUint::from(pending_inactive),
        }
    }
}
