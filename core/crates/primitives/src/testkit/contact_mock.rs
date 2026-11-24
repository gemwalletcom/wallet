use crate::{Chain, Contact, ContactAddress, DateTime};

impl Contact {
    pub fn mock() -> Self {
        Self {
            id: "contact".to_string(),
            name: "Alice".to_string(),
            description: None,
            image_url: None,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }
}

impl ContactAddress {
    pub fn mock(id: &str) -> Self {
        Self {
            id: id.to_string(),
            contact_id: "contact".to_string(),
            address: format!("0x{id}"),
            chain: Chain::Ethereum,
            memo: None,
        }
    }
}
