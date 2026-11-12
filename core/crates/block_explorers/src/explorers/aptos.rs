use crate::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, COIN_PATH, Explorer, Metadata, TXN_PATH};

pub fn new_aptos_explorer() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "AptosExplorer",
        base_url: "https://explorer.aptoslabs.com",
        tx_path: TXN_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        nft_path: None,
        validator_path: None,
    })
}
