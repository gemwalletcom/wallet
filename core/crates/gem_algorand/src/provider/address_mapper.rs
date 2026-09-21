use primitives::AddressStatus;

use crate::models::Account;

pub fn map_address_status(address: &str, account: &Account) -> Vec<AddressStatus> {
    match account.auth_addr.as_deref() {
        Some(auth_addr) if auth_addr != address => vec![AddressStatus::ExternallyControlled],
        Some(_) | None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS: &str = "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM";

    #[test]
    fn test_an_account_its_key_signs_for_is_its_own() {
        let rekeyed_to_itself = Account {
            auth_addr: Some(ADDRESS.to_string()),
            ..Account::mock()
        };

        assert!(map_address_status(ADDRESS, &Account::mock()).is_empty());
        assert!(map_address_status(ADDRESS, &rekeyed_to_itself).is_empty());
    }

    #[test]
    fn test_a_rekeyed_account_is_externally_controlled() {
        let account = Account {
            auth_addr: Some("5HRVWKG5CGTCWGAFDA3B32CJVGKZ2RQWQWSEBJYWEOM3DW4P3O3WTSUATE".to_string()),
            ..Account::mock()
        };

        assert_eq!(map_address_status(ADDRESS, &account), vec![AddressStatus::ExternallyControlled]);
    }
}
