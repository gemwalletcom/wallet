use primitives::{AddressStatus, SignerError};

use crate::{AccountAddress, models::Account};

pub fn map_address_status(address: &str, account: &Account) -> Result<Vec<AddressStatus>, SignerError> {
    match AccountAddress::from_hex(&account.authentication_key)? == AccountAddress::from_hex(address)? {
        true => Ok(vec![]),
        false => Ok(vec![AddressStatus::ExternallyControlled]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_account_with_its_original_key_is_its_own() {
        let short_form = Account {
            authentication_key: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            ..Account::mock()
        };

        assert!(map_address_status(&Account::mock().authentication_key, &Account::mock()).unwrap().is_empty());
        assert!(map_address_status("0x1", &short_form).unwrap().is_empty());
    }

    #[test]
    fn test_a_rotated_authentication_key_is_externally_controlled() {
        let account = Account {
            authentication_key: "0xf3a9bd8f8f9c7e07bef7d0a841e2165b6ed7a6767934975684f982d1c4b78581".to_string(),
            ..Account::mock()
        };

        assert_eq!(
            map_address_status("0xffd7f0d7b24ba690657dd2aa7ccb754c4be938e4fb970c72eef966d664bd1a45", &account).unwrap(),
            vec![AddressStatus::ExternallyControlled]
        );
    }
}
