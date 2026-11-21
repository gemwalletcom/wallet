use alloy_sol_types::SolCall;
use gem_evm::contracts::IERC20;
use gem_tron::address::TronAddress;
use primitives::{AssetId, ChainType, decode_hex};

use crate::{Contracts, Deployment, SwapperError, SwapperQuoteData};

pub(super) fn verify_targets(from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let chain = from_asset.chain;
    let is_contract = |address: &str| Deployment::contains(chain, &contracts.targets, address);
    let is_spender = |address: &str| Deployment::contains(chain, &contracts.spenders, address);
    if !is_contract(&data.to) && !token_transfer(from_asset, data).is_some_and(|transfer| is_spender(&recipient(from_asset, &transfer))) {
        return Err(SwapperError::InvalidRoute);
    }
    if let Some(approval) = &data.approval
        && !is_spender(&approval.spender)
    {
        return Err(SwapperError::InvalidRoute);
    }
    if let Some(permit2) = &data.permit2
        && !is_spender(&permit2.spender)
    {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

pub(super) fn token_transfer(from_asset: &AssetId, data: &SwapperQuoteData) -> Option<IERC20::transferCall> {
    from_asset.token_id.as_deref().filter(|token| Deployment::contains(from_asset.chain, &[token], &data.to))?;
    IERC20::transferCall::abi_decode(&decode_hex(&data.data).ok()?).ok()
}

fn recipient(from_asset: &AssetId, transfer: &IERC20::transferCall) -> String {
    match from_asset.chain.chain_type() {
        ChainType::Tron => TronAddress::from(transfer.to).to_string(),
        _ => format!("{:#x}", transfer.to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};
    use num_bigint::BigUint;
    use primitives::{Chain, asset_constants::TRON_USDT_TOKEN_ID, hex};

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

        assert_eq!(verify_targets(&usdt, &Contracts::new(vec![vault]), &transfer(vault)), Ok(()), "a token transfer into the shipped vault is the deposit");
        let router = "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN";
        let routed = Contracts {
            targets: vec![vault, router],
            spenders: vec![vault],
        };
        assert_eq!(
            verify_targets(&usdt, &routed, &transfer(router)),
            Err(SwapperError::InvalidRoute),
            "a token sent to a contract that only runs calls is not a deposit"
        );
        assert_eq!(verify_targets(&usdt, &Contracts::new(vec![vault]), &transfer("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi")), Err(SwapperError::InvalidRoute));
        assert_eq!(
            verify_targets(&AssetId::from_chain(Chain::Tron), &Contracts::new(vec![vault]), &transfer(vault)),
            Err(SwapperError::InvalidRoute),
            "a coin swap may not call a token contract"
        );
    }
}
