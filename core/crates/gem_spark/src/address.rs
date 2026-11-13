use bech32::primitives::iter::{ByteIterExt, Fe32IterExt};
use bech32::{Bech32m, Hrp};

pub const IDENTITY_PATH: &str = "m/8797555'/1'/0'";
const PUBLIC_KEY_LEN: usize = 33;
const IDENTITY_FIELD_TAG: u8 = 0x0a;
const HRP: Hrp = Hrp::parse_unchecked("spark");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparkAddress {
    identity_public_key: [u8; PUBLIC_KEY_LEN],
}

impl SparkAddress {
    pub fn new(identity_public_key: [u8; PUBLIC_KEY_LEN]) -> Self {
        Self { identity_public_key }
    }

    pub fn encode(&self) -> String {
        [IDENTITY_FIELD_TAG, PUBLIC_KEY_LEN as u8]
            .into_iter()
            .chain(self.identity_public_key)
            .bytes_to_fes()
            .with_checksum::<Bech32m>(&HRP)
            .chars()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use primitives::hex::decode_hex;

    use super::*;
    use crate::testkit::IDENTITY_PUBLIC_KEY;

    fn public_key() -> [u8; PUBLIC_KEY_LEN] {
        decode_hex(IDENTITY_PUBLIC_KEY).unwrap().try_into().unwrap()
    }

    #[test]
    fn test_encode() {
        assert_eq!(SparkAddress::new(public_key()).encode(), "spark1pgss9qfk8ygtphqqzkj2yhn43k3s7r3g8z822ffvpcm38ym094800574x5numh");
    }

    #[cfg(feature = "wallet")]
    #[test]
    fn test_matches_spark_encoding() {
        let key = spark_wallet::PublicKey::from_slice(&public_key()).unwrap();
        let expected = spark_wallet::SparkAddress::new(key, spark_wallet::Network::Mainnet, None).to_address_string().unwrap();

        assert_eq!(SparkAddress::new(key.serialize()).encode(), expected);
    }
}
