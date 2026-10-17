use super::{
    constants::{ESTIMATE_SWAP_OUT_METHOD, FACTORY, GET_POOL_ADDRESS_METHOD, GET_VAULT_ADDRESS_METHOD, VOLATILE_POOL_TYPE},
    model::DedustAsset,
};
use crate::{SwapperError, route_cache::Cache};
use gem_client::Client;
use gem_ton::{
    address::Address,
    models::{RunGetMethodResult, StackArg},
    rpc::client::TonClient,
};
use num_bigint::BigUint;

#[derive(Debug)]
pub(super) struct DedustClient<C: Client> {
    ton_client: TonClient<C>,
    pool_cache: Cache<(DedustAsset, DedustAsset), Address>,
}

impl<C: Client> DedustClient<C> {
    pub fn new(ton_client: TonClient<C>) -> Self {
        Self { ton_client, pool_cache: Cache::default() }
    }

    pub async fn pool_address(&self, from: &DedustAsset, to: &DedustAsset) -> Result<Address, SwapperError> {
        let key = (from.clone(), to.clone());
        if let Some(pool) = self.pool_cache.get(&key) {
            return Ok(pool);
        }
        let pool = self
            .run_factory_method(GET_POOL_ADDRESS_METHOD, vec![StackArg::num(VOLATILE_POOL_TYPE), from.stack_arg()?, to.stack_arg()?])
            .await?
            .stack_address(0)?;
        self.pool_cache.put(key, pool);
        Ok(pool)
    }

    pub async fn vault_address(&self, asset: &DedustAsset) -> Result<Address, SwapperError> {
        Ok(self.run_factory_method(GET_VAULT_ADDRESS_METHOD, vec![asset.stack_arg()?]).await?.stack_address(0)?)
    }

    pub async fn estimate_swap_out(&self, pool: &Address, asset_in: &DedustAsset, amount: &BigUint) -> Result<BigUint, SwapperError> {
        let result = self
            .ton_client
            .run_get_method(&pool.encode_bounceable(), ESTIMATE_SWAP_OUT_METHOD, vec![asset_in.stack_arg()?, StackArg::num(amount.to_string())])
            .await
            .map_err(SwapperError::compute_quote_error)?;
        if result.exit_code != 0 {
            return Err(SwapperError::NoQuoteAvailable);
        }
        match result.stack_num(1)? {
            amount_out if amount_out == BigUint::ZERO => Err(SwapperError::NoQuoteAvailable),
            amount_out => Ok(amount_out),
        }
    }

    pub async fn jetton_wallet(&self, owner: &str, jetton: &Address) -> Result<Address, SwapperError> {
        self.ton_client.get_jetton_wallet_address(owner, &jetton.encode_bounceable()).await.map_err(SwapperError::compute_quote_error)
    }

    async fn run_factory_method(&self, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, SwapperError> {
        let result = self.ton_client.run_get_method(FACTORY, method, stack).await.map_err(SwapperError::compute_quote_error)?;
        match result.exit_code {
            0 => Ok(result),
            exit_code => Err(SwapperError::ComputeQuoteError(format!("DeDust factory {method} exit code {exit_code}"))),
        }
    }
}
