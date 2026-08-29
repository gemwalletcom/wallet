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
pub struct GemAddressRulesService {}

#[uniffi::export]
impl GemAddressRulesService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate(&self, address: String, chain: Chain) -> bool {
        crate::address::validate_address(&address, chain)
    }

    pub fn checksum(&self, address: String, chain: Chain) -> String {
        crate::address::checksum_address(&address, chain)
    }

    pub fn short(&self, address: String, chain: Chain) -> String {
        crate::address::short_address(&address, chain)
    }

    pub fn format(&self, address: String, chain: Option<Chain>, style: GemAddressFormatStyle) -> String {
        format_address(&address, chain, style)
    }
}
