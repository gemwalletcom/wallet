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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemContactAvatarChoice {
    Empty,
    Image { image_url: String },
    Emoji { emoji: String },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemContactSession {
    pub id: String,
    pub existing: Option<Contact>,
    pub name: String,
    pub description: String,
    pub avatar: GemContactAvatarChoice,
    pub addresses: Vec<ContactAddress>,
    pub is_saving: bool,
}

#[uniffi::export]
impl GemContactSession {
    pub fn on_name_changed(&self, name: String) -> Self {
        Self { name, ..self.clone() }
    }

    pub fn on_description_changed(&self, description: String) -> Self {
        Self { description, ..self.clone() }
    }

    pub fn on_avatar_changed(&self, avatar: GemContactAvatarChoice) -> Self {
        Self { avatar, ..self.clone() }
    }

    pub fn on_address_saved(&self, input: GemContactAddressInput) -> Self {
        Self {
            addresses: input.add_address(self.addresses.clone()),
            ..self.clone()
        }
    }

    pub fn on_address_deleted(&self, address_id: String) -> Self {
        Self {
            addresses: self.addresses.iter().filter(|address| address.id != address_id).cloned().collect(),
            ..self.clone()
        }
    }

    pub fn on_saving(&self, is_saving: bool) -> Self {
        Self { is_saving, ..self.clone() }
    }

    pub fn can_save(&self) -> bool {
        rules::can_save_contact(&self.name, self.is_saving)
    }

    pub fn initials(&self) -> String {
        contact_initials(self.name.clone())
    }

    pub fn input(&self, avatar: GemContactAvatar) -> GemContactInput {
        GemContactInput {
            id: self.id.clone(),
            existing: self.existing.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            avatar,
            addresses: self.addresses.clone(),
        }
    }
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

    fn session() -> GemContactSession {
        GemContactSession {
            id: "contact".into(),
            existing: None,
            name: String::new(),
            description: String::new(),
            avatar: GemContactAvatarChoice::Empty,
            addresses: vec![],
            is_saving: false,
        }
    }

    #[test]
    fn test_a_session_saves_once_it_has_a_name_and_is_not_saving() {
        assert!(!session().can_save());
        assert!(!session().on_name_changed("  ".into()).can_save(), "spaces are not a name");
        let named = session().on_name_changed("ada lovelace".into());
        assert!(named.can_save());
        assert_eq!(named.initials(), "AD");
        assert!(!named.on_saving(true).can_save(), "a save already running blocks another");
    }

    #[test]
    fn test_a_session_adds_replaces_and_deletes_addresses() {
        let input = |address: &str, replacing_id: Option<String>| GemContactAddressInput {
            contact_id: "contact".into(),
            chain: Chain::Ethereum,
            address: address.into(),
            memo: None,
            replacing_id,
        };
        let added = session().on_address_saved(input("0x1", None));
        let replaced = added.on_address_saved(input("0x2", Some(added.addresses[0].id.clone())));

        assert_eq!(replaced.addresses.len(), 1);
        assert_eq!(replaced.addresses[0].address, "0x2");
        assert!(replaced.on_address_deleted(replaced.addresses[0].id.clone()).addresses.is_empty());
    }

    #[test]
    fn test_the_save_input_carries_the_form_and_the_rendered_avatar() {
        let input = session()
            .on_name_changed("Ada".into())
            .on_description_changed("Friend".into())
            .on_avatar_changed(GemContactAvatarChoice::Emoji { emoji: "🦊".into() })
            .input(GemContactAvatar::Rendered { image: vec![1] });

        assert_eq!((input.id.as_str(), input.name.as_str(), input.description.as_str()), ("contact", "Ada", "Friend"));
        assert!(matches!(input.avatar, GemContactAvatar::Rendered { .. }));
    }

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
        assert_eq!(contact_row(Contact { name: "Ada".into(), ..Contact::mock() }).subtitle, None);
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
        assert_eq!(contact_row(Contact { name: "Q".into(), ..Contact::mock() }).initials, "Q");
        assert_eq!(contact_row(Contact { name: "".into(), ..Contact::mock() }).initials, "");
    }
}
