use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::Metadata;

const NAME: &str = "Mintscan";

pub fn new_cosmos() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/cosmos").explorer()
}

pub fn new_osmosis() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/osmosis").explorer()
}

pub fn new_celestia() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/celestia").explorer()
}

pub fn new_injective() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/injective").explorer()
}

pub fn new_sei() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/sei").explorer()
}

pub fn new_noble() -> Box<dyn BlockExplorer> {
    Metadata::mintscan(NAME, "https://www.mintscan.io/noble").explorer()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mintscan_cosmos() {
        let explorer = new_cosmos();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/cosmos/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/cosmos/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/cosmos/validators/val123".to_string()));
    }

    #[test]
    fn test_mintscan_injective() {
        let explorer = new_injective();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/injective/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/injective/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/injective/validators/val123".to_string()));
    }

    #[test]
    fn test_mintscan_osmosis() {
        let explorer = new_osmosis();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/osmosis/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/osmosis/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/osmosis/validators/val123".to_string()));
    }
}
