use primitives::AddressStatus;

use crate::models::{AccountAccessKeyList, AccountAccessKeyPermission};

pub(super) fn map_address_status(public_key: &str, access_keys: &AccountAccessKeyList) -> Vec<AddressStatus> {
    let has_full_access = access_keys.keys.iter().any(|entry| {
        entry.public_key == public_key
            && match &entry.access_key.permission {
                AccountAccessKeyPermission::FullAccess => true,
                AccountAccessKeyPermission::FunctionCall(_) => false,
            }
    });
    match has_full_access {
        true => vec![],
        false => vec![AddressStatus::ExternallyControlled],
    }
}
