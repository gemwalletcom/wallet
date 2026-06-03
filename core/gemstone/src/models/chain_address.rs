use primitives::{Chain, ChainAddress};

#[uniffi::remote(Record)]
pub struct ChainAddress {
    pub chain: Chain,
    pub address: String,
}
