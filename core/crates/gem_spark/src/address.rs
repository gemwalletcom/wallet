use bech32::primitives::iter::{ByteIterExt, Fe32IterExt};
use bech32::{Bech32m, Hrp};

use crate::error::LightningError;
use crate::model::LightningNetwork;

const IDENTITY_FIELD_TAG: u8 = 0x0a;
const PUBLIC_KEY_LEN: usize = 33;
const HRP_MAINNET: Hrp = Hrp::parse_unchecked("spark");
const HRP_REGTEST: Hrp = Hrp::parse_unchecked("sparkrt");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparkAddress {
    identity_public_key: [u8; PUBLIC_KEY_LEN],
    network: LightningNetwork,
}

impl SparkAddress {
    pub fn new(identity_public_key: [u8; PUBLIC_KEY_LEN], network: LightningNetwork) -> Self {
        Self { identity_public_key, network }
    }

    pub fn from_public_key(public_key: &[u8], network: LightningNetwork) -> Result<Self, LightningError> {
        let identity_public_key = public_key.try_into().map_err(|_| LightningError::Key("invalid identity public key".to_string()))?;
        Ok(Self::new(identity_public_key, network))
    }

    pub fn encode(&self) -> String {
        let hrp = match self.network {
            LightningNetwork::Mainnet => HRP_MAINNET,
            LightningNetwork::Regtest => HRP_REGTEST,
        };
        [IDENTITY_FIELD_TAG, PUBLIC_KEY_LEN as u8]
            .into_iter()
            .chain(self.identity_public_key)
            .bytes_to_fes()
            .with_checksum::<Bech32m>(&hrp)
            .chars()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use primitives::hex::decode_hex;

    use super::*;

    const PUBLIC_KEY: &str = "0281363910b0dc0015a4a25e758da30f0e28388ea5252c0e3713936f2d4ef7d3d5";

    #[test]
    fn test_encode() {
        let public_key = decode_hex(PUBLIC_KEY).unwrap();
        let address = SparkAddress::from_public_key(&public_key, LightningNetwork::Mainnet).unwrap();

        assert_eq!(address.encode(), "spark1pgss9qfk8ygtphqqzkj2yhn43k3s7r3g8z822ffvpcm38ym094800574x5numh");
        assert!(SparkAddress::from_public_key(&public_key[1..], LightningNetwork::Mainnet).is_err());
    }

    #[cfg(feature = "wallet")]
    #[test]
    fn test_matches_spark_encoding() {
        let public_key = decode_hex(PUBLIC_KEY).unwrap();
        let key = spark_wallet::PublicKey::from_slice(&public_key).unwrap();
        let expected = spark_wallet::SparkAddress::new(key, spark_wallet::Network::Mainnet, None).to_address_string().unwrap();

        assert_eq!(SparkAddress::from_public_key(&public_key, LightningNetwork::Mainnet).unwrap().encode(), expected);
        assert_eq!(SparkAddress::new(key.serialize(), LightningNetwork::Mainnet).encode(), expected);
    }
}
