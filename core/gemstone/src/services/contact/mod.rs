pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::contact::ContactAddress;
use primitives::{Chain, Contact};

use crate::services::file::GemFileStore;
use crate::services::name::GemAddressStore;

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

    pub async fn add_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        self.store.save_contact(contact.clone(), addresses.clone()).await?;
        self.save_address_names(&contact, &addresses).await
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

    pub async fn delete_contact(&self, contact: Contact) -> Result<(), GemServiceError> {
        let existing = self.store.get_addresses(contact.id.clone()).await?;
        self.store.delete_contact(contact.id.clone()).await?;
        self.address_store.delete_address_names(rules::address_names(&contact, &existing)).await?;
        match contact.image_url {
            Some(file_name) => self.files.remove(file_name),
            None => Ok(()),
        }
    }

    pub fn save_avatar(&self, image: Vec<u8>) -> Result<String, GemServiceError> {
        self.files.save(image, AVATAR_EXTENSION.to_string())
    }

    pub fn remove_avatar(&self, file_name: String) -> Result<(), GemServiceError> {
        self.files.remove(file_name)
    }
}

impl GemContactService {
    async fn save_address_names(&self, contact: &Contact, addresses: &[ContactAddress]) -> Result<(), GemServiceError> {
        self.address_store.save_address_names(rules::address_names(contact, addresses)).await
    }
}

#[uniffi::export]
pub fn default_contact_chain() -> Chain {
    rules::default_contact_chain()
}
