use gem_encoding::protobuf::proto_encode;
use sui_types as sdk;

use crate::rpc::proto::Bcs;

#[derive(Clone, Debug, Default)]
pub struct UserSignature {
    pub bcs: Option<Bcs>,
    pub scheme: Option<i32>,
}

impl UserSignature {
    pub fn from_sdk(signature: sdk::UserSignature) -> Self {
        Self {
            bcs: Some(Bcs::new("UserSignatureBytes", signature.to_bytes())),
            scheme: Some(signature.scheme().to_u8().into()),
        }
    }
}

proto_encode!(UserSignature {
    1 => bcs: optional_message,
    2 => scheme: optional_varint_i32,
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::MessageEncode;
    use primitives::hex;

    #[test]
    fn test_user_signature_from_sdk() {
        let signature = sdk::UserSignature::from_base64("ALmKZNcvdmYgYloqKMAq7eSw5neV1mSEKfZProHEh8Ddw+6aJvLpuViFqZCHqwKdCqtzN8a+7jIDQSxbvmt04QDTaUUhl8KlZIHl4tPovwPeI0n2emMVGVaCIgjCM0re4g==").unwrap();

        assert_eq!(
            hex::encode(UserSignature::from_sdk(signature).encode()),
            "0a770a12557365725369676e61747572654279746573126100b98a64d72f766620625a2a28c02aede4b0e67795d6648429f64fae81c487c0ddc3ee9a26f2e9b95885a99087ab029d0aab7337c6beee3203412c5bbe6b74e100d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee21000"
        );
        assert!(sdk::UserSignature::from_base64("AA==").is_err());
        assert!(sdk::UserSignature::from_base64("").is_err());
    }
}
