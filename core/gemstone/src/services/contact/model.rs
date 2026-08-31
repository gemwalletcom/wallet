use primitives::contact::ContactAddress;
use primitives::{Chain, Contact};

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

#[derive(uniffi::Record)]
pub struct GemContactAddressInput {
    pub contact_id: String,
    pub chain: Chain,
    pub address: String,
    pub memo: Option<String>,
    pub replacing_id: Option<String>,
}
