use primitives::{AddressFormatStyle, AddressFormatter, Chain, ChainAddress};

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

    pub fn format_all(&self, addresses: Vec<ChainAddress>, style: GemAddressFormatStyle) -> Vec<String> {
        addresses.iter().map(|entry| format_address(&entry.address, Some(entry.chain), style)).collect()
    }

    pub fn display(&self, name: Option<String>, address: String, has_image: bool) -> GemAddressDisplay {
        match name.filter(|name| !name.is_empty() && *name != address) {
            None => GemAddressDisplay::Address,
            Some(name) if has_image || address.is_empty() => GemAddressDisplay::Name { name },
            Some(name) => GemAddressDisplay::NameWithAddress { name },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemAddressDisplay {
    Address,
    Name { name: String },
    NameWithAddress { name: String },
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn test_a_name_that_repeats_the_address_is_not_a_name_and_an_imageless_one_keeps_its_address() {
        let service = GemAddressService::new();
        let address = "0xabc".to_string();

        assert_eq!(service.display(None, address.clone(), false), GemAddressDisplay::Address);
        assert_eq!(service.display(Some(address.clone()), address.clone(), false), GemAddressDisplay::Address);
        assert_eq!(service.display(Some(String::new()), address.clone(), false), GemAddressDisplay::Address);
        assert_eq!(service.display(Some("Ada".into()), address.clone(), true), GemAddressDisplay::Name { name: "Ada".into() });
        assert_eq!(
            service.display(Some("Ada".into()), address, false),
            GemAddressDisplay::NameWithAddress { name: "Ada".into() }
        );
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    #[test]
    fn test_format_all_answers_one_string_per_address_in_order() {
        let service = GemAddressService::new();
        let addresses = vec![
            ChainAddress {
                chain: Chain::Ethereum,
                address: "0x1234567890abcdef".to_string(),
            },
            ChainAddress {
                chain: Chain::Bitcoin,
                address: "bc1qxy2kgdygjrsqtzq2n0yrf249".to_string(),
            },
        ];

        let formatted = service.format_all(addresses.clone(), GemAddressFormatStyle::Short);

        assert_eq!(
            formatted,
            addresses
                .iter()
                .map(|entry| service.format(entry.address.clone(), Some(entry.chain), GemAddressFormatStyle::Short))
                .collect::<Vec<_>>()
        );
    }
}
