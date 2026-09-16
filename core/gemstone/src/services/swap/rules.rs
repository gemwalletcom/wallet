use crate::formatted_number::GemFormattedNumber;
use num_bigint::BigInt;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::swap::{SwapProviderData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId, Chain, Wallet};
use swapper::permit2_data::{Permit2Detail, PermitSingle};
use swapper::{AssetList, Options, Permit2ApprovalData, Quote, QuoteRequest, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode};

use crate::config::swap_config::{SwapConfig, get_default_slippage};
use crate::models::swap::GemSlippageCheck;
use crate::services::swap::model::{
    GemAssetRate, GemSwapButtonAction, GemSwapButtonInput, GemSwapPair, GemSwapPairSelection, GemSwapPairSuggestion, GemSwapRate, GemSwapSide, GemSwapTransfer,
};
use std::collections::HashMap;

pub fn quote_request(wallet: &Wallet, from_asset: &Asset, to_asset: &Asset, value: BigUint, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<QuoteRequest, SwapperError> {
    let wallet_address = account_address(wallet, from_asset.chain())?;
    let destination_address = account_address(wallet, to_asset.chain())?;
    Ok(QuoteRequest {
        from_asset: quote_asset(from_asset),
        to_asset: quote_asset(to_asset),
        wallet_address,
        destination_address,
        value,
        options: Options {
            slippage: slippage(from_asset, slippage_bps),
            use_max_amount,
        },
    })
}

const BASIS_POINTS: u32 = 10_000;
const BPS_PER_PERCENT: f64 = 100.0;

pub fn selected_quote(quotes: &[Quote], preferred: Option<SwapperProvider>) -> Option<Quote> {
    quotes.iter().find(|quote| Some(quote.data.provider.id) == preferred).or_else(|| quotes.first()).cloned()
}

pub fn min_receive_value(value: &BigUint, slippage_bps: u32) -> BigUint {
    let kept = BASIS_POINTS.saturating_sub(slippage_bps);
    value * BigUint::from(kept) / BigUint::from(BASIS_POINTS)
}

pub fn slippage_bps_from_percent(percent: f64) -> Option<u32> {
    match percent > 0.0 && percent.is_finite() {
        true => Some((percent * BPS_PER_PERCENT).round() as u32),
        false => None,
    }
}

pub fn amount_for_percent(available: &BigInt, percent: u32) -> BigInt {
    available * BigInt::from(percent) / BigInt::from(100u32)
}

pub fn slippage_percent(bps: u32) -> f64 {
    f64::from(bps) / BPS_PER_PERCENT
}

pub fn slippage_check(bps: u32, config: &SwapConfig) -> GemSlippageCheck {
    if bps < config.min_slippage_bps {
        GemSlippageCheck::BelowMinimum
    } else if bps > config.max_slippage_bps {
        GemSlippageCheck::AboveMaximum
    } else if bps >= config.high_slippage_warning_bps {
        GemSlippageCheck::High
    } else {
        GemSlippageCheck::Valid
    }
}

pub fn swap_transfer(wallet: &Wallet, quote: &Quote, data: SwapQuoteData) -> Result<GemSwapTransfer, SwapperError> {
    let to_chain = AssetId::new(&quote.request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?.chain;
    Ok(GemSwapTransfer {
        quote: swap_quote(quote),
        data,
        recipient: account_address(wallet, to_chain)?,
        value: quote.request.value.clone(),
        use_max_amount: quote.request.options.use_max_amount,
    })
}

const QUOTE_REFRESH_INTERVAL_MILLISECONDS: u64 = 30_000;
const QUOTE_DEBOUNCE_MILLISECONDS: u64 = 250;

pub fn quote_refresh_interval_milliseconds() -> u64 {
    QUOTE_REFRESH_INTERVAL_MILLISECONDS
}

pub fn quote_debounce_milliseconds() -> u64 {
    QUOTE_DEBOUNCE_MILLISECONDS
}

pub fn swap_rate(from_asset: &Asset, from_value: &BigUint, to_asset: &Asset, to_value: &BigUint) -> Option<GemSwapRate> {
    let from_amount = amount(from_value, from_asset.decimals)?;
    let to_amount = amount(to_value, to_asset.decimals)?;
    (from_amount > 0.0 && to_amount > 0.0).then(|| GemSwapRate {
        direct: asset_rate(from_asset, to_asset, to_amount / from_amount),
        inverse: asset_rate(to_asset, from_asset, from_amount / to_amount),
    })
}

fn amount(value: &BigUint, decimals: i32) -> Option<f64> {
    BigNumberFormatter::value_as_f64(&value.to_string(), u32::try_from(decimals).ok()?).ok()
}

fn asset_rate(base: &Asset, quote: &Asset, value: f64) -> GemAssetRate {
    GemAssetRate {
        base_symbol: base.symbol.clone(),
        quote_symbol: quote.symbol.clone(),
        value: GemFormattedNumber::adaptive(value, Some(quote.symbol.clone())),
    }
}

pub fn swap_quote(quote: &Quote) -> SwapQuote {
    SwapQuote {
        from_address: quote.request.wallet_address.clone(),
        from_value: quote.from_value.clone(),
        min_from_value: quote.min_from_value.clone(),
        to_address: quote.request.destination_address.clone(),
        to_value: quote.to_value.clone(),
        provider_data: SwapProviderData {
            provider: quote.data.provider.id,
            name: quote.data.provider.name.clone(),
            protocol_name: quote.data.provider.protocol.clone(),
        },
        slippage_bps: quote.data.slippage_bps,
        eta_in_seconds: quote.eta_in_seconds,
        use_max_amount: Some(quote.request.options.use_max_amount),
    }
}

pub fn permit_single(approval: &Permit2ApprovalData, now: u64, config: &SwapConfig) -> PermitSingle {
    PermitSingle {
        details: Permit2Detail {
            token: approval.token.clone(),
            amount: approval.value.to_string(),
            expiration: now + config.permit2_expiration,
            nonce: approval.permit2_nonce,
        },
        spender: approval.spender.clone(),
        sig_deadline: now + config.permit2_sig_deadline,
    }
}

fn slippage(from_asset: &Asset, slippage_bps: Option<u32>) -> SwapperSlippage {
    match slippage_bps {
        Some(bps) => SwapperSlippage {
            bps,
            mode: SwapperSlippageMode::Exact,
        },
        None => SwapperSlippage {
            mode: SwapperSlippageMode::Auto,
            ..get_default_slippage(&from_asset.chain())
        },
    }
}

fn quote_asset(asset: &Asset) -> SwapperQuoteAsset {
    SwapperQuoteAsset {
        id: asset.id.to_string(),
        symbol: asset.symbol.clone(),
        decimals: asset.decimals as u32,
        asset_type: asset.asset_type.clone(),
    }
}

fn account_address(wallet: &Wallet, chain: Chain) -> Result<String, SwapperError> {
    wallet
        .accounts
        .iter()
        .find(|account| account.chain == chain)
        .map(|account| account.address.clone())
        .ok_or(SwapperError::NotSupportedChain)
}

pub fn most_swapped_receive_asset(pairs: &[GemSwapPair], pay_asset_id: &AssetId) -> Option<AssetId> {
    let received: Vec<&GemSwapPair> = pairs.iter().filter(|pair| &pair.to_asset_id != pay_asset_id).collect();
    let received_for_pay_asset: Vec<AssetId> = received
        .iter()
        .filter(|pair| &pair.from_asset_id == pay_asset_id)
        .map(|pair| pair.to_asset_id.clone())
        .collect();
    most_frequent_asset(&received_for_pay_asset).or_else(|| {
        let received: Vec<AssetId> = received.iter().map(|pair| pair.to_asset_id.clone()).collect();
        most_frequent_asset(&received)
    })
}

fn most_frequent_asset(asset_ids: &[AssetId]) -> Option<AssetId> {
    let mut counts: HashMap<&AssetId, usize> = HashMap::new();
    for asset_id in asset_ids {
        *counts.entry(asset_id).or_default() += 1;
    }
    asset_ids.iter().min_by_key(|asset_id| std::cmp::Reverse(counts[*asset_id])).cloned()
}

pub fn first_other_asset(asset_ids: Vec<AssetId>, pay_asset_id: &AssetId) -> Option<AssetId> {
    asset_ids.into_iter().find(|asset_id| asset_id != pay_asset_id)
}

pub fn assets_in_wallet(supported: AssetList, wallet: &Wallet) -> AssetList {
    let has_account = |chain: &Chain| wallet.accounts.iter().any(|account| &account.chain == chain);
    AssetList {
        chains: supported.chains.into_iter().filter(has_account).collect(),
        asset_ids: supported.asset_ids.into_iter().filter(|asset_id| has_account(&asset_id.chain)).collect(),
    }
}

impl GemSwapButtonInput {
    pub fn action(&self) -> GemSwapButtonAction {
        if let Some(minimum) = minimum_amount(self.quote_error.as_ref()) {
            if minimum > self.available_balance {
                return GemSwapButtonAction::InsufficientBalance;
            }
            return GemSwapButtonAction::UseMinimumAmount { value: minimum };
        }
        if is_retryable(self.transfer_error.as_ref()) {
            return GemSwapButtonAction::RetryTransfer;
        }
        if is_retryable(self.quote_error.as_ref()) {
            return GemSwapButtonAction::RetryQuote;
        }
        if self.value > self.available_balance {
            return GemSwapButtonAction::InsufficientBalance;
        }
        GemSwapButtonAction::Swap
    }
}

pub fn is_retryable(error: Option<&SwapperError>) -> bool {
    match error {
        Some(SwapperError::NoQuoteAvailable | SwapperError::ComputeQuoteError(_) | SwapperError::TransactionError(_)) => true,
        Some(
            SwapperError::NotSupportedChain
            | SwapperError::NotSupportedAsset
            | SwapperError::NoAvailableProvider
            | SwapperError::InvalidRoute
            | SwapperError::InputAmountError { .. },
        )
        | None => false,
    }
}

pub fn minimum_amount(error: Option<&SwapperError>) -> Option<BigInt> {
    let SwapperError::InputAmountError { min_amount } = error? else {
        return None;
    };
    let minimum = min_amount.as_ref()?.parse::<BigInt>().ok()?;
    (minimum > BigInt::from(0)).then_some(minimum)
}

pub fn select_pair_asset(selection: GemSwapPairSelection, side: GemSwapSide, asset_id: AssetId) -> GemSwapPairSelection {
    let (chosen, other) = match side {
        GemSwapSide::Pay => (selection.pay_asset_id, selection.receive_asset_id),
        GemSwapSide::Receive => (selection.receive_asset_id, selection.pay_asset_id),
    };
    let other = match other.as_ref() == Some(&asset_id) {
        true => chosen,
        false => other,
    };
    match side {
        GemSwapSide::Pay => GemSwapPairSelection {
            pay_asset_id: Some(asset_id),
            receive_asset_id: other,
        },
        GemSwapSide::Receive => GemSwapPairSelection {
            pay_asset_id: other,
            receive_asset_id: Some(asset_id),
        },
    }
}

pub fn pair_for_asset(asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
    let pays_with_native = asset_id.is_token() && !has_balance && asset_id.chain.has_native_asset();
    if pays_with_native {
        return GemSwapPairSuggestion {
            pay_asset_id: AssetId::from_chain(asset_id.chain),
            receive_asset_id: Some(asset_id),
        };
    }
    GemSwapPairSuggestion {
        pay_asset_id: asset_id,
        receive_asset_id: None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_selected_quote_prefers_the_chosen_provider_then_the_best() {
        let quotes = vec![
            Quote::mock_with_provider(SwapperProvider::Okx, "1"),
            Quote::mock_with_provider(SwapperProvider::Jupiter, "1"),
        ];

        assert_eq!(selected_quote(&quotes, Some(SwapperProvider::Jupiter)).unwrap().data.provider.id, SwapperProvider::Jupiter);
        assert_eq!(selected_quote(&quotes, Some(SwapperProvider::Thorchain)).unwrap().data.provider.id, SwapperProvider::Okx);
        assert_eq!(selected_quote(&quotes, None).unwrap().data.provider.id, SwapperProvider::Okx);
        assert!(selected_quote(&[], Some(SwapperProvider::Jupiter)).is_none());
    }

    #[test]
    fn test_swap_rate_pairs_each_direction_and_needs_both_amounts() {
        let eth = Asset::mock_eth();
        let usdc = Asset::mock_ethereum_usdc();
        let one_eth = BigUint::from(1_000_000_000_000_000_000u128);
        let two_thousand_usdc = BigUint::from(2_000_000_000u64);

        let rate = swap_rate(&eth, &one_eth, &usdc, &two_thousand_usdc).unwrap();
        assert_eq!(
            (rate.direct.base_symbol.as_str(), rate.direct.quote_symbol.as_str(), rate.direct.value.value),
            ("ETH", "USDC", 2000.0)
        );
        assert_eq!(
            (rate.inverse.base_symbol.as_str(), rate.inverse.quote_symbol.as_str(), rate.inverse.value.value),
            ("USDC", "ETH", 0.0005)
        );

        assert!(swap_rate(&eth, &BigUint::from(0u32), &usdc, &two_thousand_usdc).is_none());
        assert!(swap_rate(&eth, &one_eth, &usdc, &BigUint::from(0u32)).is_none());
    }

    #[test]
    fn test_min_receive_value_keeps_the_slippage_share() {
        let value = BigUint::from(1_000_000u32);
        assert_eq!(min_receive_value(&value, 0), value);
        assert_eq!(min_receive_value(&value, 100), BigUint::from(990_000u32));
        assert_eq!(min_receive_value(&value, BASIS_POINTS), BigUint::from(0u32));
        assert_eq!(min_receive_value(&value, BASIS_POINTS + 1), BigUint::from(0u32));
    }

    #[test]
    fn test_a_percent_button_takes_that_share_of_the_balance() {
        let available = BigInt::from(1_000_000_000u64);
        assert_eq!(amount_for_percent(&available, 100), available);
        assert_eq!(amount_for_percent(&available, 50), BigInt::from(500_000_000u64));
        assert_eq!(amount_for_percent(&available, 25), BigInt::from(250_000_000u64));
        assert_eq!(amount_for_percent(&available, 0), BigInt::from(0u32));
        assert_eq!(
            amount_for_percent(&BigInt::from(3u32), 50),
            BigInt::from(1u32),
            "a share that does not divide evenly rounds down, never up past the balance"
        );
    }

    #[test]
    fn test_a_slippage_percent_rounds_to_the_nearest_basis_point() {
        assert_eq!(slippage_bps_from_percent(1.0), Some(100));
        assert_eq!(slippage_bps_from_percent(0.5), Some(50));
        assert_eq!(slippage_bps_from_percent(0.125), Some(13), "half a basis point rounds up rather than truncating");
        assert_eq!(slippage_bps_from_percent(0.0), None);
        assert_eq!(slippage_bps_from_percent(-1.0), None);
        assert_eq!(slippage_bps_from_percent(f64::NAN), None);
        assert_eq!(slippage_percent(250), 2.5);
    }

    #[test]
    fn test_slippage_check_rejects_the_bounds_and_warns_from_the_threshold() {
        let config = SwapConfig {
            min_slippage_bps: 10,
            max_slippage_bps: 1_000,
            high_slippage_warning_bps: 500,
            ..crate::config::swap_config::get_swap_config()
        };
        assert_eq!(slippage_check(9, &config), GemSlippageCheck::BelowMinimum);
        assert_eq!(slippage_check(10, &config), GemSlippageCheck::Valid);
        assert_eq!(slippage_check(499, &config), GemSlippageCheck::Valid);
        assert_eq!(slippage_check(500, &config), GemSlippageCheck::High);
        assert_eq!(slippage_check(1_000, &config), GemSlippageCheck::High);
        assert_eq!(slippage_check(1_001, &config), GemSlippageCheck::AboveMaximum);
    }

    #[test]
    fn test_choosing_the_other_side_s_asset_swaps_the_pair() {
        let eth = AssetId::from_chain(Chain::Ethereum);
        let btc = AssetId::from_chain(Chain::Bitcoin);
        let sol = AssetId::from_chain(Chain::Solana);
        let pair = GemSwapPairSelection {
            pay_asset_id: Some(eth.clone()),
            receive_asset_id: Some(btc.clone()),
        };

        let swapped = select_pair_asset(pair.clone(), GemSwapSide::Pay, btc.clone());
        assert_eq!(
            (swapped.pay_asset_id, swapped.receive_asset_id),
            (Some(btc.clone()), Some(eth.clone())),
            "paying with what was being received turns the pair around instead of emptying a side"
        );

        let swapped_back = select_pair_asset(pair.clone(), GemSwapSide::Receive, eth.clone());
        assert_eq!((swapped_back.pay_asset_id, swapped_back.receive_asset_id), (Some(btc.clone()), Some(eth.clone())));

        let replaced = select_pair_asset(pair, GemSwapSide::Pay, sol.clone());
        assert_eq!(
            (replaced.pay_asset_id, replaced.receive_asset_id),
            (Some(sol.clone()), Some(btc)),
            "any other asset only replaces the side it was chosen for"
        );

        let first = select_pair_asset(
            GemSwapPairSelection {
                pay_asset_id: None,
                receive_asset_id: None,
            },
            GemSwapSide::Receive,
            sol.clone(),
        );
        assert_eq!((first.pay_asset_id, first.receive_asset_id), (None, Some(sol)));
    }

    #[test]
    fn test_pair_for_asset_pays_with_the_native_asset_only_when_the_token_is_unheld() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let usdc = AssetId::from_token(Chain::Ethereum, "0xusdc");

        assert_eq!(
            pair_for_asset(usdc.clone(), false),
            GemSwapPairSuggestion {
                pay_asset_id: ethereum.clone(),
                receive_asset_id: Some(usdc.clone()),
            }
        );
        assert_eq!(
            pair_for_asset(usdc.clone(), true),
            GemSwapPairSuggestion {
                pay_asset_id: usdc,
                receive_asset_id: None,
            }
        );
        assert_eq!(
            pair_for_asset(ethereum.clone(), false),
            GemSwapPairSuggestion {
                pay_asset_id: ethereum,
                receive_asset_id: None,
            }
        );
    }

    use super::*;
    use crate::models::custom_types::GemBigInt;
    use primitives::{Account, AssetId, Chain};

    #[test]
    fn test_quote_request_uses_wallet_accounts_and_slippage() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "ethereum-address"), Account::mock(Chain::Solana, "solana-address")]);
        let request = quote_request(
            &wallet,
            &Asset::from_chain(Chain::Ethereum),
            &Asset::from_chain(Chain::Solana),
            BigUint::from(100u32),
            true,
            Some(50),
        )
        .unwrap();
        assert_eq!(request.wallet_address, "ethereum-address");
        assert_eq!(request.destination_address, "solana-address");
        assert_eq!(
            request.options.slippage,
            SwapperSlippage {
                bps: 50,
                mode: SwapperSlippageMode::Exact
            }
        );
        assert!(request.options.use_max_amount);

        let auto = quote_request(
            &wallet,
            &Asset::from_chain(Chain::Ethereum),
            &Asset::from_chain(Chain::Solana),
            BigUint::from(100u32),
            false,
            None,
        )
        .unwrap();
        assert_eq!(auto.options.slippage.mode, SwapperSlippageMode::Auto);
        assert_eq!(auto.options.slippage.bps, get_default_slippage(&Chain::Ethereum).bps);
    }

    #[test]
    fn test_quote_request_requires_accounts() {
        let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
        assert!(matches!(
            quote_request(
                &wallet,
                &Asset::from_chain(Chain::Ethereum),
                &Asset::from_chain(Chain::Solana),
                BigUint::from(1u32),
                false,
                None
            ),
            Err(SwapperError::NotSupportedChain)
        ));
    }

    #[test]
    fn test_swap_transfer_maps_quote_and_recipient() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "ethereum-address"), Account::mock(Chain::Solana, "solana-address")]);
        let request = quote_request(
            &wallet,
            &Asset::from_chain(Chain::Ethereum),
            &Asset::from_chain(Chain::Solana),
            BigUint::from(100u32),
            true,
            Some(50),
        )
        .unwrap();
        let quote = Quote {
            from_value: BigUint::from(99u64),
            min_from_value: Some(BigUint::from(90u64)),
            request,
            eta_in_seconds: Some(30),
            ..Quote::mock_with_provider(SwapperProvider::Jupiter, "1")
        };
        let data = SwapQuoteData::mock_contract_call("0xrouter", "100", "0x", Some("swap-memo"));

        let transfer = swap_transfer(&wallet, &quote, data.clone()).unwrap();

        let transfer_data = transfer.transfer_data(Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana));
        assert_eq!(transfer_data.recipient.address, "solana-address");
        assert_eq!(transfer_data.recipient.memo.as_deref(), Some("swap-memo"));
        assert_eq!(transfer_data.value, num_bigint::BigInt::from(100u64));
        assert!(transfer_data.use_max_amount);
        assert!(matches!(&transfer_data.input_type, primitives::TransactionInputType::Swap { swap_data, .. } if swap_data.data == data));

        assert_eq!(transfer.recipient, "solana-address");
        assert_eq!(transfer.value, BigUint::from(100u64));
        assert!(transfer.use_max_amount);
        assert_eq!(transfer.data, data);
        assert_eq!(transfer.quote.from_address, "ethereum-address");
        assert_eq!(transfer.quote.to_address, "solana-address");
        assert_eq!(transfer.quote.from_value, BigUint::from(99u64));
        assert_eq!(transfer.quote.min_from_value, Some(BigUint::from(90u64)));
        assert_eq!(transfer.quote.provider_data.provider, swapper::SwapperProvider::Jupiter);
        assert_eq!(transfer.quote.slippage_bps, 50);
        assert_eq!(transfer.quote.use_max_amount, Some(true));

        let ethereum_only = Wallet::mock_with_chains(&[Chain::Ethereum]);
        assert!(matches!(swap_transfer(&ethereum_only, &quote, data), Err(SwapperError::NotSupportedChain)));
    }

    #[test]
    fn test_permit_single_deadlines() {
        let approval = Permit2ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xspender".to_string(),
            value: BigUint::from(1u64),
            permit2_contract: "0xpermit2".to_string(),
            permit2_nonce: 7,
        };
        let config = crate::config::swap_config::get_swap_config();
        let permit = permit_single(&approval, 1_000, &config);
        assert_eq!(permit.details.expiration, 1_000 + config.permit2_expiration);
        assert_eq!(permit.sig_deadline, 1_000 + config.permit2_sig_deadline);
        assert_eq!(permit.details.nonce, 7);
        assert_eq!(permit.spender, "0xspender");
    }

    #[test]
    fn test_most_swapped_receive_asset_prefers_the_pay_asset_history() {
        let pairs = [
            GemSwapPair::mock(Chain::Ethereum, Chain::Solana),
            GemSwapPair::mock(Chain::Ethereum, Chain::Solana),
            GemSwapPair::mock(Chain::Ethereum, Chain::Bitcoin),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
        ];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_falls_back_to_the_overall_history() {
        let pairs = [
            GemSwapPair::mock(Chain::Bitcoin, Chain::Solana),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Solana),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
        ];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_keeps_the_first_seen_on_a_tie() {
        let pairs = [GemSwapPair::mock(Chain::Ethereum, Chain::Bitcoin), GemSwapPair::mock(Chain::Ethereum, Chain::Solana)];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Bitcoin))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_never_suggests_the_pay_asset() {
        let pairs = [
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Ethereum),
            GemSwapPair::mock(Chain::Bitcoin, Chain::Solana),
        ];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_is_none_without_history() {
        assert_eq!(most_swapped_receive_asset(&[], &AssetId::from_chain(Chain::Ethereum)), None);
    }

    #[test]
    fn test_button_action_offers_the_minimum_amount_only_when_the_balance_covers_it() {
        for (min_amount, action) in [
            (Some("18900023"), GemSwapButtonAction::UseMinimumAmount { value: BigInt::from(18_900_023) }),
            (Some("22000000"), GemSwapButtonAction::InsufficientBalance),
            (Some("0"), GemSwapButtonAction::Swap),
            (None, GemSwapButtonAction::Swap),
        ] {
            let input = GemSwapButtonInput {
                quote_error: Some(SwapperError::InputAmountError {
                    min_amount: min_amount.map(str::to_string),
                }),
                ..GemSwapButtonInput::mock(100, 18_900_023)
            };
            assert_eq!(input.action(), action);
        }
    }

    #[test]
    fn test_button_action_blocks_an_unaffordable_amount_before_any_quote() {
        assert_eq!(GemSwapButtonInput::mock(101, 100).action(), GemSwapButtonAction::InsufficientBalance);
        assert_eq!(GemSwapButtonInput::mock(100, 100).action(), GemSwapButtonAction::Swap);
        assert_eq!(GemSwapButtonInput::mock(0, 100).action(), GemSwapButtonAction::Swap);
    }

    #[test]
    fn test_swap_button_input_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemSwapButtonInput) -> (GemBigInt, GemBigInt) = |input| (input.value, input.available_balance);
        assert_eq!(GemSwapButtonInput::mock(101, 100).value, BigInt::from(101));
    }

    #[test]
    fn test_button_action_retries_only_a_retryable_error() {
        for (error, action) in [
            (SwapperError::NoQuoteAvailable, GemSwapButtonAction::RetryQuote),
            (SwapperError::NotSupportedAsset, GemSwapButtonAction::Swap),
            (SwapperError::NoAvailableProvider, GemSwapButtonAction::Swap),
            (SwapperError::InvalidRoute, GemSwapButtonAction::Swap),
        ] {
            let input = GemSwapButtonInput {
                quote_error: Some(error),
                ..GemSwapButtonInput::mock(100, 100)
            };
            assert_eq!(input.action(), action);
        }
        for (error, action) in [
            (SwapperError::TransactionError("nonce".to_string()), GemSwapButtonAction::RetryTransfer),
            (SwapperError::NotSupportedChain, GemSwapButtonAction::Swap),
        ] {
            let input = GemSwapButtonInput {
                transfer_error: Some(error),
                ..GemSwapButtonInput::mock(100, 100)
            };
            assert_eq!(input.action(), action);
        }
    }

    #[test]
    fn test_button_action_prefers_the_minimum_amount_then_the_transfer_retry() {
        let input = GemSwapButtonInput {
            quote_error: Some(SwapperError::NoQuoteAvailable),
            transfer_error: Some(SwapperError::NoQuoteAvailable),
            ..GemSwapButtonInput::mock(101, 100)
        };
        assert_eq!(input.action(), GemSwapButtonAction::RetryTransfer);

        let with_minimum = GemSwapButtonInput {
            quote_error: Some(SwapperError::InputAmountError {
                min_amount: Some("50".to_string()),
            }),
            ..input
        };
        assert_eq!(with_minimum.action(), GemSwapButtonAction::UseMinimumAmount { value: BigInt::from(50) });
    }

    #[test]
    fn test_is_retryable_covers_every_error() {
        assert!(is_retryable(Some(&SwapperError::NoQuoteAvailable)));
        assert!(is_retryable(Some(&SwapperError::ComputeQuoteError("boom".to_string()))));
        assert!(is_retryable(Some(&SwapperError::TransactionError("boom".to_string()))));
        assert!(!is_retryable(Some(&SwapperError::NotSupportedChain)));
        assert!(!is_retryable(Some(&SwapperError::NotSupportedAsset)));
        assert!(!is_retryable(Some(&SwapperError::NoAvailableProvider)));
        assert!(!is_retryable(Some(&SwapperError::InvalidRoute)));
        assert!(!is_retryable(Some(&SwapperError::InputAmountError { min_amount: None })));
        assert!(!is_retryable(None));
    }

    #[test]
    fn test_first_other_asset_skips_the_pay_asset() {
        let asset_ids = vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Solana)];

        assert_eq!(
            first_other_asset(asset_ids, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_first_other_asset_is_none_when_only_the_pay_asset_is_available() {
        let asset_ids = vec![AssetId::from_chain(Chain::Ethereum)];

        assert_eq!(first_other_asset(asset_ids, &AssetId::from_chain(Chain::Ethereum)), None);
    }

    #[test]
    fn test_assets_in_wallet_drops_chains_without_an_account() {
        let assets = assets_in_wallet(AssetList::mock(), &Wallet::mock_with_chains(&[Chain::Tron]));

        assert_eq!(assets.chains, vec![Chain::Tron]);
        assert_eq!(assets.asset_ids, vec![AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()))]);
    }

    #[test]
    fn test_assets_in_wallet_keeps_every_chain_the_wallet_has() {
        let supported = AssetList::mock();
        let assets = assets_in_wallet(supported.clone(), &Wallet::mock_with_chains(&[Chain::Tron, Chain::Bitcoin, Chain::Ethereum]));

        assert_eq!(assets.chains, supported.chains);
        assert_eq!(assets.asset_ids, supported.asset_ids);
    }
}
