use bitcoin::ScriptBuf;
use gem_bitcoin::signer::SpendDestination;
use orchard::Address as OrchardAddress;
use primitives::SignerError;
use zcash_address::{
    ConversionError, TryFromAddress,
    unified::{self, Container, Receiver},
};
use zcash_protocol::consensus::NetworkType;

use super::transparent::TransparentAddress;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ZcashDestination {
    Transparent(ScriptBuf),
    Shielded(OrchardAddress),
}

impl ZcashDestination {
    pub(crate) fn into_spend(self) -> (SpendDestination, Option<OrchardAddress>) {
        match self {
            Self::Transparent(script_pubkey) => (SpendDestination::Script(script_pubkey), None),
            Self::Shielded(recipient) => (SpendDestination::External, Some(recipient)),
        }
    }

    fn from_receiver(receiver: &Receiver) -> Option<Result<Self, SignerError>> {
        match receiver {
            Receiver::Orchard(bytes) => Some(
                OrchardAddress::from_raw_address_bytes(bytes)
                    .into_option()
                    .map(Self::Shielded)
                    .ok_or_else(|| SignerError::invalid_input("invalid Zcash Orchard receiver")),
            ),
            Receiver::P2pkh(hash) => Some(Ok(TransparentAddress::P2pkh(*hash).into())),
            Receiver::P2sh(hash) => Some(Ok(TransparentAddress::P2sh(*hash).into())),
            Receiver::Sapling(_) | Receiver::Unknown { .. } => None,
        }
    }
}

impl From<TransparentAddress> for ZcashDestination {
    fn from(address: TransparentAddress) -> Self {
        Self::Transparent(address.script())
    }
}

impl TryFromAddress for ZcashDestination {
    type Error = SignerError;

    fn try_from_unified(_net: NetworkType, address: unified::Address) -> Result<Self, ConversionError<Self::Error>> {
        address
            .items()
            .iter()
            .find_map(Self::from_receiver)
            .unwrap_or_else(|| Err(SignerError::invalid_input("unsupported Zcash unified address")))
            .map_err(ConversionError::User)
    }

    fn try_from_transparent_p2pkh(net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        TransparentAddress::try_from_transparent_p2pkh(net, hash).map(Self::from)
    }

    fn try_from_transparent_p2sh(net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        TransparentAddress::try_from_transparent_p2sh(net, hash).map(Self::from)
    }

    fn try_from_tex(net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        TransparentAddress::try_from_tex(net, hash).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use bech32::{Bech32, Bech32m, Hrp};
    use gem_bitcoin::testkit::address_mock::mock_sender_address;
    use primitives::{
        BitcoinChain,
        testkit::zcash_mock::{TEST_ZCASH_TEX_ADDRESS, TEST_ZCASH_TRANSPARENT_ADDRESS},
    };

    use super::*;
    use crate::{
        address::ParseAddress,
        testkit::{
            TEST_ZCASH_UNIFIED_ADDRESS, TEST_ZCASH_UNIFIED_ORCHARD_ADDRESS, TEST_ZCASH_UNIFIED_SAPLING_ADDRESS, TEST_ZCASH_UNIFIED_SAPLING_TRANSPARENT_ADDRESS, TEST_ZCASH_UNIFIED_TESTNET_ADDRESS, mock_orchard_recipient,
            mock_transparent_script,
        },
    };

    #[test]
    fn test_parse() {
        let transparent_script = mock_transparent_script(TEST_ZCASH_TRANSPARENT_ADDRESS);
        let p2sh_script = mock_transparent_script("t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd");
        let fallback_script = mock_transparent_script(&mock_sender_address(BitcoinChain::Zcash));
        let cases = [
            (TEST_ZCASH_UNIFIED_ADDRESS.to_string(), ZcashDestination::Shielded(mock_orchard_recipient())),
            (TEST_ZCASH_UNIFIED_ORCHARD_ADDRESS.to_string(), ZcashDestination::Shielded(mock_orchard_recipient())),
            (TEST_ZCASH_UNIFIED_SAPLING_TRANSPARENT_ADDRESS.to_string(), ZcashDestination::Transparent(fallback_script)),
            (TEST_ZCASH_TRANSPARENT_ADDRESS.to_string(), ZcashDestination::Transparent(transparent_script.clone())),
            ("t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd".to_string(), ZcashDestination::Transparent(p2sh_script)),
            (TEST_ZCASH_TEX_ADDRESS.to_string(), ZcashDestination::Transparent(transparent_script.clone())),
            (TEST_ZCASH_TEX_ADDRESS.to_uppercase(), ZcashDestination::Transparent(transparent_script)),
        ];
        for (address, expected) in cases {
            assert_eq!(ZcashDestination::parse(&address).unwrap(), expected, "{address}");
        }

        assert_eq!(ZcashDestination::parse(TEST_ZCASH_UNIFIED_SAPLING_ADDRESS).unwrap_err(), SignerError::invalid_input("unsupported Zcash unified address"));
        assert_eq!(ZcashDestination::parse(TEST_ZCASH_UNIFIED_TESTNET_ADDRESS).unwrap_err(), SignerError::invalid_input("unsupported Zcash network"));
        assert_eq!(
            ZcashDestination::parse(&bech32::encode::<Bech32>(Hrp::parse("zs").unwrap(), &[0; 43]).unwrap()).unwrap_err(),
            SignerError::invalid_input("unsupported Zcash address")
        );
    }

    #[test]
    fn test_parse_rejects_invalid_tex() {
        let tex = Hrp::parse("tex").unwrap();
        let addresses = [
            bech32::encode::<Bech32>(tex, &[0; 20]).unwrap(),
            bech32::encode::<Bech32m>(Hrp::parse("tex1other").unwrap(), &[0; 20]).unwrap(),
            bech32::encode::<Bech32m>(tex, &[0; 19]).unwrap(),
            bech32::encode::<Bech32m>(tex, &[0; 21]).unwrap(),
            "tex1s2rt77ggv6q989lr49rkgzmh5slsksa9khdgtq".to_string(),
            "tEX1S2RT77GGV6Q989LR49RKGZMH5SLSKSA9KHDGTE".to_string(),
        ];
        for address in addresses {
            assert!(ZcashDestination::parse(&address).is_err(), "{address}");
        }
        assert_eq!(
            ZcashDestination::parse(&bech32::encode::<Bech32m>(Hrp::parse("textest").unwrap(), &[0; 20]).unwrap()).unwrap_err(),
            SignerError::invalid_input("unsupported Zcash network")
        );
    }
}
