use super::{constants::RouterInfo, quote::PoolData};
use crate::{Quote, SwapperError, route_cache::Cache};
use gem_client::Client;
use gem_ton::{
    address::Address,
    constants::TON_PROXY_JETTON_ADDRESS,
    models::{RunGetMethodResult, StackArg},
    rpc::client::TonClient,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::Address as PrimitiveAddress;
use std::fmt::Debug;

const GET_POOL_ADDRESS_METHOD: &str = "get_pool_address";
const GET_POOL_DATA_METHOD: &str = "get_pool_data";

#[derive(Debug)]
pub(super) struct StonfiClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    ton_client: TonClient<C>,
    jetton_wallet_cache: Cache<(String, String), String>,
}

impl<C> StonfiClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub(super) fn new(ton_client: TonClient<C>) -> Self {
        Self {
            ton_client,
            jetton_wallet_cache: Cache::default(),
        }
    }

    pub(super) async fn router_jetton_wallet(&self, router: &RouterInfo, token: &str) -> Result<String, SwapperError> {
        if token == TON_PROXY_JETTON_ADDRESS {
            return Ok(router.pton_wallet.to_string());
        }
        self.jetton_wallet(router.address, token).await
    }

    pub(super) async fn get_pool_address(&self, router: &RouterInfo, wallet0: &str, wallet1: &str) -> Result<String, SwapperError> {
        let token0 = Address::parse(wallet0)?;
        let token1 = Address::parse(wallet1)?;
        let result = self
            .run_validated_get_method(router.address, GET_POOL_ADDRESS_METHOD, vec![StackArg::slice(token0.to_boc_base64()?), StackArg::slice(token1.to_boc_base64()?)])
            .await?;
        stack_cell_address(&result, 0)
    }

    pub(super) async fn get_pool_data(&self, pool_address: &str) -> Result<PoolData, SwapperError> {
        let result = self.ton_client.run_get_method(pool_address, GET_POOL_DATA_METHOD, Vec::new()).await.map_err(SwapperError::compute_quote_error)?;
        if result.exit_code != 0 {
            return Err(SwapperError::NoQuoteAvailable);
        }
        parse_pool_data(&result)
    }

    pub(super) async fn sender_jetton_wallet(&self, quote: &Quote) -> Result<Option<String>, SwapperError> {
        if quote.request.from_asset.is_native() {
            return Ok(None);
        }
        let token_id = quote.request.from_asset.asset_id().token_id.ok_or(SwapperError::NotSupportedAsset)?;
        Ok(Some(self.jetton_wallet(&quote.request.wallet_address, &token_id).await?))
    }

    async fn jetton_wallet(&self, owner: &str, token: &str) -> Result<String, SwapperError> {
        let key = (owner.to_string(), token.to_string());
        if let Some(wallet) = self.jetton_wallet_cache.get(&key) {
            return Ok(wallet);
        }
        let wallet = PrimitiveAddress::encode(&self.ton_client.get_jetton_wallet_address(owner, token).await.map_err(SwapperError::compute_quote_error)?);
        self.jetton_wallet_cache.put(key, wallet.clone());
        Ok(wallet)
    }

    async fn run_validated_get_method(&self, address: &str, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, SwapperError> {
        let result = self.ton_client.run_get_method(address, method, stack).await.map_err(SwapperError::compute_quote_error)?;
        validate_run_get_method(method, result)
    }
}

fn validate_run_get_method(method: &str, result: RunGetMethodResult) -> Result<RunGetMethodResult, SwapperError> {
    if result.exit_code != 0 {
        return Err(SwapperError::ComputeQuoteError(format!("TON get-method {method} exit code {}", result.exit_code)));
    }
    Ok(result)
}

fn parse_pool_data(result: &RunGetMethodResult) -> Result<PoolData, SwapperError> {
    match result.stack.len() {
        12.. => Ok(PoolData {
            is_locked: result.stack_num(0)? != BigUint::from(0u8),
            reserve0: result.stack_num(3)?,
            reserve1: result.stack_num(4)?,
            token0_wallet: stack_cell_address(result, 5)?,
            token1_wallet: stack_cell_address(result, 6)?,
            lp_fee: stack_num_u32(result, 7)?,
            protocol_fee: stack_num_u32(result, 8)?,
        }),
        10.. => Ok(PoolData {
            is_locked: false,
            reserve0: result.stack_num(0)?,
            reserve1: result.stack_num(1)?,
            token0_wallet: stack_cell_address(result, 2)?,
            token1_wallet: stack_cell_address(result, 3)?,
            lp_fee: stack_num_u32(result, 4)?,
            protocol_fee: stack_num_u32(result, 5)?,
        }),
        _ => Err(SwapperError::ComputeQuoteError("STON.fi get_pool_data returned truncated stack".into())),
    }
}

fn stack_cell_address(result: &RunGetMethodResult, index: usize) -> Result<String, SwapperError> {
    Ok(PrimitiveAddress::encode(&result.stack_address(index)?))
}

fn stack_num_u32(result: &RunGetMethodResult, index: usize) -> Result<u32, SwapperError> {
    result.stack_num(index)?.to_u32().ok_or_else(|| SwapperError::ComputeQuoteError("TON stack number does not fit u32".into()))
}
