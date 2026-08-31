use primitives::block_explorer::{BlockExplorerLink, ExplorerInput};

pub type GemExplorerInput = ExplorerInput;
pub type GemBlockExplorerLink = BlockExplorerLink;

#[uniffi::remote(Record)]
pub struct GemExplorerInput {
    pub hash: String,
    pub recipient: Option<String>,
    pub memo: Option<String>,
}

#[uniffi::remote(Record)]
pub struct GemBlockExplorerLink {
    pub name: String,
    pub link: String,
}
