use primitives::{AddressFormatStyle, AddressFormatter, Chain};

pub type GemAddressFormatStyle = AddressFormatStyle;

#[uniffi::remote(Enum)]
pub enum GemAddressFormatStyle {
    Short,
    Full,
    Extra { extra: u32 },
}

pub fn format_address(address: &str, chain: Option<Chain>, style: GemAddressFormatStyle) -> String {
    AddressFormatter::format(address, chain, style)
}

#[derive(Default, uniffi::Object)]
pub struct GemAddressService {}

#[uniffi::export]
impl GemAddressService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn format(&self, address: String, chain: Option<Chain>, style: GemAddressFormatStyle) -> String {
        format_address(&address, chain, style)
    }
}
