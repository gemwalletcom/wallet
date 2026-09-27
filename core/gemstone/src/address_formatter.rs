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
}

impl GemAddressService {
    pub fn name_text(&self, name: Option<String>, address: String, has_image: bool) -> Option<String> {
        let name = name.filter(|name| !name.is_empty() && *name != address)?;
        Some(match has_image || address.is_empty() {
            true => name,
            false => format!("{name} ({address})"),
        })
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn test_a_name_that_repeats_the_address_is_not_a_name_and_an_imageless_one_keeps_its_address() {
        let service = GemAddressService::new();
        let address = "0xabc".to_string();

        assert_eq!(service.name_text(None, address.clone(), false), None);
        assert_eq!(service.name_text(Some(address.clone()), address.clone(), false), None);
        assert_eq!(service.name_text(Some(String::new()), address.clone(), false), None);
        assert_eq!(service.name_text(Some("Ada".into()), address.clone(), true).as_deref(), Some("Ada"));
        assert_eq!(service.name_text(Some("Ada".into()), String::new(), false).as_deref(), Some("Ada"));
        assert_eq!(service.name_text(Some("Ada".into()), address, false).as_deref(), Some("Ada (0xabc)"));
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
            addresses.iter().map(|entry| service.format(entry.address.clone(), Some(entry.chain), GemAddressFormatStyle::Short)).collect::<Vec<_>>()
        );
    }
}
