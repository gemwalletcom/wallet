use crate::BlockExplorer;
use crate::explorers::metadata::{ADDRESS_PATH, Explorer, Metadata, TRANSACTION_PATH};

pub struct TronScan;

impl TronScan {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "TRONSCAN",
            base_url: "https://tronscan.org/#",
            tx_path: TRANSACTION_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some("/token20"),
            nft_path: None,
            validator_path: Some(ADDRESS_PATH),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tronscan_urls() {
        let explorer = TronScan::boxed();

        assert_eq!(explorer.get_address_url("TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH"), "https://tronscan.org/#/address/TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        assert_eq!(
            explorer.get_tx_url("4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db"),
            "https://tronscan.org/#/transaction/4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db"
        );
    }
}
