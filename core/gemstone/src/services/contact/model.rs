use primitives::{AddressName, AddressType, Chain, Contact};
use primitives::{OptionStringExt, contact::ContactAddress};

use super::rules;
use crate::address_formatter::{GemAddressFormatStyle, format_address};
use crate::models::state::GemListPhase;
use crate::services::chain::{GemChainRow, chain_row};
use crate::services::empty_state::{GemEmptyStateKind, empty_state};

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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemContactAvatarImage {
    Initials { text: String },
    Placeholder,
    Image { image_url: String, initials: String },
    Emoji { emoji: String },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemContactAddressRow {
    pub address: ContactAddress,
    pub chain: GemChainRow,
    pub short_address: String,
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

    pub fn address_rows(&self) -> Vec<GemContactAddressRow> {
        self.addresses
            .iter()
            .map(|address| GemContactAddressRow {
                chain: chain_row(address.chain),
                short_address: format_address(&address.address, Some(address.chain), GemAddressFormatStyle::Short),
                address: address.clone(),
            })
            .collect()
    }

    pub fn can_save(&self) -> bool {
        rules::can_save_contact(&self.name, self.is_saving)
    }

    pub fn avatar_image(&self) -> GemContactAvatarImage {
        match &self.avatar {
            GemContactAvatarChoice::Empty => contact_avatar_image(None, &self.name),
            GemContactAvatarChoice::Image { image_url } => contact_avatar_image(Some(image_url.clone()), &self.name),
            GemContactAvatarChoice::Emoji { emoji } => GemContactAvatarImage::Emoji { emoji: emoji.clone() },
        }
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemContactAddressSession {
    pub contact_id: String,
    pub replacing_id: Option<String>,
    pub chain: Chain,
    pub memo: String,
    pub fields: Vec<GemContactAddressField>,
}

#[uniffi::export]
impl GemContactAddressSession {
    pub fn on_chain_changed(&self, chain: Chain) -> Self {
        Self {
            chain,
            memo: String::new(),
            fields: rules::contact_address_fields(chain),
            ..self.clone()
        }
    }

    pub fn on_memo_changed(&self, memo: String) -> Self {
        Self { memo, ..self.clone() }
    }

    pub fn on_scanned(&self, scan: GemContactScannedAddress) -> Self {
        Self {
            memo: scan.memo.unwrap_or_else(|| self.memo.clone()),
            ..self.clone()
        }
    }

    pub fn input(&self, address: String) -> GemContactAddressInput {
        GemContactAddressInput {
            contact_id: self.contact_id.clone(),
            chain: self.chain,
            address,
            memo: Some(self.memo.clone()),
            replacing_id: self.replacing_id.clone(),
        }
    }
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
        assert_eq!(named.avatar_image(), GemContactAvatarImage::Initials { text: "AD".into() });
        assert_eq!(session().avatar_image(), GemContactAvatarImage::Placeholder, "a blank name has no initials to show");
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
    fn test_each_address_row_names_its_network_and_shortens_the_address() {
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let session = session().on_address_saved(GemContactAddressInput {
            contact_id: "contact".into(),
            chain: Chain::Ethereum,
            address: address.into(),
            memo: None,
            replacing_id: None,
        });
        let row = session.address_rows().remove(0);

        assert_eq!(row.chain, chain_row(Chain::Ethereum));
        assert_eq!(row.short_address, format_address(address, Some(Chain::Ethereum), GemAddressFormatStyle::Short));
        assert_ne!(row.short_address, address);
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
    fn test_an_address_session_clears_the_memo_with_the_chain_and_takes_a_scanned_memo() {
        let session = rules::new_address_session("contact".into(), None).on_memo_changed("note".into());
        assert_eq!(session.memo, "note");

        let cosmos = session.on_chain_changed(Chain::Cosmos);
        assert_eq!(cosmos.memo, "", "a new chain starts without the old memo");
        assert_eq!(cosmos.fields, rules::contact_address_fields(Chain::Cosmos));

        let typed = cosmos.on_memo_changed("typed".into());
        assert_eq!(typed.on_scanned(GemContactScannedAddress { address: "cosmos1".into(), memo: None }).memo, "typed", "a scan without a memo keeps the typed one");
        assert_eq!(
            typed
                .on_scanned(GemContactScannedAddress {
                    address: "cosmos1".into(),
                    memo: Some("tag".into())
                })
                .memo,
            "tag"
        );

        let input = rules::new_address_session("contact".into(), Some(ContactAddress::mock("old"))).on_memo_changed("note".into()).input("0xnew".into());
        assert_eq!(input.contact_id, "contact");
        assert_eq!(input.chain, Chain::Ethereum);
        assert_eq!(input.address, "0xnew");
        assert_eq!(input.memo.as_deref(), Some("note"));
        assert_eq!(input.replacing_id.as_deref(), Some("old"));
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
    pub avatar: GemContactAvatarImage,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemContactList {
    pub rows: Vec<GemContactRow>,
    pub phase: GemListPhase,
}

#[uniffi::export]
pub fn contact_list(contacts: Vec<Contact>) -> GemContactList {
    GemContactList {
        phase: GemListPhase::local(!contacts.is_empty(), empty_state(GemEmptyStateKind::Contacts)),
        rows: contacts.into_iter().map(contact_row).collect(),
    }
}

pub fn contact_row(contact: Contact) -> GemContactRow {
    GemContactRow {
        title: contact.name.clone(),
        avatar: contact_avatar_image(contact.image_url.clone(), &contact.name),
        subtitle: contact.description.filter(|description| !description.trim().is_empty()),
    }
}

fn contact_avatar_image(image_url: Option<String>, name: &str) -> GemContactAvatarImage {
    let initials = contact_initials(name.to_string());
    match (image_url, initials.is_empty()) {
        (Some(image_url), _) => GemContactAvatarImage::Image { image_url, initials },
        (None, true) => GemContactAvatarImage::Placeholder,
        (None, false) => GemContactAvatarImage::Initials { text: initials },
    }
}

pub fn contact_initials(name: String) -> String {
    name.trim().chars().take(2).collect::<String>().to_uppercase()
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAvatar {
    pub image_url: Option<String>,
    pub initials: String,
}

pub fn contact_avatar(address_name: Option<&AddressName>, name: Option<&str>) -> Option<GemAvatar> {
    let address_name = address_name.filter(|address_name| address_name.address_type == AddressType::Contact)?;
    Some(GemAvatar {
        image_url: address_name.image_url.clone().non_empty(),
        initials: contact_initials(name.unwrap_or(&address_name.name).to_string()),
    })
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
    fn test_a_contact_list_shows_its_rows_or_the_contacts_empty_state() {
        let listed = contact_list(vec![Contact::mock()]);
        assert_eq!((listed.rows.len(), listed.phase), (1, GemListPhase::Rows));
        assert_eq!(
            contact_list(vec![]).phase,
            GemListPhase::Empty {
                state: empty_state(GemEmptyStateKind::Contacts)
            }
        );
    }

    #[test]
    fn test_a_row_avatar_shows_the_image_then_the_initials_then_a_placeholder() {
        let row = |name: &str, image_url: Option<&str>| {
            contact_row(Contact {
                name: name.into(),
                image_url: image_url.map(String::from),
                ..Contact::mock()
            })
            .avatar
        };
        assert_eq!(row("  ada lovelace", None), GemContactAvatarImage::Initials { text: "AD".into() });
        assert_eq!(row("Q", None), GemContactAvatarImage::Initials { text: "Q".into() });
        assert_eq!(row("", None), GemContactAvatarImage::Placeholder);
        assert_eq!(
            row("ada", Some("avatar.png")),
            GemContactAvatarImage::Image {
                image_url: "avatar.png".into(),
                initials: "AD".into()
            }
        );
    }
}
