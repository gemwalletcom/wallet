use bech32::hrp::Hrp;
use primitives::chain_cosmos::CosmosChain;
use primitives::{Address as AddressTrait, Chain};
use std::error::Error;

pub struct CosmosAddress {
    hrp: String,
    bytes: Vec<u8>,
}

impl CosmosAddress {
    pub fn from_public_key_hash(chain: Chain, public_key_hash: [u8; 20]) -> Option<Self> {
        Some(Self {
            hrp: CosmosChain::from_chain(chain)?.hrp().to_string(),
            bytes: public_key_hash.to_vec(),
        })
    }

    fn has_chain_hrp(address: &str, chain: Chain) -> bool {
        let Some(cosmos_chain) = CosmosChain::from_chain(chain) else {
            return false;
        };
        Self::try_parse(address).is_some_and(|address| address.hrp == cosmos_chain.hrp())
    }

    pub fn is_valid_for_chain(address: &str, chain: Chain) -> bool {
        Self::has_chain_hrp(address, chain)
    }

    pub fn convert(address: &str, hrp: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let (_, decoded) = bech32::decode(address)?;
        let new_hrp = Hrp::parse(hrp)?;
        let encoded = bech32::encode::<bech32::Bech32>(new_hrp, decoded.as_slice())?;

        Ok(encoded)
    }
}

impl AddressTrait for CosmosAddress {
    fn try_parse(address: &str) -> Option<Self> {
        let (hrp, bytes) = bech32::decode(address).ok()?;
        let hrp = hrp.as_str().to_string();
        (CosmosChain::all().any(|chain| chain.hrp() == hrp) && bytes.len() == 20).then_some(Self { hrp, bytes })
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn encode(&self) -> String {
        let hrp = Hrp::parse(&self.hrp).expect("valid Cosmos address hrp");
        bech32::encode::<bech32::Bech32>(hrp, &self.bytes).expect("valid Cosmos address bytes")
    }
}

pub fn validate_address(address: &str, chain: Chain) -> bool {
    CosmosAddress::is_valid_for_chain(address, chain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_address_convert() {
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj";
        let expected = "osmosis1h3laqcrmul79zwtw6j63ncsl0adfj07wm8vf00";

        let output_address = CosmosAddress::convert(cosmos_address, "osmosis").unwrap();
        assert_eq!(expected, output_address);
    }

    #[test]
    fn test_invalid_cosmos_address() {
        // invalid checksum
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgu";

        let result = CosmosAddress::convert(cosmos_address, "osmosis");
        assert!(result.is_err());
    }

    #[test]
    fn test_cosmos_address() {
        let address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj";
        let parsed = CosmosAddress::try_parse(address).unwrap();

        assert!(validate_address(address, Chain::Cosmos));
        assert_eq!(parsed.as_bytes().len(), 20);
        assert_eq!(parsed.encode(), address);
        assert!(!validate_address(address, Chain::Osmosis));
        assert!(!validate_address("invalid", Chain::Cosmos));
    }

    #[test]
    fn test_cosmos_address_from_public_key_hash() {
        let public_key_hash = hex::decode("bc7fd0607be7fc51396ed4b519e21f7f5a993fce").unwrap().try_into().unwrap();
        let address = CosmosAddress::from_public_key_hash(Chain::Cosmos, public_key_hash).unwrap();

        assert_eq!(address.encode(), "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj");
    }
}
