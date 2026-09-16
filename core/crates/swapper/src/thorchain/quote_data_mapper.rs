use num_bigint::BigUint;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolCall;
use primitives::swap::ApprovalData;

use super::{asset::THORChainAsset, contracts::RouterInterface, deposit_gas_limit, model::RouteData};
use crate::{SwapperError, SwapperQuoteData, approval::get_swap_gas_limit_with_approval};

const EXPIRY_SECONDS: u64 = 86400;

pub fn map_quote_data(
    from_asset: &THORChainAsset,
    route_data: &RouteData,
    token_id: Option<String>,
    value: BigUint,
    memo: String,
    approval: Option<ApprovalData>,
) -> Result<SwapperQuoteData, SwapperError> {
    let gas_limit = get_swap_gas_limit_with_approval(&approval, None, deposit_gas_limit(&memo));

    if from_asset.use_evm_router() {
        let router_address = route_data.router_address.clone().unwrap_or_default();
        let inbound_address = Address::from_str(&route_data.inbound_address).map_err(|_| SwapperError::InvalidRoute)?;
        let token_id = token_id.ok_or(SwapperError::NotSupportedAsset)?;
        let token_address = Address::from_str(&token_id).map_err(|_| SwapperError::NotSupportedAsset)?;
        let amount = U256::from_str(&value.to_string()).map_err(SwapperError::transaction_error)?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() + EXPIRY_SECONDS;
        let expiry = U256::from(timestamp);

        let call_data = RouterInterface::depositWithExpiryCall {
            inbound_address,
            token_address,
            amount,
            memo,
            expiry,
        }
        .abi_encode();

        Ok(SwapperQuoteData::new_contract(router_address, BigUint::ZERO, HexEncode(call_data), approval, gas_limit))
    } else if from_asset.chain.is_evm_chain() {
        Ok(SwapperQuoteData::new_contract(
            route_data.inbound_address.clone(),
            value,
            HexEncode(memo.as_bytes()),
            approval,
            gas_limit,
        ))
    } else {
        Ok(SwapperQuoteData::new_transfer(route_data.inbound_address.clone(), value, Some(memo)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{Chain, asset_constants::ETHEREUM_USDC_TOKEN_ID, swap::ApprovalData};

    #[test]
    fn evm_router() {
        let usdc_eth = ETHEREUM_USDC_TOKEN_ID.to_string();
        let result = map_quote_data(
            &THORChainAsset {
                token_id: Some(usdc_eth.clone()),
                ..THORChainAsset::mock(Chain::Ethereum)
            },
            &RouteData {
                router_address: Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string()),
                inbound_address: "0x1234567890123456789012345678901234567890".to_string(),
            },
            Some(usdc_eth),
            BigUint::from(1000000u64),
            "memo".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.to, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146");
        assert_eq!(result.value, BigUint::from(0u64));
        assert!(result.data.starts_with("0x"));
        assert_eq!(result.memo, None);
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn evm_native() {
        let result = map_quote_data(
            &THORChainAsset::mock(Chain::Ethereum),
            &RouteData {
                router_address: Some("0xrouter".to_string()),
                inbound_address: "0xinbound".to_string(),
            },
            None,
            BigUint::from(1000u64),
            "memo".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.to, "0xinbound");
        assert_eq!(result.value, BigUint::from(1000u64));
        assert_eq!(result.data, "0x6d656d6f");
        assert_eq!(result.memo, None);
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn non_evm() {
        let result = map_quote_data(
            &THORChainAsset::mock(Chain::Bitcoin),
            &RouteData {
                router_address: None,
                inbound_address: "bc1q".to_string(),
            },
            None,
            BigUint::from(1_000u64),
            "memo".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.to, "bc1q");
        assert_eq!(result.value, BigUint::from(1000u64));
        assert_eq!(result.data, "");
        assert_eq!(result.memo, Some("memo".to_string()));
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn zcash_native() {
        let result = map_quote_data(
            &THORChainAsset::mock(Chain::Zcash),
            &RouteData {
                router_address: None,
                inbound_address: "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML".to_string(),
            },
            None,
            BigUint::from(10000000u64),
            "=:b:bc1qdestination:0/1/0:g1:50".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.to, "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML");
        assert_eq!(result.value, BigUint::from(10000000u64));
        assert_eq!(result.data, "");
        assert_eq!(result.memo, Some("=:b:bc1qdestination:0/1/0:g1:50".to_string()));
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn evm_router_with_approval() {
        let usdc_eth = ETHEREUM_USDC_TOKEN_ID.to_string();
        let approval = Some(ApprovalData::make(&usdc_eth, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146", BigUint::from(2000u64), false));

        let result = map_quote_data(
            &THORChainAsset {
                token_id: Some(usdc_eth.clone()),
                ..THORChainAsset::mock(Chain::Ethereum)
            },
            &RouteData {
                router_address: Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string()),
                inbound_address: "0x1234567890123456789012345678901234567890".to_string(),
            },
            Some(usdc_eth),
            BigUint::from(1000000u64),
            "memo".to_string(),
            approval.clone(),
        )
        .unwrap();

        assert_eq!(result.to, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146");
        assert_eq!(result.value, BigUint::from(0u64));
        assert_eq!(result.approval, approval);
        assert_eq!(result.gas_limit, Some(deposit_gas_limit("memo").to_string()));
    }

    #[test]
    fn a_longer_memo_raises_the_deposit_gas_limit() {
        let short = deposit_gas_limit("=:ETH.ETH:0xabc");
        let long = deposit_gas_limit("=:ETH.ETH:0xabc:100000000/1/0:gem:20");

        assert!(long > short);
        assert_eq!(long - short, ("=:ETH.ETH:0xabc:100000000/1/0:gem:20".len() - "=:ETH.ETH:0xabc".len()) as u64 * 16);
        assert_eq!(deposit_gas_limit(""), 90_000);
    }

    #[test]
    fn evm_native_without_approval() {
        let result = map_quote_data(
            &THORChainAsset::mock(Chain::Ethereum),
            &RouteData {
                router_address: Some("0xrouter".to_string()),
                inbound_address: "0xinbound".to_string(),
            },
            None,
            BigUint::from(1000u64),
            "memo".to_string(),
            None,
        )
        .unwrap();

        assert_eq!(result.to, "0xinbound");
        assert_eq!(result.value, BigUint::from(1000u64));
        assert_eq!(result.gas_limit, None);
    }
}
