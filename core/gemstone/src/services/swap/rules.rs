use num_bigint::BigInt;
use primitives::swap::{SwapProviderData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId, Chain, Wallet};
use swapper::permit2_data::{Permit2Detail, PermitSingle};
use swapper::{Options, Permit2ApprovalData, Quote, QuoteRequest, SwapperError, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode};

use crate::config::swap_config::{SwapConfig, get_default_slippage};
use crate::services::swap::model::{GemSwapButtonAction, GemSwapButtonInput, GemSwapPair, GemSwapPairSuggestion, GemSwapTransfer};
use std::collections::HashMap;

pub fn quote_request(wallet: &Wallet, from_asset: &Asset, to_asset: &Asset, value: String, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<QuoteRequest, SwapperError> {
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

pub fn sort_quotes(mut quotes: Vec<Quote>) -> Vec<Quote> {
    quotes.sort_by_key(|quote| std::cmp::Reverse(to_value(quote)));
    quotes
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
            amount: approval.value.clone(),
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

fn to_value(quote: &Quote) -> BigInt {
    quote.to_value.parse().unwrap_or_default()
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

pub fn button_action(input: &GemSwapButtonInput) -> GemSwapButtonAction {
    if let Some(minimum) = minimum_amount(input.quote_error.as_ref()) {
        if minimum > input.available_balance {
            return GemSwapButtonAction::InsufficientBalance;
        }
        return GemSwapButtonAction::UseMinimumAmount { value: minimum };
    }
    if is_retryable(input.transfer_error.as_ref()) {
        return GemSwapButtonAction::RetryTransfer;
    }
    if is_retryable(input.quote_error.as_ref()) {
        return GemSwapButtonAction::RetryQuote;
    }
    if input.value > input.available_balance {
        return GemSwapButtonAction::InsufficientBalance;
    }
    GemSwapButtonAction::Swap
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

fn minimum_amount(error: Option<&SwapperError>) -> Option<BigInt> {
    let SwapperError::InputAmountError { min_amount } = error? else {
        return None;
    };
    let minimum = min_amount.as_ref()?.parse::<BigInt>().ok()?;
    (minimum > BigInt::from(0)).then_some(minimum)
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
    use primitives::{Account, AssetId, AssetType, Chain};

    fn wallet(chains: &[Chain]) -> Wallet {
        Wallet::mock_with_accounts(chains.iter().map(|chain| Account::mock(*chain, &format!("{chain}-address"))).collect())
    }

    fn asset(chain: Chain) -> Asset {
        Asset::new(AssetId::from_chain(chain), chain.to_string(), chain.to_string().to_uppercase(), 18, AssetType::NATIVE)
    }

    #[test]
    fn test_quote_request_uses_wallet_accounts_and_slippage() {
        let wallet = wallet(&[Chain::Ethereum, Chain::Solana]);
        let request = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), true, Some(50)).unwrap();
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

        let auto = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), false, None).unwrap();
        assert_eq!(auto.options.slippage.mode, SwapperSlippageMode::Auto);
        assert_eq!(auto.options.slippage.bps, get_default_slippage(&Chain::Ethereum).bps);
    }

    #[test]
    fn test_quote_request_requires_accounts() {
        let wallet = wallet(&[Chain::Ethereum]);
        assert!(matches!(
            quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "1".to_string(), false, None),
            Err(SwapperError::NotSupportedChain)
        ));
    }

    #[test]
    fn test_swap_transfer_maps_quote_and_recipient() {
        let wallet = wallet(&[Chain::Ethereum, Chain::Solana]);
        let request = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), true, Some(50)).unwrap();
        let quote = Quote {
            from_value: "99".to_string(),
            min_from_value: Some("90".to_string()),
            to_value: "1".to_string(),
            data: swapper::ProviderData {
                provider: swapper::ProviderType::new(swapper::SwapperProvider::Jupiter),
                slippage_bps: 50,
                routes: vec![],
            },
            request,
            eta_in_seconds: Some(30),
        };
        let data = SwapQuoteData {
            to: "0xrouter".to_string(),
            data_type: primitives::swap::SwapQuoteDataType::Contract,
            value: "100".to_string(),
            data: "0x".to_string(),
            memo: None,
            approval: None,
            gas_limit: None,
        };

        let transfer = swap_transfer(&wallet, &quote, data.clone()).unwrap();

        assert_eq!(transfer.recipient, "solana-address");
        assert_eq!(transfer.value, "100");
        assert!(transfer.use_max_amount);
        assert_eq!(transfer.data, data);
        assert_eq!(transfer.quote.from_address, "ethereum-address");
        assert_eq!(transfer.quote.to_address, "solana-address");
        assert_eq!(transfer.quote.from_value, "99");
        assert_eq!(transfer.quote.min_from_value.as_deref(), Some("90"));
        assert_eq!(transfer.quote.provider_data.provider, swapper::SwapperProvider::Jupiter);
        assert_eq!(transfer.quote.slippage_bps, 50);
        assert_eq!(transfer.quote.use_max_amount, Some(true));

        let ethereum_only = super::tests::wallet(&[Chain::Ethereum]);
        assert!(matches!(swap_transfer(&ethereum_only, &quote, data), Err(SwapperError::NotSupportedChain)));
    }

    #[test]
    fn test_permit_single_deadlines() {
        let approval = Permit2ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xspender".to_string(),
            value: "1".to_string(),
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
    fn test_sort_quotes_prefers_highest_output() {
        let wallet = wallet(&[Chain::Ethereum, Chain::Solana]);
        let quote = |to_value: &str| Quote {
            from_value: "100".to_string(),
            min_from_value: None,
            to_value: to_value.to_string(),
            data: swapper::ProviderData {
                provider: swapper::ProviderType::new(swapper::SwapperProvider::Jupiter),
                slippage_bps: 50,
                routes: vec![],
            },
            request: quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), false, None).unwrap(),
            eta_in_seconds: None,
        };

        let sorted = sort_quotes(vec![quote("5"), quote("50"), quote("abc"), quote("7")]);

        assert_eq!(sorted.iter().map(|quote| quote.to_value.as_str()).collect::<Vec<_>>(), vec!["50", "7", "5", "abc"]);
    }

    fn pair(from: Chain, to: Chain) -> GemSwapPair {
        GemSwapPair {
            from_asset_id: AssetId::from_chain(from),
            to_asset_id: AssetId::from_chain(to),
        }
    }

    #[test]
    fn test_most_swapped_receive_asset_prefers_the_pay_asset_history() {
        let pairs = [
            pair(Chain::Ethereum, Chain::Solana),
            pair(Chain::Ethereum, Chain::Solana),
            pair(Chain::Ethereum, Chain::Bitcoin),
            pair(Chain::Bitcoin, Chain::Ethereum),
            pair(Chain::Bitcoin, Chain::Ethereum),
            pair(Chain::Bitcoin, Chain::Ethereum),
        ];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_falls_back_to_the_overall_history() {
        let pairs = [
            pair(Chain::Bitcoin, Chain::Solana),
            pair(Chain::Bitcoin, Chain::Solana),
            pair(Chain::Bitcoin, Chain::Ethereum),
        ];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Solana))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_keeps_the_first_seen_on_a_tie() {
        let pairs = [pair(Chain::Ethereum, Chain::Bitcoin), pair(Chain::Ethereum, Chain::Solana)];

        assert_eq!(
            most_swapped_receive_asset(&pairs, &AssetId::from_chain(Chain::Ethereum)),
            Some(AssetId::from_chain(Chain::Bitcoin))
        );
    }

    #[test]
    fn test_most_swapped_receive_asset_never_suggests_the_pay_asset() {
        let pairs = [
            pair(Chain::Bitcoin, Chain::Ethereum),
            pair(Chain::Bitcoin, Chain::Ethereum),
            pair(Chain::Bitcoin, Chain::Solana),
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

    fn button_input(value: u64, available_balance: u64) -> GemSwapButtonInput {
        GemSwapButtonInput {
            value: BigInt::from(value),
            available_balance: BigInt::from(available_balance),
            quote_error: None,
            transfer_error: None,
        }
    }

    fn lift_button_input(value: &str, available_balance: &str) -> uniffi::Result<GemSwapButtonInput> {
        let mut buffer = Vec::new();
        for field in [value, available_balance] {
            <String as uniffi::Lower<crate::UniFfiTag>>::write(field.to_string(), &mut buffer);
        }
        for _ in 0..2 {
            <Option<SwapperError> as uniffi::Lower<crate::UniFfiTag>>::write(None, &mut buffer);
        }
        <GemSwapButtonInput as uniffi::Lift<crate::UniFfiTag>>::try_read(&mut buffer.as_slice())
    }

    #[test]
    fn test_button_action_offers_the_minimum_amount_only_when_the_balance_covers_it() {
        let too_small = |min_amount: Option<&str>| GemSwapButtonInput {
            quote_error: Some(SwapperError::InputAmountError {
                min_amount: min_amount.map(str::to_string),
            }),
            ..button_input(100, 18_900_023)
        };

        assert_eq!(
            button_action(&too_small(Some("18900023"))),
            GemSwapButtonAction::UseMinimumAmount { value: BigInt::from(18_900_023) }
        );
        assert_eq!(button_action(&too_small(Some("22000000"))), GemSwapButtonAction::InsufficientBalance);
        assert_eq!(button_action(&too_small(Some("0"))), GemSwapButtonAction::Swap);
        assert_eq!(button_action(&too_small(None)), GemSwapButtonAction::Swap);
    }

    #[test]
    fn test_button_action_blocks_an_unaffordable_amount_before_any_quote() {
        assert_eq!(button_action(&button_input(101, 100)), GemSwapButtonAction::InsufficientBalance);
        assert_eq!(button_action(&button_input(100, 100)), GemSwapButtonAction::Swap);
        assert_eq!(button_action(&button_input(0, 100)), GemSwapButtonAction::Swap);
    }

    #[test]
    fn test_a_malformed_swap_value_is_reported_instead_of_reading_as_zero() {
        assert_eq!(lift_button_input("101", "100").unwrap().value, BigInt::from(101));
        assert!(lift_button_input("", "100").is_err());
        assert!(lift_button_input("101", "not-a-number").is_err());
    }

    #[test]
    fn test_button_action_retries_only_a_retryable_error() {
        let quote_failed = |error: SwapperError| GemSwapButtonInput {
            quote_error: Some(error),
            ..button_input(100, 100)
        };
        let transfer_failed = |error: SwapperError| GemSwapButtonInput {
            transfer_error: Some(error),
            ..button_input(100, 100)
        };

        assert_eq!(button_action(&quote_failed(SwapperError::NoQuoteAvailable)), GemSwapButtonAction::RetryQuote);
        assert_eq!(button_action(&quote_failed(SwapperError::NotSupportedAsset)), GemSwapButtonAction::Swap);
        assert_eq!(button_action(&quote_failed(SwapperError::NoAvailableProvider)), GemSwapButtonAction::Swap);
        assert_eq!(button_action(&quote_failed(SwapperError::InvalidRoute)), GemSwapButtonAction::Swap);
        assert_eq!(
            button_action(&transfer_failed(SwapperError::TransactionError("nonce".to_string()))),
            GemSwapButtonAction::RetryTransfer
        );
        assert_eq!(button_action(&transfer_failed(SwapperError::NotSupportedChain)), GemSwapButtonAction::Swap);
    }

    #[test]
    fn test_button_action_prefers_the_minimum_amount_then_the_transfer_retry() {
        let input = GemSwapButtonInput {
            quote_error: Some(SwapperError::NoQuoteAvailable),
            transfer_error: Some(SwapperError::NoQuoteAvailable),
            ..button_input(101, 100)
        };
        assert_eq!(button_action(&input), GemSwapButtonAction::RetryTransfer);

        let with_minimum = GemSwapButtonInput {
            quote_error: Some(SwapperError::InputAmountError {
                min_amount: Some("50".to_string()),
            }),
            ..input
        };
        assert_eq!(button_action(&with_minimum), GemSwapButtonAction::UseMinimumAmount { value: BigInt::from(50) });
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
}
