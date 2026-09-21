use primitives::AddressStatus;

use crate::models::account::Account;

pub fn map_address_status(address: &str, account: Option<&Account>) -> Vec<AddressStatus> {
    let Some(account) = account else {
        return vec![];
    };
    let own_weight: u32 = account.signers.iter().filter(|signer| signer.key == address).map(|signer| signer.weight).sum();
    match own_weight < account.thresholds.med_threshold.max(1) {
        true => vec![AddressStatus::ExternallyControlled],
        false => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{Signer, Thresholds};

    const ADDRESS: &str = "GAN2JTIWVKGZIDN5R2AFYLUV4IUXLBG3MQA3R5ECIIM5RUYT74Y3LDOP";
    const OTHER_ADDRESS: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";

    #[test]
    fn test_an_account_its_key_can_pay_from_is_its_own() {
        let co_signed = Account {
            signers: vec![Signer { key: ADDRESS.to_string(), weight: 2 }, Signer { key: OTHER_ADDRESS.to_string(), weight: 1 }],
            thresholds: Thresholds { med_threshold: 2 },
            ..Account::mock(ADDRESS)
        };

        assert!(map_address_status(ADDRESS, Some(&Account::mock(ADDRESS))).is_empty());
        assert!(map_address_status(ADDRESS, Some(&co_signed)).is_empty());
        assert!(map_address_status(ADDRESS, None).is_empty());
    }

    #[test]
    fn test_a_key_below_the_payment_threshold_is_externally_controlled() {
        let master_disabled = Account {
            signers: vec![Signer { key: ADDRESS.to_string(), weight: 0 }, Signer { key: OTHER_ADDRESS.to_string(), weight: 1 }],
            ..Account::mock(ADDRESS)
        };
        let below_threshold = Account {
            thresholds: Thresholds { med_threshold: 2 },
            ..Account::mock(ADDRESS)
        };

        assert_eq!(map_address_status(ADDRESS, Some(&master_disabled)), vec![AddressStatus::ExternallyControlled]);
        assert_eq!(map_address_status(ADDRESS, Some(&below_threshold)), vec![AddressStatus::ExternallyControlled]);
    }
}
