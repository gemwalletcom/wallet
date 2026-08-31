use super::{
    constants::{EVM_NATIVE_TOKEN_ADDRESS, chain_index, dex_ids},
    model::{QuoteData, QuoteParams, SwapParams},
    referral::referrer_wallet_addresses,
};
use crate::{
    QuoteRequest, SwapperError,
    fees::{DEFAULT_AGGREGATOR_FEE_BPS, bps_to_percent_string},
};
use primitives::{
    Chain, ChainType,
    contract_constants::{SOLANA_SYSTEM_PROGRAM_ID, TRON_BLACK_HOLE_ADDRESS},
    swap::{HUNDRED_PERCENT_IN_BPS, QuoteAsset, SlippageMode},
};

const OKX_MAX_SLIPPAGE_BPS_EVM: u32 = HUNDRED_PERCENT_IN_BPS;
const OKX_MAX_SLIPPAGE_BPS_SOLANA: u32 = HUNDRED_PERCENT_IN_BPS - 1;
const MAX_SLIPPAGE_PERCENT_BPS: u32 = 100;

fn limit_slippage_bps(slippage_bps: u32, chain: Chain) -> u32 {
    let max = if chain == Chain::Solana {
        OKX_MAX_SLIPPAGE_BPS_SOLANA
    } else {
        OKX_MAX_SLIPPAGE_BPS_EVM
    };
    slippage_bps.min(max)
}

fn slippage_percent(slippage_bps: u32) -> String {
    bps_to_percent_string(slippage_bps.min(MAX_SLIPPAGE_PERCENT_BPS)).unwrap_or_else(|_| "1".to_string())
}

fn max_auto_slippage_percent(slippage_bps: u32) -> Option<String> {
    bps_to_percent_string(slippage_bps.saturating_mul(2)).ok()
}

pub(super) fn asset_to_token_address(asset: &QuoteAsset) -> Result<String, SwapperError> {
    let asset_id = asset.asset_id();
    if asset_id.chain == Chain::Solana {
        return Ok(asset_id.token_id.unwrap_or_else(|| SOLANA_SYSTEM_PROGRAM_ID.to_string()));
    }
    if asset_id.chain == Chain::Tron {
        return Ok(asset_id.token_id.unwrap_or_else(|| TRON_BLACK_HOLE_ADDRESS.to_string()));
    }
    if asset_id.chain.chain_type() == ChainType::Ethereum {
        return Ok(asset_id.token_id.unwrap_or_else(|| EVM_NATIVE_TOKEN_ADDRESS.to_string()));
    }
    Err(SwapperError::NotSupportedChain)
}

pub(super) fn build_quote_params(request: &QuoteRequest) -> Result<QuoteParams, SwapperError> {
    let chain = request.from_asset.chain();
    Ok(QuoteParams {
        chain_index: chain_index(chain).ok_or(SwapperError::NotSupportedChain)?.to_string(),
        amount: request.value.clone(),
        from_token_address: asset_to_token_address(&request.from_asset)?,
        to_token_address: asset_to_token_address(&request.to_asset)?,
        slippage_percent: slippage_percent(request.options.slippage.bps),
        dex_ids: dex_ids(chain).map(str::to_string),
        fee_percent: bps_to_percent_string(DEFAULT_AGGREGATOR_FEE_BPS)?,
    })
}

pub(super) fn build_swap_params(request: &QuoteRequest, route: &QuoteData) -> Result<SwapParams, SwapperError> {
    let chain = request.from_asset.chain();
    let approve_transaction = chain.chain_type() == ChainType::Ethereum && request.from_asset.asset_id().token_id.is_some();
    let referrers = referrer_wallet_addresses(&request.from_asset, &request.to_asset, chain);
    let is_auto = request.options.slippage.mode == SlippageMode::Auto;
    let slippage_bps = limit_slippage_bps(request.options.slippage.bps, chain);
    Ok(SwapParams {
        chain_index: chain_index(chain).ok_or(SwapperError::NotSupportedChain)?.to_string(),
        amount: request.value.clone(),
        from_token_address: route.from_token.token_contract_address.clone(),
        to_token_address: route.to_token.token_contract_address.clone(),
        user_wallet_address: request.wallet_address.clone(),
        approve_transaction: approve_transaction.then_some(true),
        approve_amount: approve_transaction.then(|| request.value.clone()),
        slippage_percent: Some(slippage_percent(slippage_bps)),
        auto_slippage: Some(is_auto),
        max_auto_slippage_percent: is_auto.then(|| max_auto_slippage_percent(slippage_bps)).flatten(),
        dex_ids: dex_ids(chain).map(str::to_string),
        fee_percent: bps_to_percent_string(DEFAULT_AGGREGATOR_FEE_BPS)?,
        from_token_referrer_wallet_address: referrers.from_token,
        to_token_referrer_wallet_address: referrers.to_token,
    })
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{mock_quote_asset_with_symbol, mock_quote_data};
    use super::*;
    use crate::{SwapperSlippage, fees::default_referral_address, testkit::mock_quote};
    use primitives::{
        AssetId,
        asset_constants::{
            ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID, PLASMA_USDT_ASSET_ID, PLASMA_USDT_TOKEN_ID, ROBINHOOD_USDG_ASSET_ID, ROBINHOOD_USDG_TOKEN_ID, SMARTCHAIN_CAKE_TOKEN_ID,
            SOLANA_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID, TRON_USDT_TOKEN_ID,
        },
    };

    fn mock_request(from_asset: QuoteAsset, to_asset: QuoteAsset, slippage_bps: u32, mode: SlippageMode) -> QuoteRequest {
        let mut request = mock_quote(from_asset, to_asset);
        request.options.slippage = SwapperSlippage { bps: slippage_bps, mode };
        request
    }

    #[test]
    fn test_slippage_percent() {
        assert_eq!(slippage_percent(10), "0.1");
        assert_eq!(slippage_percent(50), "0.5");
        assert_eq!(slippage_percent(100), "1");
        assert_eq!(slippage_percent(500), "1");
    }

    #[test]
    fn test_limit_slippage_bps() {
        assert_eq!(limit_slippage_bps(500, Chain::Ethereum), 500);
        assert_eq!(limit_slippage_bps(500, Chain::Solana), 500);

        assert_eq!(limit_slippage_bps(10_000, Chain::Ethereum), 10_000);
        assert_eq!(limit_slippage_bps(20_000, Chain::Ethereum), 10_000);

        assert_eq!(limit_slippage_bps(10_000, Chain::Solana), 9_999);
        assert_eq!(limit_slippage_bps(9_999, Chain::Solana), 9_999);
    }

    #[test]
    fn test_asset_to_token_address() {
        let sol = AssetId::from_chain(Chain::Solana).to_string();
        let eth = AssetId::from_chain(Chain::Ethereum).to_string();
        let trx = AssetId::from_chain(Chain::Tron).to_string();
        assert_eq!(asset_to_token_address(&mock_quote_asset_with_symbol(&sol, "")).unwrap(), SOLANA_SYSTEM_PROGRAM_ID);
        assert_eq!(asset_to_token_address(&mock_quote_asset_with_symbol(&eth, "")).unwrap(), EVM_NATIVE_TOKEN_ADDRESS);
        assert_eq!(
            asset_to_token_address(&mock_quote_asset_with_symbol(&ETHEREUM_USDC_ASSET_ID.to_string(), "")).unwrap(),
            ETHEREUM_USDC_TOKEN_ID
        );
        assert_eq!(asset_to_token_address(&mock_quote_asset_with_symbol(&trx, "")).unwrap(), TRON_BLACK_HOLE_ADDRESS);
        assert_eq!(
            asset_to_token_address(&mock_quote_asset_with_symbol(&AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID).to_string(), "")).unwrap(),
            TRON_USDT_TOKEN_ID
        );
    }

    #[test]
    fn test_build_swap_params() {
        let eth = AssetId::from_chain(Chain::Ethereum).to_string();
        let evm_request = mock_request(
            mock_quote_asset_with_symbol(&ETHEREUM_USDC_ASSET_ID.to_string(), ""),
            mock_quote_asset_with_symbol(&eth, ""),
            100,
            SlippageMode::Auto,
        );
        let evm_route = mock_quote_data(ETHEREUM_USDC_TOKEN_ID, EVM_NATIVE_TOKEN_ADDRESS);
        let evm_params = build_swap_params(&evm_request, &evm_route).unwrap();
        assert_eq!(evm_params.chain_index, "1");
        assert_eq!(evm_params.approve_transaction, Some(true));
        assert_eq!(evm_params.approve_amount.as_deref(), Some("1000000"));
        assert_eq!(evm_params.fee_percent, "0.7");
        assert_eq!(evm_params.auto_slippage, Some(true));
        assert_eq!(evm_params.dex_ids, None);
        assert_eq!(evm_params.max_auto_slippage_percent.as_deref(), Some("2"));
        assert!(evm_params.to_token_referrer_wallet_address.is_some());
        assert!(evm_params.from_token_referrer_wallet_address.is_none());

        let bnb = AssetId::from_chain(Chain::SmartChain).to_string();
        let cake = AssetId::from_token(Chain::SmartChain, SMARTCHAIN_CAKE_TOKEN_ID).to_string();
        let bsc_request = mock_request(
            mock_quote_asset_with_symbol(&bnb, "BNB"),
            mock_quote_asset_with_symbol(&cake, "CAKE"),
            100,
            SlippageMode::Auto,
        );
        let bsc_route = mock_quote_data(EVM_NATIVE_TOKEN_ADDRESS, SMARTCHAIN_CAKE_TOKEN_ID);
        let bsc_params = build_swap_params(&bsc_request, &bsc_route).unwrap();
        let evm_referrer = default_referral_address(Chain::SmartChain);
        assert_eq!(bsc_params.from_token_referrer_wallet_address.as_deref(), Some(evm_referrer.as_str()));
        assert_eq!(bsc_params.to_token_referrer_wallet_address, None);

        let sol = AssetId::from_chain(Chain::Solana).to_string();
        let sol_request = mock_request(
            mock_quote_asset_with_symbol(&sol, ""),
            mock_quote_asset_with_symbol(&SOLANA_USDC_ASSET_ID.to_string(), ""),
            300,
            SlippageMode::Auto,
        );
        let sol_route = mock_quote_data(SOLANA_SYSTEM_PROGRAM_ID, SOLANA_USDC_TOKEN_ID);
        let sol_params = build_swap_params(&sol_request, &sol_route).unwrap();
        assert_eq!(sol_params.chain_index, "501");
        assert!(sol_params.approve_transaction.is_none());
        assert!(sol_params.approve_amount.is_none());
        assert!(sol_params.dex_ids.is_some());
        assert_eq!(sol_params.fee_percent, "0.7");
        assert!(sol_params.from_token_referrer_wallet_address.is_some());
        assert!(sol_params.to_token_referrer_wallet_address.is_none());

        let trx = AssetId::from_chain(Chain::Tron).to_string();
        let tron_request = mock_request(
            mock_quote_asset_with_symbol(&trx, ""),
            mock_quote_asset_with_symbol(&AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID).to_string(), ""),
            100,
            SlippageMode::Auto,
        );
        let tron_route = mock_quote_data(TRON_BLACK_HOLE_ADDRESS, TRON_USDT_TOKEN_ID);
        let tron_params = build_swap_params(&tron_request, &tron_route).unwrap();
        assert_eq!(tron_params.chain_index, "195");
        assert_eq!(tron_params.fee_percent, "0.7");
        assert_eq!(tron_params.dex_ids.as_deref(), Some("64,98,596"));
        assert!(tron_params.from_token_referrer_wallet_address.is_some());
        assert!(tron_params.to_token_referrer_wallet_address.is_none());

        let robinhood = AssetId::from_chain(Chain::Robinhood).to_string();
        let robinhood_request = mock_request(
            mock_quote_asset_with_symbol(&robinhood, ""),
            mock_quote_asset_with_symbol(&ROBINHOOD_USDG_ASSET_ID.to_string(), ""),
            100,
            SlippageMode::Auto,
        );
        let robinhood_route = mock_quote_data(EVM_NATIVE_TOKEN_ADDRESS, ROBINHOOD_USDG_TOKEN_ID);
        let robinhood_params = build_swap_params(&robinhood_request, &robinhood_route).unwrap();
        assert_eq!(robinhood_params.chain_index, "4663");
        assert_eq!(robinhood_params.dex_ids, None);

        let plasma = AssetId::from_chain(Chain::Plasma).to_string();
        let plasma_request = mock_request(
            mock_quote_asset_with_symbol(&plasma, ""),
            mock_quote_asset_with_symbol(&PLASMA_USDT_ASSET_ID.to_string(), ""),
            100,
            SlippageMode::Auto,
        );
        let plasma_route = mock_quote_data(EVM_NATIVE_TOKEN_ADDRESS, PLASMA_USDT_TOKEN_ID);
        let plasma_params = build_swap_params(&plasma_request, &plasma_route).unwrap();
        assert_eq!(plasma_params.chain_index, "9745");
        assert_eq!(plasma_params.dex_ids, None);
    }

    #[test]
    fn test_build_swap_params_exact_slippage_disables_auto() {
        let eth = AssetId::from_chain(Chain::Ethereum).to_string();
        let evm_request = mock_request(
            mock_quote_asset_with_symbol(&ETHEREUM_USDC_ASSET_ID.to_string(), ""),
            mock_quote_asset_with_symbol(&eth, ""),
            100,
            SlippageMode::Exact,
        );
        let evm_route = mock_quote_data(ETHEREUM_USDC_TOKEN_ID, EVM_NATIVE_TOKEN_ADDRESS);
        let evm_params = build_swap_params(&evm_request, &evm_route).unwrap();

        assert_eq!(evm_params.auto_slippage, Some(false));
        assert_eq!(evm_params.max_auto_slippage_percent, None);
        assert_eq!(evm_params.slippage_percent.as_deref(), Some("1"));
    }

    #[test]
    fn test_build_quote_params() {
        let request = mock_request(
            mock_quote_asset_with_symbol(&AssetId::from_chain(Chain::Solana).to_string(), ""),
            mock_quote_asset_with_symbol(&SOLANA_USDC_ASSET_ID.to_string(), ""),
            300,
            SlippageMode::Auto,
        );
        let params = build_quote_params(&request).unwrap();

        assert_eq!(params.chain_index, "501");
        assert_eq!(params.amount, "1000000");
        assert_eq!(params.from_token_address, SOLANA_SYSTEM_PROGRAM_ID);
        assert_eq!(params.to_token_address, SOLANA_USDC_TOKEN_ID);
        assert_eq!(params.slippage_percent, "1");
        assert!(params.dex_ids.is_some());
        assert_eq!(params.fee_percent, "0.7");
    }
}
