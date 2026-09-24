pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use primitives::contact::ContactAddress;
use primitives::{Chain, Contact};

use crate::address_formatter::{GemAddressFormatStyle, GemAddressService};
use crate::models::payment::GemPayment;
use crate::payment::GemPaymentService;
use crate::services::file::{GemFileStore, IMAGE_EXTENSION};
use crate::services::name::GemNameService;

pub use model::{GemContactAddressInput, GemContactAvatar, GemContactAvatarChoice, GemContactAvatarImage, GemContactInput, GemContactRow, GemContactScannedAddress, GemContactSession, contact_initials, contact_row};
pub use store::GemContactStore;

#[derive(uniffi::Object)]
pub struct GemContactService {
    store: Arc<dyn GemContactStore>,
    names: Arc<GemNameService>,
    files: Arc<dyn GemFileStore>,
}

#[uniffi::export]
impl GemContactService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemContactStore>, names: Arc<GemNameService>, files: Arc<dyn GemFileStore>) -> Self {
        Self { store, names, files }
    }

    pub async fn delete_contact(&self, contact: Contact) -> Result<(), GemServiceError> {
        let existing = self.store.get_addresses(contact.id.clone()).await?;
        self.store.delete_contact(contact.id.clone()).await?;
        self.names.delete_names(rules::address_names(&contact, &existing)).await?;
        match contact.image_url {
            Some(file_name) => self.files.remove(file_name),
            None => Ok(()),
        }
    }

    pub async fn update_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        let existing = self.store.get_addresses(contact.id.clone()).await?;
        let stale = rules::stale_addresses(existing, &addresses);
        self.store.update_contact(contact.clone(), addresses.clone(), stale.iter().map(|address| address.id.clone()).collect()).await?;
        self.names.delete_names(rules::address_names(&contact, &stale)).await?;
        self.save_address_names(&contact, &addresses).await
    }
}

impl GemContactService {
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

    pub fn default_chain(&self) -> Chain {
        rules::default_contact_chain()
    }

    async fn add_contact(&self, contact: Contact, addresses: Vec<ContactAddress>) -> Result<(), GemServiceError> {
        self.store.save_contact(contact.clone(), addresses.clone()).await?;
        self.save_address_names(&contact, &addresses).await
    }

    fn save_avatar(&self, image: Vec<u8>) -> Result<String, GemServiceError> {
        self.files.save_file(image, IMAGE_EXTENSION.to_string())
    }

    fn remove_avatar(&self, file_name: String) -> Result<(), GemServiceError> {
        self.files.remove(file_name)
    }

    async fn save_address_names(&self, contact: &Contact, addresses: &[ContactAddress]) -> Result<(), GemServiceError> {
        self.names.save_names(rules::address_names(contact, addresses)).await
    }
}

#[derive(uniffi::Object)]
pub struct GemContactEditorService {
    contacts: Arc<GemContactService>,
    addresses: Arc<GemAddressService>,
    payments: Arc<GemPaymentService>,
}

#[uniffi::export]
impl GemContactEditorService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>, addresses: Arc<GemAddressService>, payments: Arc<GemPaymentService>) -> Self {
        Self { contacts, addresses, payments }
    }

    pub fn scanned_address(&self, input: String) -> GemContactScannedAddress {
        let request = match self.payments.decode_url(input.clone()) {
            Ok(GemPayment::Request { request }) => Some(request),
            Ok(GemPayment::Link { link: _ }) | Err(_) => None,
        };
        rules::scanned_address(&input, request.as_ref())
    }

    pub fn default_chain(&self) -> Chain {
        self.contacts.default_chain()
    }

    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> {
        self.contacts.save_contact(input).await
    }

    pub fn new_session(&self, contact: Option<Contact>, addresses: Vec<ContactAddress>) -> GemContactSession {
        rules::new_session(contact, addresses, Uuid::new_v4().to_string())
    }

    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String {
        self.addresses.format(address, Some(chain), style)
    }
}
