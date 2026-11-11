use std::collections::HashMap;

use primitives::Account;
use serde_json::Value;

use crate::accounts::{BITCOIN_ACCOUNT_ADDRESSES_PROPERTY, map_bitcoin_account_addresses};

pub fn bitcoin_session_properties(properties: &mut HashMap<String, String>, accounts: &[Account]) {
    let addresses = map_bitcoin_account_addresses(accounts);
    if !addresses.is_empty() {
        properties.insert(BITCOIN_ACCOUNT_ADDRESSES_PROPERTY.to_string(), Value::Array(addresses).to_string());
    }
}
