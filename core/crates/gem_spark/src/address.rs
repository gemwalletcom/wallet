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
