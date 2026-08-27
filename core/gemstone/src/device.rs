use std::fmt;

use zeroize::Zeroizing;

use crate::GemstoneError;

#[derive(Clone, uniffi::Record)]
pub struct GemDeviceKeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl fmt::Debug for GemDeviceKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemDeviceKeyPair")
            .field("private_key", &"<redacted>")
            .field("public_key", &self.public_key)
            .finish()
    }
}

#[uniffi::export]
pub fn generate_device_key_pair() -> GemDeviceKeyPair {
    let seed = Zeroizing::new(gem_crypto::random::bytes::<32>().expect("OS RNG must provide 32 bytes for the device key"));
    let public_key = gem_auth::device_public_key(seed.as_slice()).expect("32 bytes is a valid Ed25519 seed");
    GemDeviceKeyPair {
        private_key: seed.to_vec(),
        public_key: public_key.to_vec(),
    }
}

pub fn device_public_key(private_key: Vec<u8>) -> Result<Vec<u8>, GemstoneError> {
    let private_key = Zeroizing::new(private_key);
    Ok(gem_auth::device_public_key(&private_key).map_err(GemstoneError::from)?.to_vec())
}
