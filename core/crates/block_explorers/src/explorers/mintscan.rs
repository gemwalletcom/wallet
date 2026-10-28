use crate::BlockExplorer;
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
    use primitives::{AssetId, Chain};

    use super::*;

    #[test]
    fn test_mintscan_cosmos() {
        let explorer = new_cosmos();
        assert_eq!(explorer.name(), "Mintscan");
        assert_eq!(explorer.get_tx_url("abc123"), "https://www.mintscan.io/cosmos/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://www.mintscan.io/cosmos/address/addr123");
        assert_eq!(explorer.get_validator_url("val123"), Some("https://www.mintscan.io/cosmos/validators/val123".to_string()));
        assert_eq!(
            explorer.get_token_url(&AssetId::from_token(Chain::Cosmos, "ibc/0025F8A87464A471E66B234C4F93AEC5B4DA3D42D7986451A059273426290DD5")),
            Some("https://www.mintscan.io/cosmos/assets/ibc/0025F8A87464A471E66B234C4F93AEC5B4DA3D42D7986451A059273426290DD5".to_string())
        );
    }

    #[test]
    fn test_mintscan_noble() {
        let explorer = new_noble();
        assert_eq!(
            explorer.get_tx_url("22F0B4F48A85925A668D64134B7377476DC5BAE3CF7CC38AFC0E17E5F7D90001"),
            "https://www.mintscan.io/noble/tx/22F0B4F48A85925A668D64134B7377476DC5BAE3CF7CC38AFC0E17E5F7D90001"
        );
        assert_eq!(
            explorer.get_address_url("noble17w8y9eujrz4m08nn0h349s5h2rs8uz5hqe02z4"),
            "https://www.mintscan.io/noble/address/noble17w8y9eujrz4m08nn0h349s5h2rs8uz5hqe02z4"
        );
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
