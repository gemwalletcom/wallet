use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TOKEN_PATH, TX_PATH};

pub struct TempoExplorer;

impl TempoExplorer {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Tempo Explorer",
            base_url: "https://explore.tempo.xyz",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            nft_path: None,
            validator_path: None,
        })
    }
}
