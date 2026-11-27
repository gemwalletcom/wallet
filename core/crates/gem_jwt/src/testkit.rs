use gem_encoding::decode_base64;

use crate::{SigningKey, VerifyingKey};

const TEST_ES256_PRIVATE_KEY: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg8kS6eCT+mFBNeFYamocxNgl7+dQsZEeBJVDpyWQ7ycqhRANCAARcuha82eRrrMHqIB/N6DIFT45/ca601g0syLK/gLodiez4eMx3CGkBTu9AIrvzRx/6FJ6xOZnfNV0hsLhADGpb";
const TEST_ES256_PUBLIC_KEY: &str = r#"{"kty":"EC","crv":"P-256","x":"XLoWvNnka6zB6iAfzegyBU-Of3GutNYNLMiyv4C6HYk","y":"7Ph4zHcIaQFO70Aiu_NHH_oUnrE5md81XSGwuEAMals"}"#;

impl SigningKey {
    pub fn mock() -> Self {
        Self::es256(&decode_base64(TEST_ES256_PRIVATE_KEY).unwrap()).unwrap()
    }
}

impl VerifyingKey {
    pub fn mock() -> Self {
        Self::es256(TEST_ES256_PUBLIC_KEY).unwrap()
    }
}
