pub use gem_evm::siwe::SiweMessage;

#[uniffi::remote(Record)]
pub struct SiweMessage {
    pub domain: String,
    pub address: String,
    pub uri: String,
    pub chain_id: u64,
    pub nonce: String,
    pub version: String,
    pub issued_at: String,
}
