use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::Contact;
use primitives::contact::ContactAddress;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemContactStore: Send + Sync {
    async fn get_addresses(&self, contact_id: String) -> Result<Vec<ContactAddress>, GemServiceError>;
    async fn save_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError>;
    async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>, delete_address_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn delete_contact(&self, contact_id: String) -> Result<(), GemServiceError>;
}
