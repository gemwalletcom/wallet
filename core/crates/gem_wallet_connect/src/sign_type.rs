use primitives::Chain;

#[derive(Debug, Clone, PartialEq)]
pub enum SignDigestType {
    Eip191,
    Eip712,
    Base58,
    SuiPersonal,
    Siwe,
    TonPersonal,
    TronPersonal,
    BitcoinEcdsa,
    BitcoinBip322,
}

impl SignDigestType {
    pub fn is_bitcoin(&self) -> bool {
        match self {
            Self::BitcoinEcdsa | Self::BitcoinBip322 => true,
            Self::Eip191 | Self::Eip712 | Self::Base58 | Self::SuiPersonal | Self::Siwe | Self::TonPersonal | Self::TronPersonal => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
