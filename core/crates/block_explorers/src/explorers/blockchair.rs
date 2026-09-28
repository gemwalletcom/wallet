use crate::BlockExplorer;
use crate::explorers::metadata::Metadata;

const NAME: &str = "Blockchair";

pub fn new_bitcoin() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/bitcoin").explorer()
}

pub fn new_bitcoin_cash() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/bitcoin-cash").explorer()
}

pub fn new_litecoin() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/litecoin").explorer()
}

pub fn new_doge() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/dogecoin").explorer()
}

pub fn new_dash() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/dash").explorer()
}

pub fn new_zcash() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/zcash").explorer()
}

pub fn new_ethereum() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/ethereum").explorer()
}

pub fn new_base() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/base").explorer()
}

pub fn new_polygon() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/polygon").explorer()
}

pub fn new_arbitrum() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/arbitrum-one").explorer()
}

pub fn new_optimism() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/optimism").explorer()
}

pub fn new_avalanche() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/avalanche").explorer()
}

pub fn new_solana() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/solana").explorer()
}

pub fn new_stellar() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/stellar").explorer()
}

pub fn new_bnb() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/bnb").explorer()
}

pub fn new_opbnb() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/opbnb").explorer()
}

pub fn new_fantom() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/fantom").explorer()
}

pub fn new_gnosis() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/gnosis-chain").explorer()
}

pub fn new_linea() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/linea").explorer()
}

pub fn new_ton() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/ton").explorer()
}

pub fn new_tron() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/tron").explorer()
}

pub fn new_xrp() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/xrp-ledger").explorer()
}

pub fn new_aptos() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/aptos").explorer()
}

pub fn new_polkadot() -> Box<dyn BlockExplorer> {
    Metadata::blockchair(NAME, "https://blockchair.com/polkadot").explorer()
}
