use alloy_primitives::{Address, U256};
use num_bigint::{BigInt, Sign};
use primitives::contract_constants::ETHEREUM_ACROSS_HUB_POOL_CONTRACT;

use crate::SwapperError;
use gem_evm::{
    across::contracts::HubPoolInterface,
    multicall3::{IMulticall3, create_call3, decode_call3_return},
};

pub(super) struct HubPoolClient {
    contract: String,
}

impl HubPoolClient {
    pub(super) fn new() -> HubPoolClient {
        HubPoolClient {
            contract: ETHEREUM_ACROSS_HUB_POOL_CONTRACT.into(),
        }
    }

    pub(super) fn paused_call3(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::pausedCall {})
    }

    pub(super) fn decoded_paused_call3(&self, result: &IMulticall3::Result) -> Result<bool, SwapperError> {
        let value = decode_call3_return::<HubPoolInterface::pausedCall>(result).map_err(SwapperError::compute_quote_error)?;
        Ok(value)
    }

    pub(super) fn sync_call3(&self, l1token: &Address) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::syncCall { l1Token: *l1token })
    }

    pub(super) fn pooled_token_call3(&self, l1token: &Address) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::pooledTokensCall { l1Token: *l1token })
    }

    pub(super) fn decoded_pooled_token_call3(&self, result: &IMulticall3::Result) -> Result<HubPoolInterface::PooledToken, SwapperError> {
        decode_call3_return::<HubPoolInterface::pooledTokensCall>(result).map_err(SwapperError::compute_quote_error)
    }

    pub(super) fn utilization_call3(&self, l1_token: &Address, amount: U256) -> IMulticall3::Call3 {
        if amount.is_zero() {
            create_call3(&self.contract, HubPoolInterface::liquidityUtilizationCurrentCall { l1Token: *l1_token })
        } else {
            create_call3(
                &self.contract,
                HubPoolInterface::liquidityUtilizationPostRelayCall {
                    l1Token: *l1_token,
                    relayedAmount: amount,
                },
            )
        }
    }

    pub(super) fn decoded_utilization_call3(&self, result: &IMulticall3::Result) -> Result<BigInt, SwapperError> {
        let value = decode_call3_return::<HubPoolInterface::liquidityUtilizationCurrentCall>(result).map_err(SwapperError::compute_quote_error)?;
        Ok(BigInt::from_bytes_le(Sign::Plus, &value.to_le_bytes::<32>()))
    }

    pub(super) fn get_current_time(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, HubPoolInterface::getCurrentTimeCall {})
    }

    pub(super) fn decoded_current_time(&self, result: &IMulticall3::Result) -> Result<u32, SwapperError> {
        let value = decode_call3_return::<HubPoolInterface::getCurrentTimeCall>(result).map_err(SwapperError::compute_quote_error)?;
        value.try_into().map_err(|_| SwapperError::ComputeQuoteError("decode current time failed".into()))
    }
}
