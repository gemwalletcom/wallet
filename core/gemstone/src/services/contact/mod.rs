pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::Contact;
use primitives::contact::ContactAddress;

use crate::services::name::GemAddressStore;

pub use store::GemContactStore;

#[derive(uniffi::Object)]
pub struct GemContactService {
    store: Arc<dyn GemContactStore>,
    address_store: Arc<dyn GemAddressStore>,
}

#[uniffi::export]
impl GemContactService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemContactStore>, address_store: Arc<dyn GemAddressStore>) -> Self {
        Self { store, address_store }
    }

    pub async fn add_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        self.store.save_contact(contact.clone(), addresses.clone()).await?;
        self.save_address_names(&contact, &addresses).await
    }

    pub async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        let existing_ids = self.store.get_address_ids(contact.id.clone()).await?;
        let delete_address_ids = rules::stale_address_ids(existing_ids, &addresses);
        self.store.update_contact(contact.clone(), addresses.clone(), delete_address_ids).await?;
        self.save_address_names(&contact, &addresses).await
    }

    pub async fn delete_contact(&self, contact_id: String) -> Result<(), GemServiceError> {
        self.store.delete_contact(contact_id).await
    }
}

impl GemContactService {
    async fn save_address_names(&self, contact: &Contact, addresses: &[ContactAddress]) -> Result<(), GemServiceError> {
        self.address_store.save_address_names(rules::address_names(contact, addresses)).await
    }
}
