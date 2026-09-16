use primitives::contact::ContactAddress;
use primitives::{Chain, Contact};

use super::rules;

#[derive(uniffi::Enum)]
pub enum GemContactAvatar {
    Empty,
    Image { image_url: String },
    Rendered { image: Vec<u8> },
}

#[derive(uniffi::Record)]
pub struct GemContactInput {
    pub id: String,
    pub existing: Option<Contact>,
    pub name: String,
    pub description: String,
    pub avatar: GemContactAvatar,
    pub addresses: Vec<ContactAddress>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemContactScannedAddress {
    pub address: String,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemContactAddressField {
    Network,
    Address,
    Memo,
}

#[uniffi::export]
pub fn contact_address_fields(chain: Chain) -> Vec<GemContactAddressField> {
    rules::contact_address_fields(chain)
}

#[derive(uniffi::Record)]
pub struct GemContactAddressInput {
    pub contact_id: String,
    pub chain: Chain,
    pub address: String,
    pub memo: Option<String>,
    pub replacing_id: Option<String>,
}

#[uniffi::export]
impl GemContactAddressInput {
    pub fn add_address(&self, addresses: Vec<ContactAddress>) -> Vec<ContactAddress> {
        let address = rules::contact_address(self.contact_id.clone(), self.chain, self.address.clone(), self.memo.clone());
        rules::upsert_address(addresses, address, self.replacing_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_address_replaces_the_selected_address() {
        let existing = ContactAddress::mock("old");
        let input = GemContactAddressInput {
            contact_id: "contact".into(),
            chain: Chain::Ethereum,
            address: "0xnew".into(),
            memo: Some(" note ".into()),
            replacing_id: Some(existing.id.clone()),
        };

        let addresses = input.add_address(vec![existing]);

        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0].id, "contact_ethereum_0xnew");
        assert_eq!(addresses[0].memo.as_deref(), Some("note"));
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemContactRow {
    pub title: String,
    pub subtitle: Option<String>,
    pub initials: String,
}

#[uniffi::export]
pub fn contact_row(contact: Contact) -> GemContactRow {
    GemContactRow {
        title: contact.name.clone(),
        subtitle: contact.description.filter(|description| !description.trim().is_empty()),
        initials: contact_initials(contact.name.clone()),
    }
}

#[uniffi::export]
pub fn contact_initials(name: String) -> String {
    name.trim().chars().take(2).collect::<String>().to_uppercase()
}

#[cfg(test)]
mod row_tests {
    use super::*;

    #[test]
    fn test_a_blank_description_is_not_a_subtitle() {
        assert_eq!(
            contact_row(Contact {
                name: "Ada".into(),
                description: Some("Friend".into()),
                ..Contact::mock()
            })
            .subtitle
            .as_deref(),
            Some("Friend")
        );
        assert_eq!(
            contact_row(Contact {
                name: "Ada".into(),
                description: Some("   ".into()),
                ..Contact::mock()
            })
            .subtitle,
            None
        );
        assert_eq!(
            contact_row(Contact {
                name: "Ada".into(),
                ..Contact::mock()
            })
            .subtitle,
            None
        );
    }

    #[test]
    fn test_initials_take_two_trimmed_characters_in_upper_case() {
        assert_eq!(
            contact_row(Contact {
                name: "  ada lovelace".into(),
                ..Contact::mock()
            })
            .initials,
            "AD"
        );
        assert_eq!(
            contact_row(Contact {
                name: "Q".into(),
                ..Contact::mock()
            })
            .initials,
            "Q"
        );
        assert_eq!(
            contact_row(Contact {
                name: "".into(),
                ..Contact::mock()
            })
            .initials,
            ""
        );
    }
}
