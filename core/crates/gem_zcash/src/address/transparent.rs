use bitcoin::ScriptBuf;
use gem_bitcoin::signer::address::{p2pkh_script, p2sh_script};
use primitives::SignerError;
use zcash_address::{ConversionError, ToAddress, TryFromAddress, ZcashAddress};
use zcash_protocol::consensus::NetworkType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TransparentAddress {
    P2pkh([u8; 20]),
    P2sh([u8; 20]),
}

impl TransparentAddress {
    pub(super) fn script(self) -> ScriptBuf {
        match self {
            Self::P2pkh(hash) => p2pkh_script(hash),
            Self::P2sh(hash) => p2sh_script(hash),
        }
    }

    pub(super) fn encode(self) -> String {
        let address = match self {
            Self::P2pkh(hash) => ZcashAddress::from_transparent_p2pkh(NetworkType::Main, hash),
            Self::P2sh(hash) => ZcashAddress::from_transparent_p2sh(NetworkType::Main, hash),
        };
        address.encode()
    }
}

impl TryFromAddress for TransparentAddress {
    type Error = SignerError;

    fn try_from_transparent_p2pkh(_net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(Self::P2pkh(hash))
    }

    fn try_from_transparent_p2sh(_net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(Self::P2sh(hash))
    }

    fn try_from_tex(_net: NetworkType, hash: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(Self::P2pkh(hash))
    }
}
