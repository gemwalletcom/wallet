use crate::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TRANSACTION_PATH, TX_PATH};

pub fn new_ton_viewer() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "TonViewer",
        base_url: "https://tonviewer.com",
        tx_path: TRANSACTION_PATH,
        address_path: "",
        token_path: Some(""),
        nft_path: None,
        validator_path: Some(""),
    })
}

pub struct TonScan;

impl TonScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Tonscan",
            base_url: "https://tonscan.org",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some("/jetton"),
            nft_path: None,
            validator_path: Some(ADDRESS_PATH),
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::{TON_USDT_ASSET_ID, TON_USDT_TOKEN_ID};

    use super::*;

    #[test]
    fn test_ton_viewer_urls() {
        let explorer = new_ton_viewer();

        assert_eq!(explorer.get_address_url(TON_USDT_TOKEN_ID), format!("https://tonviewer.com/{TON_USDT_TOKEN_ID}"));
        assert_eq!(explorer.get_token_url(&TON_USDT_ASSET_ID), Some(format!("https://tonviewer.com/{TON_USDT_TOKEN_ID}")));
        assert_eq!(
            explorer.get_tx_url("cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0"),
            "https://tonviewer.com/transaction/cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0"
        );
    }
}
