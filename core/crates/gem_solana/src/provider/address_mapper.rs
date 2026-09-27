use primitives::AddressStatus;

use crate::{SYSTEM_PROGRAM_ID, models::AccountData};

pub fn map_address_status(account: Option<&AccountData>) -> Vec<AddressStatus> {
    match account {
        Some(account) if account.owner != SYSTEM_PROGRAM_ID => vec![AddressStatus::ExternallyControlled],
        Some(_) | None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_system_owned_account_is_its_own() {
        assert!(map_address_status(Some(&AccountData::mock())).is_empty());
    }

    #[test]
    fn test_an_account_that_does_not_exist_yet_is_not_flagged() {
        assert!(map_address_status(None).is_empty());
    }

    #[test]
    fn test_an_account_assigned_to_a_program_is_externally_controlled() {
        let account = AccountData {
            owner: "9G4pPipvCwQkf2X3CtFFJgK88vdN43EoKn7Kf8wxjKa".to_string(),
            ..AccountData::mock()
        };
        assert_eq!(map_address_status(Some(&account)), vec![AddressStatus::ExternallyControlled]);
    }
}
