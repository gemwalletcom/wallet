use crate::BlockExplorerLink;

impl BlockExplorerLink {
    pub fn mock() -> Self {
        BlockExplorerLink {
            name: "Etherscan".to_string(),
            link: "https://etherscan.io/token/0x1".to_string(),
        }
    }
}
