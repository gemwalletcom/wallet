use async_trait::async_trait;
use primitives::contact::ContactAddress;
use primitives::Contact;

use super::error::GemContactError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemContactStore: Send + Sync {
    async fn get_address_ids(&self, contact_id: String) -> Result<Vec<String>, GemContactError>;
    async fn save_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemContactError>;
    async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>, delete_address_ids: Vec<String>) -> Result<(), GemContactError>;
    async fn delete_contact(&self, contact_id: String) -> Result<(), GemContactError>;
}
