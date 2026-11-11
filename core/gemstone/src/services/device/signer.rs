use std::collections::HashMap;

use primitives::hex;
use primitives::unix_seconds;

use zeroize::Zeroizing;

use crate::config::public::{DEVICE_STREAM_PATH, device_stream_url};
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemDeviceStreamRequest {
    pub url: String,
    pub headers: HashMap<String, String>,
}

pub struct GemDeviceRequestSigner {
    private_key: Zeroizing<Vec<u8>>,
    public_key_hex: String,
}

impl GemDeviceRequestSigner {
    pub fn new(private_key: Vec<u8>) -> Result<Self, GemServiceError> {
        let private_key = Zeroizing::new(private_key);
        let public_key = gem_auth::device_public_key(&private_key).map_err(GemServiceError::core)?;
        Ok(Self {
            private_key,
            public_key_hex: hex::encode(public_key),
        })
    }

    pub fn public_key_hex(&self) -> String {
        self.public_key_hex.clone()
    }

    pub fn device_stream_request(&self) -> Result<GemDeviceStreamRequest, GemServiceError> {
        let created = unix_seconds().map_err(GemServiceError::core)?;
        Ok(GemDeviceStreamRequest {
            url: device_stream_url(),
            headers: gem_auth::sign_device_request(&self.private_key, "GET", DEVICE_STREAM_PATH, None, &[], created).map_err(GemServiceError::core)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_auth::DeviceSignature;

    #[test]
    fn test_device_stream_request_uses_signer_key() {
        let key_pair = crate::device::generate_device_key_pair();
        let signer = GemDeviceRequestSigner::new(key_pair.private_key).unwrap();
        let stream = signer.device_stream_request().unwrap();
        let signature = DeviceSignature::from_fields(|name| stream.headers.get(name).map(String::as_str)).unwrap();

        assert_eq!(signer.public_key_hex(), hex::encode(&key_pair.public_key));
        assert_eq!(signature.device_id, signer.public_key_hex());
        assert_eq!(signature.wallet_id, None);
        assert!(signature.verify("GET", "/v4/devices/stream", None));
        assert_eq!(stream.url, "wss://api.gemwallet.com/v4/devices/stream");
        assert!(GemDeviceRequestSigner::new(vec![1, 2, 3]).is_err());
    }
}
