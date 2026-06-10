#[cfg(any(feature = "rpc", feature = "signer"))]
mod address;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod cbor;
pub mod models;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod planner;
#[cfg(feature = "rpc")]
pub mod provider;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod transaction;

#[cfg(feature = "rpc")]
pub use provider::map_transaction;
#[cfg(feature = "rpc")]
pub use rpc::client::CardanoClient;

#[cfg(feature = "signer")]
pub mod signer;

#[cfg(feature = "signer")]
pub fn address_from_public_keys(payment_public_key: [u8; 32], stake_public_key: [u8; 32]) -> Result<String, primitives::SignerError> {
    let payment_hash = gem_hash::blake2::blake2b_224(&payment_public_key);
    let stake_hash = gem_hash::blake2::blake2b_224(&stake_public_key);
    address::ShelleyAddress::from_public_key_hashes(payment_hash, stake_hash).encode()
}

#[cfg(feature = "signer")]
pub fn public_key_from_extended_secret(secret: [u8; 32]) -> [u8; 32] {
    signer::extended_key::public_key_from_secret(secret)
}

#[cfg(any(feature = "rpc", feature = "signer"))]
pub fn validate_address(address: &str) -> bool {
    address::ShelleyAddress::parse(address).is_ok()
}
