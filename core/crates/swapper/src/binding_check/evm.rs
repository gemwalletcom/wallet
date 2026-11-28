use alloy_sol_types::SolCall;
use gem_evm::contracts::IERC20;
use primitives::{AssetId, decode_hex};

use crate::{Contracts, Deployment, SwapperError, SwapperQuoteData, eth_address};

pub(super) fn verify_evm_call(from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let chain = from_asset.chain;
    let is_contract = |address: &str| Deployment::contains(chain, &contracts.targets, address);
    let is_spender = |address: &str| Deployment::contains(chain, &contracts.spenders, address);
    let pays_deposit_contract = token_transfer(from_asset, data)
        .map(|transfer| eth_address::token_id(chain, &transfer.to))
        .is_some_and(|recipient| is_contract(&recipient) && is_spender(&recipient));
    let approves_spenders = approvals(data).all(|(_, spender)| is_spender(spender));
    if (!is_contract(&data.to) && !pays_deposit_contract) || !approves_spenders {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

pub(super) fn approvals(data: &SwapperQuoteData) -> impl Iterator<Item = (&str, &str)> {
    data.approval
        .iter()
        .map(|approval| (approval.token.as_str(), approval.spender.as_str()))
        .chain(data.permit2.iter().map(|permit2| (permit2.token.as_str(), permit2.spender.as_str())))
}

pub(super) fn token_transfer(from_asset: &AssetId, data: &SwapperQuoteData) -> Option<IERC20::transferCall> {
    from_asset.token_id.as_deref().filter(|token| Deployment::contains(from_asset.chain, &[token], &data.to))?;
    IERC20::transferCall::abi_decode(&decode_hex(&data.data).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};
    use gem_tron::address::TronAddress;
    use num_bigint::BigUint;
    use primitives::{
        Chain,
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, TRON_USDT_TOKEN_ID},
        contract_constants::UNISWAP_PERMIT2_CONTRACT,
        hex,
        swap::{ApprovalData, Permit2ApprovalData},
        testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER},
    };

    #[test]
    fn test_verify_evm_call_spenders() {
        let router = TEST_EVM_RECIPIENT;
        let usdc = AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID);
        let permit2 = UNISWAP_PERMIT2_CONTRACT;
        let contracts = Contracts::universal_router(router, permit2);
        let call = |to: &str, approved: &str, permitted: &str| SwapperQuoteData {
            permit2: Some(Permit2ApprovalData {
                spender: permitted.to_string(),
                ..Permit2ApprovalData::mock()
            }),
            ..SwapperQuoteData::new_contract(to.to_string(), BigUint::ZERO, String::new(), Some(ApprovalData::make(ETHEREUM_USDC_TOKEN_ID, approved, BigUint::from(1u64), false)), None)
        };

        assert_eq!(verify_evm_call(&usdc, &contracts, &call(router, permit2, router)), Ok(()));
        for (data, reason) in [
            (call(permit2, permit2, router), "Permit2 takes approvals, never the swap call"),
            (call(router, TEST_EVM_SENDER, router), "an approval to another spender"),
            (call(router, permit2, TEST_EVM_SENDER), "a Permit2 signature for another spender"),
        ] {
            assert_eq!(verify_evm_call(&usdc, &contracts, &data), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_verify_token_transfer_recipient() {
        let vault = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC";
        let usdt = AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID);
        let transfer = |recipient: &str| {
            let to = TronAddress::from_hex_or_base58(recipient).unwrap();
            let call = IERC20::transferCall {
                to: Address::from(to),
                value: U256::from(1_000_000u64),
            };
            SwapperQuoteData::new_contract(TRON_USDT_TOKEN_ID.to_string(), BigUint::ZERO, hex::encode(call.abi_encode()), None, None)
        };

        assert_eq!(verify_evm_call(&usdt, &Contracts::new(vec![vault]), &transfer(vault)), Ok(()), "a token transfer into the shipped vault is the deposit");
        let router = "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN";
        let routed = Contracts {
            targets: vec![vault, router],
            spenders: vec![vault],
        };
        assert_eq!(
            verify_evm_call(&usdt, &routed, &transfer(router)),
            Err(SwapperError::InvalidRoute),
            "a token sent to a contract that only runs calls is not a deposit"
        );
        let approved = Contracts {
            targets: vec![router],
            spenders: vec![vault],
        };
        assert_eq!(
            verify_evm_call(&usdt, &approved, &transfer(vault)),
            Err(SwapperError::InvalidRoute),
            "a token sent to a contract that only takes approvals is not a deposit"
        );
        assert_eq!(verify_evm_call(&usdt, &Contracts::new(vec![vault]), &transfer("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi")), Err(SwapperError::InvalidRoute));
        assert_eq!(
            verify_evm_call(&AssetId::from_chain(Chain::Tron), &Contracts::new(vec![vault]), &transfer(vault)),
            Err(SwapperError::InvalidRoute),
            "a coin swap may not call a token contract"
        );
    }
}
