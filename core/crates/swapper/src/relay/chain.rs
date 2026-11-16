use gem_evm::address::ethereum_address_checksum_or_original;
use gem_ton::Address as TonAddress;
use gem_tron::address::TronAddress;
use primitives::{Chain, ChainType, chain_evm::EVMChain};

use crate::{SwapAmountMode, SwapperError};

pub(super) const BITCOIN_CHAIN_ID: u64 = 8253038;
pub(super) const BITCOIN_CURRENCY: &str = "bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmql8k8";

const SOLANA_CHAIN_ID: u64 = 792703809;
pub(super) const TON_CHAIN_ID: u64 = 224235520;
const XRP_CHAIN_ID: u64 = 537724;
pub(super) const XRP_CURRENCY: &str = "xrp";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayChain {
    Evm(EVMChain),
    Bitcoin,
    Tron,
    Solana,
    Ton,
    Xrp,
}

impl RelayChain {
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            Self::Evm(chain) => Some(chain.chain_id()),
            Self::Bitcoin => Some(BITCOIN_CHAIN_ID),
            Self::Tron => Chain::Tron.network_id_value(),
            Self::Solana => Some(SOLANA_CHAIN_ID),
            Self::Ton => Some(TON_CHAIN_ID),
            Self::Xrp => Some(XRP_CHAIN_ID),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        match chain.chain_type() {
            ChainType::Bitcoin if *chain == Chain::Bitcoin => Some(Self::Bitcoin),
            ChainType::Ethereum => Some(Self::Evm(EVMChain::from_chain(*chain)?)),
            ChainType::Tron => Some(Self::Tron),
            ChainType::Solana => Some(Self::Solana),
            ChainType::Ton => Some(Self::Ton),
            ChainType::Xrp => Some(Self::Xrp),
            _ => None,
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Evm(chain) => chain.to_chain(),
            Self::Bitcoin => Chain::Bitcoin,
            Self::Tron => Chain::Tron,
            Self::Solana => Chain::Solana,
            Self::Ton => Chain::Ton,
            Self::Xrp => Chain::Xrp,
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            BITCOIN_CHAIN_ID => Some(Self::Bitcoin),
            SOLANA_CHAIN_ID => Some(Self::Solana),
            TON_CHAIN_ID => Some(Self::Ton),
            XRP_CHAIN_ID => Some(Self::Xrp),
            _ => Self::from_chain(&Chain::from_chain_id(chain_id)?),
        }
    }

    pub fn amount_mode(&self) -> SwapAmountMode {
        match self {
            Self::Bitcoin => SwapAmountMode::Flexible,
            Self::Evm(_) | Self::Tron | Self::Solana | Self::Ton | Self::Xrp => SwapAmountMode::Fixed,
        }
    }

    pub fn uses_deposit_address(&self) -> bool {
        match self {
            Self::Bitcoin => true,
            Self::Evm(_) | Self::Tron | Self::Solana | Self::Ton | Self::Xrp => false,
        }
    }

    pub fn contract_address(&self, address: &str) -> Result<String, SwapperError> {
        match self {
            Self::Tron => Ok(TronAddress::parse_hex_or_base58(address).map_err(|_| SwapperError::InvalidRoute)?.to_string()),
            Self::Ton => Ok(TonAddress::parse(address)?.encode_bounceable()),
            Self::Evm(_) | Self::Bitcoin | Self::Solana | Self::Xrp => Ok(address.to_string()),
        }
    }

    pub fn checksum_address(&self, address: &str) -> String {
        match self {
            Self::Evm(_) => ethereum_address_checksum_or_original(address),
            Self::Ton => TonAddress::try_parse_base64(address).map_or_else(|| address.to_string(), |ton_address| ton_address.encode_non_bounceable()),
            Self::Bitcoin | Self::Tron | Self::Solana | Self::Xrp => address.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum).unwrap().chain_id(), Some(EVMChain::Ethereum.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain).unwrap().chain_id(), Some(EVMChain::SmartChain.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::Robinhood).unwrap().chain_id(), Some(EVMChain::Robinhood.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::Tron), Some(RelayChain::Tron));
        assert_eq!(RelayChain::Tron.chain_id(), Some(728126428));
        assert_eq!(RelayChain::from_chain_id(728126428), Some(RelayChain::Tron));
        assert_eq!(RelayChain::Tron.to_chain(), Chain::Tron);
        assert_eq!(RelayChain::from_chain(&Chain::Solana), Some(RelayChain::Solana));
        assert_eq!(RelayChain::Solana.chain_id(), Some(792703809));
        assert_eq!(RelayChain::from_chain_id(792703809), Some(RelayChain::Solana));
        assert_eq!(RelayChain::Solana.to_chain(), Chain::Solana);
        assert_eq!(RelayChain::from_chain(&Chain::Ton), Some(RelayChain::Ton));
        assert_eq!(RelayChain::Ton.chain_id(), Some(224235520));
        assert_eq!(RelayChain::from_chain_id(224235520), Some(RelayChain::Ton));
        assert_eq!(RelayChain::Ton.to_chain(), Chain::Ton);
        assert_eq!(RelayChain::from_chain(&Chain::Xrp), Some(RelayChain::Xrp));
        assert_eq!(RelayChain::Xrp.chain_id(), Some(537724));
        assert_eq!(RelayChain::from_chain_id(537724), Some(RelayChain::Xrp));
        assert_eq!(RelayChain::Xrp.to_chain(), Chain::Xrp);
        assert!(RelayChain::Bitcoin.uses_deposit_address());
        assert!(!RelayChain::Tron.uses_deposit_address());
        assert_eq!(RelayChain::Bitcoin.amount_mode(), SwapAmountMode::Flexible);
        assert_eq!(RelayChain::Evm(EVMChain::Ethereum).amount_mode(), SwapAmountMode::Fixed);
        assert_eq!(RelayChain::from_chain(&Chain::Litecoin), None);
        assert!(RelayChain::from_chain(&Chain::Cosmos).is_none());
    }
}
