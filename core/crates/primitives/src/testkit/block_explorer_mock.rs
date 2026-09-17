use crate::BlockExplorerLink;

impl BlockExplorerLink {
    pub fn mock() -> Self {
        BlockExplorerLink {
            name: "Etherscan".to_string(),
            link: "https://etherscan.io/token/0x1".to_string(),
        }
    }

    pub fn mock_with_address(address: &str) -> Self {
        BlockExplorerLink {
            name: "Explorer".to_string(),
            link: format!("https://explorer/{address}"),
        }
    }
}
