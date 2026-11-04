use super::{join_permit2_approvals, permit2_approval};
use crate::{SwapperError, alien::RpcProvider, client_factory::create_client_with_chain, error::INVALID_ADDRESS, eth_address, models::Permit2ApprovalData};
use gem_client::Client;
use gem_evm::u256::u256_to_biguint;
use gem_jsonrpc::client::JsonRpcClient;

use alloy_primitives::{Address, U256, hex::decode as HexDecode};
use alloy_sol_types::SolCall;

use gem_evm::{contracts::erc20::IERC20, jsonrpc::EthereumRpc, permit2::IAllowanceTransfer};
use primitives::{Chain, swap::ApprovalData};
use std::sync::Arc;

pub async fn check_approval_erc20_with_client<C>(owner: String, token: String, spender: String, amount: U256, client: &JsonRpcClient<C>) -> Result<Option<ApprovalData>, SwapperError>
where
    C: Client + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    let owner: Address = owner.as_str().parse().map_err(|_| SwapperError::TransactionError(format!("{}: {owner}", INVALID_ADDRESS)))?;
    let spender: Address = spender.as_str().parse().map_err(|_| SwapperError::TransactionError(format!("{}: {spender}", INVALID_ADDRESS)))?;
    let allowance_data = IERC20::allowanceCall { owner, spender }.abi_encode();
    let allowance_call = EthereumRpc::call(&token, allowance_data);

    let result: String = client.request(allowance_call).await.map_err(SwapperError::from)?;

    let decoded = HexDecode(result).map_err(|_| SwapperError::TransactionError("failed to decode allowance_call result".into()))?;

    let allowance = IERC20::allowanceCall::abi_decode_returns(&decoded).map_err(SwapperError::from)?;

    Ok((allowance < amount).then(|| ApprovalData {
        token,
        spender: spender.to_string(),
        value: u256_to_biguint(&amount),
        is_unlimited: true,
    }))
}

pub async fn check_approval_erc20(owner: String, token: String, spender: String, amount: U256, provider: Arc<dyn RpcProvider>, chain: &Chain) -> Result<Option<ApprovalData>, SwapperError> {
    let client = create_client_with_chain(provider.clone(), *chain)?;
    check_approval_erc20_with_client(owner, token, spender, amount, &client).await
}

async fn check_approval_permit2_with_client<C>(permit2_contract: &str, owner: String, token: String, spender: String, amount: U256, client: &JsonRpcClient<C>) -> Result<Option<Permit2ApprovalData>, SwapperError>
where
    C: Client + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    // Check permit2 allowance, spender is universal router
    let permit2_data = IAllowanceTransfer::allowanceCall {
        _0: eth_address::parse_str(&owner)?,
        _1: eth_address::parse_str(&token)?,
        _2: eth_address::parse_str(&spender)?,
    }
    .abi_encode();
    let permit2_call = EthereumRpc::call(permit2_contract, permit2_data);

    let result: String = client.request(permit2_call).await.map_err(SwapperError::from)?;

    let decoded = HexDecode(result).map_err(|_| SwapperError::TransactionError("failed to decode permit2 allowance result".into()))?;
    let allowance = IAllowanceTransfer::allowanceCall::abi_decode_returns(&decoded).map_err(SwapperError::from)?;
    permit2_approval(&allowance, token, spender, permit2_contract.into(), amount)
}

pub async fn check_permit2_approvals<C>(
    client: &JsonRpcClient<C>,
    owner: &str,
    token: &str,
    amount: U256,
    (permit2_contract, router): (&str, &str),
    signed_permit: bool,
) -> Result<(Option<ApprovalData>, Option<Permit2ApprovalData>), SwapperError>
where
    C: Client + Clone + std::fmt::Debug + Send + Sync + 'static,
{
    join_permit2_approvals(
        check_approval_erc20_with_client(owner.to_string(), token.to_string(), permit2_contract.to_string(), amount, client),
        check_approval_permit2_with_client(permit2_contract, owner.to_string(), token.to_string(), router.to_string(), amount, client),
        signed_permit,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::{MockFn, ProviderMock};
    use primitives::contract_constants::{OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT, UNISWAP_PERMIT2_CONTRACT};

    #[tokio::test]
    async fn test_approval_tx_spender_is_permit2() -> Result<(), SwapperError> {
        let token = "0xdC6fF44d5d932Cbd77B52E5612Ba0529DC6226F1".to_string();
        let owner = "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string();
        let spender = OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.to_string();
        let permit2_contract = UNISWAP_PERMIT2_CONTRACT.to_string();
        let amount = U256::from(1000000000000000000u64);
        let chain: Chain = Chain::Optimism;

        let token_clone = token.clone();
        let mock = ProviderMock {
            response: MockFn(Box::new(move |target| {
                let body = target.body.unwrap();
                let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
                let params = json["params"].as_array().unwrap();
                let param = params[0].as_object().unwrap();
                let to = param["to"].as_str().unwrap();
                if to == token_clone {
                    return r#"{"id":1,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#.to_string();
                }
                r#"{"id":1,"jsonrpc":"2.0","result":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}"#
                    .to_string()
            })),
        };
        let provider = Arc::new(mock);

        let erc20_result = check_approval_erc20(owner.clone(), token.clone(), permit2_contract.clone(), amount, provider.clone(), &chain).await?;
        let client = create_client_with_chain(provider.clone(), chain).unwrap();
        let permit2_result = check_approval_permit2_with_client(&permit2_contract, owner.clone(), token.clone(), spender.clone(), amount, &client).await?;

        assert_eq!(erc20_result, Some(ApprovalData::make(&token, &permit2_contract, u256_to_biguint(&amount), true)));
        assert_eq!(
            permit2_result,
            Some(Permit2ApprovalData {
                token: token.clone(),
                spender: spender.clone(),
                value: u256_to_biguint(&amount),
                permit2_contract,
                permit2_nonce: 0,
            })
        );
        Ok(())
    }
}
