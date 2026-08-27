use crate::services::collections::stale_by;
use primitives::contact::ContactAddress;
use primitives::{AddressName, AddressType, Chain, Contact, VerificationStatus};

pub fn default_contact_chain() -> Chain {
    Chain::Bitcoin
}

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

pub fn stale_addresses(existing: Vec<ContactAddress>, addresses: &[ContactAddress]) -> Vec<ContactAddress> {
    stale_by(existing, addresses.iter().map(|address| address.id.clone()), |address| address.id.clone())
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

        let stale: Vec<String> = stale_addresses(vec![address("a"), address("b")], &[address("a"), address("c")])
            .into_iter()
            .map(|address| address.id)
            .collect();
        assert_eq!(stale, vec!["b".to_string()]);
    }
}
