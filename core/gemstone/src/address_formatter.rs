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

pub fn name_text(name: Option<String>, address: String, has_image: bool) -> Option<String> {
    let name = name.filter(|name| !name.is_empty() && *name != address)?;
    Some(match has_image || address.is_empty() {
        true => name,
        false => format!("{name} ({address})"),
    })
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn test_a_name_that_repeats_the_address_is_not_a_name_and_an_imageless_one_keeps_its_address() {
        let address = "0xabc".to_string();

        assert_eq!(name_text(None, address.clone(), false), None);
        assert_eq!(name_text(Some(address.clone()), address.clone(), false), None);
        assert_eq!(name_text(Some(String::new()), address.clone(), false), None);
        assert_eq!(name_text(Some("Ada".into()), address.clone(), true).as_deref(), Some("Ada"));
        assert_eq!(name_text(Some("Ada".into()), String::new(), false).as_deref(), Some("Ada"));
        assert_eq!(name_text(Some("Ada".into()), address, false).as_deref(), Some("Ada (0xabc)"));
    }
}
