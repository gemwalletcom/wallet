use crate::GemstoneError;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDeviceKeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[uniffi::export]
pub fn generate_device_key_pair() -> GemDeviceKeyPair {
    let seed = gem_crypto::random::bytes::<32>().expect("OS RNG must provide 32 bytes for the device key");
    let public_key = gem_auth::device_public_key(&seed).expect("32 bytes is a valid Ed25519 seed");
    GemDeviceKeyPair {
        private_key: seed.to_vec(),
        public_key: public_key.to_vec(),
    }
}

#[uniffi::export]
pub fn device_public_key(private_key: Vec<u8>) -> Result<Vec<u8>, GemstoneError> {
    Ok(gem_auth::device_public_key(&private_key).map_err(GemstoneError::from)?.to_vec())
}

#[uniffi::export]
pub fn sign_device_auth(private_key: Vec<u8>, method: String, path: String, wallet_id: String, body: Vec<u8>, timestamp_ms: u64) -> Result<String, GemstoneError> {
    gem_auth::build_device_auth_header(&private_key, &method, &path, &wallet_id, &body, timestamp_ms).map_err(GemstoneError::from)
}
