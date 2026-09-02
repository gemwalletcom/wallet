pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use chrono::Utc;
use std::sync::Arc;

use primitives::contact::ContactAddress;
use primitives::name::NameRecord;
use primitives::{Chain, Contact};

use crate::address_formatter::{GemAddressFormatStyle, GemAddressService};
use crate::services::chain::GemChainService;
use crate::services::file::GemFileStore;
use crate::services::name::{GemAddressStore, GemNameService};
use crate::services::recipient::{GemRecipientError, GemRecipientValidation};
use crate::services::transfer::model::GemRecipient;

pub use model::{GemContactAddressInput, GemContactAvatar, GemContactInput};
pub use store::GemContactStore;

const AVATAR_EXTENSION: &str = "png";

#[derive(uniffi::Object)]
pub struct GemContactService {
    store: Arc<dyn GemContactStore>,
    address_store: Arc<dyn GemAddressStore>,
    files: Arc<dyn GemFileStore>,
}

#[uniffi::export]
impl GemContactService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemContactStore>, address_store: Arc<dyn GemAddressStore>, files: Arc<dyn GemFileStore>) -> Self {
        Self { store, address_store, files }
    }

    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> {
        let image_url = match input.avatar {
            GemContactAvatar::Empty => None,
            GemContactAvatar::Image { image_url } => Some(image_url),
            GemContactAvatar::Rendered { image } => Some(self.save_avatar(image)?),
        };
        let contact = rules::contact(input.existing.as_ref(), input.id, input.name, input.description, image_url, Utc::now());
        match input.existing {
            Some(existing) => {
                self.update_contact(contact.clone(), input.addresses).await?;
                if let Some(previous) = existing.image_url.filter(|previous| contact.image_url.as_ref() != Some(previous)) {
                    let _ = self.remove_avatar(previous);
                }
            }
            None => self.add_contact(contact.clone(), input.addresses).await?,
        }
        Ok(contact)
    }

    pub async fn delete_contact(&self, contact: Contact) -> Result<(), GemServiceError> {
        let existing = self.store.get_addresses(contact.id.clone()).await?;
        self.store.delete_contact(contact.id.clone()).await?;
        self.address_store.delete_address_names(rules::address_names(&contact, &existing)).await?;
        match contact.image_url {
            Some(file_name) => self.files.remove(file_name),
            None => Ok(()),
        }
    }

    pub async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        let existing = self.store.get_addresses(contact.id.clone()).await?;
        let stale = rules::stale_addresses(existing, &addresses);
        self.store
            .update_contact(contact.clone(), addresses.clone(), stale.iter().map(|address| address.id.clone()).collect())
            .await?;
        self.address_store.delete_address_names(rules::address_names(&contact, &stale)).await?;
        self.save_address_names(&contact, &addresses).await
    }

    pub fn default_chain(&self) -> Chain {
        rules::default_contact_chain()
    }
}

impl GemContactService {
    async fn add_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        self.store.save_contact(contact.clone(), addresses.clone()).await?;
        self.save_address_names(&contact, &addresses).await
    }

    fn save_avatar(&self, image: Vec<u8>) -> Result<String, GemServiceError> {
        self.files.save_file(image, AVATAR_EXTENSION.to_string())
    }

    fn remove_avatar(&self, file_name: String) -> Result<(), GemServiceError> {
        self.files.remove(file_name)
    }

    async fn save_address_names(&self, contact: &Contact, addresses: &[ContactAddress]) -> Result<(), GemServiceError> {
        self.address_store.save_address_names(rules::address_names(contact, addresses)).await
    }
}

#[derive(uniffi::Object)]
pub struct GemContactsService {
    contacts: Arc<GemContactService>,
}

#[uniffi::export]
impl GemContactsService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>) -> Self {
        Self { contacts }
    }

    pub async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        self.contacts.update_contact(contact, addresses).await
    }

    pub async fn delete_contact(&self, contact: Contact) -> Result<(), GemServiceError> {
        self.contacts.delete_contact(contact).await
    }
}

#[derive(uniffi::Object)]
pub struct GemManageContactService {
    contacts: Arc<GemContactService>,
    addresses: Arc<GemAddressService>,
    names: Arc<GemNameService>,
    chains: Arc<GemChainService>,
}

#[uniffi::export]
impl GemManageContactService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>, addresses: Arc<GemAddressService>, names: Arc<GemNameService>, chains: Arc<GemChainService>) -> Self {
        Self {
            contacts,
            addresses,
            names,
            chains,
        }
    }

    pub fn default_chain(&self) -> Chain {
        self.contacts.default_chain()
    }

    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> {
        self.contacts.save_contact(input).await
    }

    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String {
        self.addresses.format(address, Some(chain), style)
    }

    pub fn validate_recipient(&self, chain: Chain, input: String, name_record: Option<NameRecord>) -> GemRecipientValidation {
        self.names.validate_recipient(chain, input, name_record)
    }

    pub fn recipient(
        &self,
        chain: Chain,
        input: String,
        name_record: Option<NameRecord>,
        memo: Option<String>,
        references: Vec<String>,
    ) -> Result<GemRecipient, GemRecipientError> {
        self.names.recipient(chain, input, name_record, memo, references)
    }

    pub fn is_name_supported(&self, name: String) -> bool {
        self.names.is_name_supported(name)
    }

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<Option<NameRecord>, GemServiceError> {
        self.names.get_name_record(name, chain).await
    }

    pub fn chains(&self) -> Arc<GemChainService> {
        self.chains.clone()
    }
}
