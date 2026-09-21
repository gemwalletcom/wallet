use primitives::AddressStatus;

use crate::{constants::ACCOUNT_FLAG_DISABLE_MASTER, models::rpc::AccountInfo};

pub fn map_address_status(account: Option<&AccountInfo>) -> Vec<AddressStatus> {
    match account.and_then(|account| account.flags) {
        Some(flags) if flags & ACCOUNT_FLAG_DISABLE_MASTER != 0 => vec![AddressStatus::ExternallyControlled],
        Some(_) | None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_account_its_master_key_still_signs_for_is_its_own() {
        let flagged_otherwise = AccountInfo {
            flags: Some(0x0080_0000),
            ..AccountInfo::mock_with_balance(1_000_000, 0)
        };

        assert!(map_address_status(Some(&AccountInfo::mock_with_balance(1_000_000, 0))).is_empty());
        assert!(map_address_status(Some(&flagged_otherwise)).is_empty());
        assert!(map_address_status(None).is_empty());
    }

    #[test]
    fn test_a_disabled_master_key_is_externally_controlled() {
        let account = AccountInfo {
            flags: Some(0x0080_0000 | ACCOUNT_FLAG_DISABLE_MASTER),
            ..AccountInfo::mock_with_balance(1_000_000, 0)
        };

        assert_eq!(map_address_status(Some(&account)), vec![AddressStatus::ExternallyControlled]);
    }
}
