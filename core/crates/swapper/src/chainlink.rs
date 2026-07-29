use num_bigint::BigInt;
use num_traits::FromBytes;

use crate::SwapperError;
use gem_evm::{
    chainlink::contract::AggregatorInterface,
    multicall3::{IMulticall3, create_call3, decode_call3_return},
};
use primitives::{
    Chain,
    contract_constants::{ETHEREUM_CHAINLINK_ETH_USD_FEED_CONTRACT, MONAD_CHAINLINK_USD_FEED_CONTRACT},
};

pub(super) struct ChainlinkPriceFeed {
    contract: String,
}

impl ChainlinkPriceFeed {
    pub(super) fn new(chain: Chain) -> Self {
        Self {
            contract: match chain {
                Chain::Monad => MONAD_CHAINLINK_USD_FEED_CONTRACT,
                _ => ETHEREUM_CHAINLINK_ETH_USD_FEED_CONTRACT,
            }
            .into(),
        }
    }

    pub(super) fn latest_round_call3(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, AggregatorInterface::latestRoundDataCall {})
    }

    // Price is in 8 decimals
    pub(super) fn decoded_answer(result: &IMulticall3::Result) -> Result<BigInt, SwapperError> {
        let decoded = decode_call3_return::<AggregatorInterface::latestRoundDataCall>(result).map_err(|_| SwapperError::ComputeQuoteError("failed to decode answer".into()))?;
        Ok(BigInt::from_le_bytes(&decoded.answer.to_le_bytes::<32>()))
    }
}
