use primitives::unix_milliseconds;

use zeroize::Zeroizing;

use crate::services::error::GemServiceError;

#[derive(uniffi::Object)]
pub struct GemDeviceRequestSigner {
    private_key: Zeroizing<Vec<u8>>,
    public_key_hex: String,
}

#[uniffi::export]
impl GemDeviceRequestSigner {
    #[uniffi::constructor]
    pub fn new(private_key: Vec<u8>) -> Result<Self, GemServiceError> {
        let private_key = Zeroizing::new(private_key);
        let public_key = gem_auth::device_public_key(&private_key).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        Ok(Self {
            private_key,
            public_key_hex: hex::encode(public_key),
        })
    }

    pub fn sign(&self, method: String, path: String, wallet_id: String, body: Vec<u8>) -> Result<String, GemServiceError> {
        let timestamp_ms = unix_milliseconds().map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        gem_auth::build_device_auth_header(&self.private_key, &method, &path, &wallet_id, &body, timestamp_ms).map_err(|error| GemServiceError::Core { msg: error.to_string() })
    }
}

impl GemDeviceRequestSigner {
    pub fn public_key_hex(&self) -> String {
        self.public_key_hex.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};

    #[test]
    fn test_sign_header_uses_signer_key() {
        let key_pair = crate::device::generate_device_key_pair();
        let signer = GemDeviceRequestSigner::new(key_pair.private_key).unwrap();
        assert_eq!(signer.public_key_hex(), hex::encode(&key_pair.public_key));

        let header = signer.sign("GET".into(), "/v2/devices".into(), "wallet".into(), Vec::new()).unwrap();
        let payload = STANDARD.decode(header.strip_prefix("Gem ").unwrap()).unwrap();
        let parts: Vec<&str> = std::str::from_utf8(&payload).unwrap().splitn(5, '.').collect();
        assert_eq!(parts[0], signer.public_key_hex());
        assert_eq!(parts[2], "wallet");
        assert_eq!(parts[3].len(), 64);
        assert!(GemDeviceRequestSigner::new(vec![1, 2, 3]).is_err());
    }
}
