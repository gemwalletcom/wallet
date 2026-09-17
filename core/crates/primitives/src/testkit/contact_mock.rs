use chrono::Utc;

use crate::{Chain, Contact, ContactAddress};

impl Contact {
    pub fn mock() -> Self {
        Self {
            id: "contact".to_string(),
            name: "Alice".to_string(),
            description: None,
            image_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
