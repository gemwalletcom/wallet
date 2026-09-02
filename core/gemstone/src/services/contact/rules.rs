use crate::services::collections::stale_by;
use chrono::{DateTime, Utc};
use primitives::contact::ContactAddress;
use primitives::{AddressName, AddressType, Chain, Contact, PaymentRequest, VerificationStatus};

use super::model::GemContactScannedAddress;
use std::collections::HashSet;

pub fn default_contact_chain() -> Chain {
    Chain::Bitcoin
}

pub fn contact(existing: Option<&Contact>, id: String, name: String, description: String, image_url: Option<String>, now: DateTime<Utc>) -> Contact {
    Contact {
        id,
        name: name.trim().to_string(),
        description: blank_to_none(description),
        image_url,
        created_at: existing.map(|contact| contact.created_at).unwrap_or(now),
        updated_at: now,
    }
}

pub fn scanned_address(input: &str, payment: Option<&PaymentRequest>) -> GemContactScannedAddress {
    let address = payment.map(|payment| payment.address.trim()).filter(|address| !address.is_empty()).unwrap_or(input.trim());
    GemContactScannedAddress {
        address: address.to_string(),
        memo: payment.and_then(|payment| payment.memo.clone()),
    }
}

pub fn contact_address(contact_id: String, chain: Chain, address: String, memo: Option<String>) -> ContactAddress {
    ContactAddress {
        id: format!("{contact_id}_{}_{address}", chain.as_ref()),
        contact_id,
        address,
        chain,
        memo: memo.and_then(blank_to_none),
    }
}

pub fn upsert_address(addresses: Vec<ContactAddress>, address: ContactAddress, replacing_id: Option<String>) -> Vec<ContactAddress> {
    let replaced: HashSet<String> = replacing_id.into_iter().chain([address.id.clone()]).collect();
    let position = addresses.iter().position(|item| replaced.contains(&item.id));
    let kept: Vec<ContactAddress> = addresses.into_iter().filter(|item| !replaced.contains(&item.id)).collect();
    let index = position.unwrap_or(kept.len()).min(kept.len());
    [&kept[..index], &[address], &kept[index..]].concat()
}

fn blank_to_none(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
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

    #[test]
    fn test_contact_address() {
        let address = contact_address("contact".into(), Chain::Ethereum, "0xabc".into(), Some("  ".into()));

        assert_eq!(address.id, "contact_ethereum_0xabc");
        assert_eq!(address.contact_id, "contact");
        assert_eq!(address.memo, None);
        assert_eq!(
            contact_address("contact".into(), Chain::Ethereum, "0xabc".into(), Some(" note ".into())).memo.as_deref(),
            Some("note")
        );
    }

    #[test]
    fn test_scanned_address_prefers_the_payment_address_and_memo() {
        let payment = PaymentRequest {
            address: " 0xabc ".into(),
            amount: None,
            memo: Some("tag".into()),
            references: None,
            asset_id: None,
        };
        assert_eq!(
            scanned_address("bitcoin:0xabc?dt=tag", Some(&payment)),
            GemContactScannedAddress {
                address: "0xabc".into(),
                memo: Some("tag".into())
            }
        );
        let blank = PaymentRequest { address: "  ".into(), ..payment };
        assert_eq!(scanned_address(" raw ", Some(&blank)).address, "raw");
        assert_eq!(scanned_address(" raw ", None), GemContactScannedAddress { address: "raw".into(), memo: None });
    }

    #[test]
    fn test_upsert_address_replaces_an_edited_address_in_place() {
        let existing = vec![address("a"), address("b"), address("c")];
        let edited = ContactAddress {
            address: "0xnew".into(),
            ..address("renamed")
        };

        let addresses = upsert_address(existing.clone(), edited.clone(), Some("b".into()));

        let ids: Vec<String> = addresses.into_iter().map(|address| address.id).collect();
        assert_eq!(ids, vec!["a".to_string(), "renamed".to_string(), "c".to_string()]);

        let appended: Vec<String> = upsert_address(existing.clone(), edited, None).into_iter().map(|address| address.id).collect();
        assert_eq!(appended, vec!["a".to_string(), "b".to_string(), "c".to_string(), "renamed".to_string()]);

        let same_id: Vec<String> = upsert_address(existing, address("b"), None).into_iter().map(|address| address.id).collect();
        assert_eq!(same_id, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }

    #[test]
    fn test_contact_keeps_the_original_creation_time() {
        let created_at = Utc::now() - chrono::Duration::days(2);
        let existing = Contact {
            id: "contact".into(),
            name: "Alice".into(),
            description: Some("old".into()),
            image_url: Some("image".into()),
            created_at,
            updated_at: created_at,
        };
        let now = Utc::now();

        let updated = contact(Some(&existing), "contact".into(), "  Bob  ".into(), "   ".into(), None, now);

        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.description, None);
        assert_eq!(updated.created_at, created_at);
        assert_eq!(updated.updated_at, now);
        assert_eq!(contact(None, "new".into(), "Bob".into(), "note".into(), None, now).created_at, now);
    }
}
