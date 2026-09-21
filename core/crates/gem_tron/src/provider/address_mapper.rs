use primitives::AddressStatus;

use crate::models::account::TronAccount;

pub fn map_address_status(account: &TronAccount) -> Vec<AddressStatus> {
    let address = account.address.as_deref().unwrap_or_default();
    let Some((threshold, keys)) = account.owner_permission.as_ref().and_then(|permission| Some((permission.threshold.unwrap_or(1), permission.keys.as_ref()?))) else {
        return vec![];
    };
    let own_weight: u64 = keys.iter().filter(|key| key.address == address).map(|key| key.weight).sum();
    match own_weight < threshold {
        true => vec![AddressStatus::ExternallyControlled],
        false => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey};

    const ADDRESS: &str = "TCXbgZUdJH14fH82rf36LCpFV53dyXLY3b";
    const OTHER_ADDRESS: &str = "TW6gATnfHd4S65BB4h5Y5Wae2k93rRduLz";

    #[test]
    fn test_an_account_its_owner_key_signs_for_is_its_own() {
        let inactive = TronAccount {
            owner_permission: None,
            active_permission: None,
            ..TronAccount::mock(ADDRESS)
        };
        let co_signed = TronAccount {
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                threshold: Some(1),
                keys: Some(vec![TronAccountPermissionKey::mock(ADDRESS, 1), TronAccountPermissionKey::mock(OTHER_ADDRESS, 1)]),
            }),
            ..TronAccount::mock(ADDRESS)
        };
        let changed_active_permissions = TronAccount {
            active_permission: Some(vec![
                TronAccountPermission {
                    id: Some(2),
                    threshold: 2,
                    keys: Some(vec![TronAccountPermissionKey::mock(OTHER_ADDRESS, 1)]),
                },
                TronAccountPermission {
                    id: Some(3),
                    threshold: 1,
                    keys: Some(vec![TronAccountPermissionKey::mock(OTHER_ADDRESS, 1)]),
                },
            ]),
            ..TronAccount::mock(ADDRESS)
        };

        assert!(map_address_status(&TronAccount::mock(ADDRESS)).is_empty());
        assert!(map_address_status(&inactive).is_empty());
        assert!(map_address_status(&co_signed).is_empty());
        assert!(map_address_status(&changed_active_permissions).is_empty());
    }

    #[test]
    fn test_an_owner_key_below_the_threshold_is_externally_controlled() {
        let replaced = TronAccount {
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                threshold: Some(1),
                keys: Some(vec![TronAccountPermissionKey::mock(OTHER_ADDRESS, 1)]),
            }),
            ..TronAccount::mock(ADDRESS)
        };
        let below_threshold = TronAccount {
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                threshold: Some(2),
                keys: Some(vec![TronAccountPermissionKey::mock(ADDRESS, 1), TronAccountPermissionKey::mock(OTHER_ADDRESS, 1)]),
            }),
            ..TronAccount::mock(ADDRESS)
        };

        assert_eq!(map_address_status(&replaced), vec![AddressStatus::ExternallyControlled]);
        assert_eq!(map_address_status(&below_threshold), vec![AddressStatus::ExternallyControlled]);
    }
}
