use super::{
    constants::{ESTIMATE_SWAP_OUT_METHOD, FACTORY, GET_POOL_ADDRESS_METHOD, UNINITIALIZED_EXIT_CODE},
    model::{DedustAsset, PoolType},
};
use crate::{SwapperError, route_cache::Cache};
use gem_client::Client;
use gem_ton::{address::Address, models::StackArg, rpc::client::TonClient, tvm::TvmError};
use num_bigint::BigUint;
use primitives::{HOUR, WEEK};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PoolKey(PoolType, DedustAsset, DedustAsset);

impl PoolKey {
    fn stack(&self) -> Result<Vec<StackArg>, TvmError> {
        let Self(pool_type, from, to) = self;
        Ok(vec![pool_type.stack_arg(), from.stack_arg()?, to.stack_arg()?])
    }
}

#[derive(Debug)]
pub(super) struct DedustClient<C: Client> {
    ton_client: TonClient<C>,
    addresses: Cache<PoolKey, Address>,
    undeployed_pools: Cache<Address, ()>,
}

impl<C: Client> DedustClient<C> {
    pub fn new(ton_client: TonClient<C>) -> Self {
        Self {
            ton_client,
            addresses: Cache::new(WEEK),
            undeployed_pools: Cache::new(HOUR),
        }
    }

    pub async fn pool_address(&self, pool_type: PoolType, from: &DedustAsset, to: &DedustAsset) -> Result<Address, SwapperError> {
        let key = PoolKey(pool_type, from.clone(), to.clone());
        if let Some(address) = self.addresses.get(&key) {
            return Ok(address);
        }
        let result = self.ton_client.run_get_method(FACTORY, GET_POOL_ADDRESS_METHOD, key.stack()?).await.map_err(SwapperError::compute_quote_error)?;
        let address = result.ok(GET_POOL_ADDRESS_METHOD)?.stack_address(0)?;
        self.addresses.put(key, address);
        Ok(address)
    }

    pub async fn estimate_swap_out(&self, pool: &Address, asset_in: &DedustAsset, amount: &BigUint) -> Result<BigUint, SwapperError> {
        if self.undeployed_pools.get(pool).is_some() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let result = self
            .ton_client
            .run_get_method(&pool.encode_bounceable(), ESTIMATE_SWAP_OUT_METHOD, vec![asset_in.stack_arg()?, StackArg::num(amount.to_string())])
            .await
            .map_err(SwapperError::compute_quote_error)?;
        match result.exit_code {
            0 => {}
            UNINITIALIZED_EXIT_CODE => {
                self.undeployed_pools.put(*pool, ());
                return Err(SwapperError::NoQuoteAvailable);
            }
            _ => return Err(SwapperError::NoQuoteAvailable),
        }
        match result.stack_num(1)? {
            amount_out if amount_out == BigUint::ZERO => Err(SwapperError::NoQuoteAvailable),
            amount_out => Ok(amount_out),
        }
    }

    pub async fn jetton_wallet(&self, owner: &str, jetton: &Address) -> Result<Address, SwapperError> {
        self.ton_client.get_jetton_wallet_address(owner, &jetton.encode_bounceable()).await.map_err(SwapperError::compute_quote_error)
    }
}
