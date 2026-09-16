use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::Metadata;

const NAME: &str = "3xpl";

pub fn new_bitcoin() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://3xpl.com/bitcoin").explorer()
}

pub fn new_bitcoin_cash() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://3xpl.com/bitcoin-cash").explorer()
}

pub fn new_litecoin() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://3xpl.com/litecoin").explorer()
}

pub fn new_doge() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://3xpl.com/dogecoin").explorer()
}

pub fn new_zcash() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://3xpl.com/zcash").explorer()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_xpl_bitcoin() {
        let explorer = new_bitcoin();
        assert_eq!(explorer.name(), "3xpl");
        assert_eq!(explorer.get_tx_url("abc123"), "https://3xpl.com/bitcoin/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://3xpl.com/bitcoin/address/addr123");
    }

    #[test]
    fn test_three_xpl_litecoin() {
        let explorer = new_litecoin();
        assert_eq!(explorer.name(), "3xpl");
        assert_eq!(explorer.get_tx_url("abc123"), "https://3xpl.com/litecoin/transaction/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://3xpl.com/litecoin/address/addr123");
    }
}
