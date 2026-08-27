use crate::services::collections::stale;

use primitives::contact::ContactAddress;
use primitives::{AddressName, AddressType, Contact, VerificationStatus};

pub fn address_names(contact: &Contact, addresses: &[ContactAddress]) -> Vec<AddressName> {
    addresses
        .iter()
        .map(|address| AddressName {
            chain: address.chain,
            address: address.address.clone(),
            name: contact.name.clone(),
            address_type: AddressType::Contact,
            status: VerificationStatus::Verified,
            image_url: contact.image_url.clone(),
        })
        .collect()
}

pub fn stale_address_ids(existing_ids: Vec<String>, addresses: &[ContactAddress]) -> Vec<String> {
    stale(existing_ids, addresses.iter().map(|address| address.id.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use primitives::Chain;

    fn address(id: &str) -> ContactAddress {
        ContactAddress {
            id: id.into(),
            contact_id: "contact".into(),
            address: format!("0x{id}"),
            chain: Chain::Ethereum,
            memo: None,
        }
    }

    #[test]
    fn test_contact_rules() {
        let contact = Contact {
            id: "contact".into(),
            name: "Alice".into(),
            description: None,
            image_url: Some("image".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let names = address_names(&contact, &[address("a")]);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].name, "Alice");
        assert_eq!(names[0].address_type, AddressType::Contact);
        assert_eq!(names[0].status, VerificationStatus::Verified);
        assert_eq!(names[0].image_url.as_deref(), Some("image"));

        assert_eq!(stale_address_ids(vec!["a".into(), "b".into()], &[address("a"), address("c")]), vec!["b".to_string()]);
    }
}
