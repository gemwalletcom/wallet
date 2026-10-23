use num_bigint::BigUint;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;

use crate::config::QuotePreference;
use crate::fees::{max_amount_spends_all_but_fee, subtract_bps};
use crate::{Quote, QuoteRequest, SwapAmountMode};

const MAX_SHORTFALL_BPS: u32 = 5_000;

pub fn rank_quotes(request: &QuoteRequest, mut quotes: Vec<Quote>, preferences: &[QuotePreference], leaves_no_dust: impl Fn(&Quote) -> bool) -> Vec<Quote> {
    quotes.sort_by(|a, b| b.to_value.cmp(&a.to_value));
    let Some(best) = quotes.first().map(|quote| quote.to_value.clone()) else {
        return quotes;
    };
    let floor = &best * (HUNDRED_PERCENT_IN_BPS - MAX_SHORTFALL_BPS);
    quotes.retain(|quote| &quote.to_value * HUNDRED_PERCENT_IN_BPS >= floor);
    for preference in preferences {
        match preference {
            QuotePreference::Providers { providers, tolerance_bps } => prefer(&mut quotes, &best, *tolerance_bps, |quote| providers.contains(&quote.data.provider.id)),
            QuotePreference::DustFreeMaxAmount { tolerance_bps } => {
                if request.options.use_max_amount && request.from_asset.is_native() {
                    prefer(&mut quotes, &best, *tolerance_bps, &leaves_no_dust);
                }
            }
        }
    }
    quotes
}

pub(crate) fn leaves_no_dust(quote: &Quote, amount_mode: Option<SwapAmountMode>) -> bool {
    match amount_mode {
        Some(SwapAmountMode::Flexible) => true,
        Some(SwapAmountMode::Fixed) | None => max_amount_spends_all_but_fee(quote.request.from_asset.chain()),
    }
}

fn prefer(quotes: &mut Vec<Quote>, best: &BigUint, tolerance_bps: u32, is_preferred: impl Fn(&Quote) -> bool) {
    let floor = subtract_bps(best, tolerance_bps);
    if let Some(index) = quotes.iter().position(|quote| is_preferred(quote) && quote.to_value >= floor) {
        let preferred = quotes.remove(index);
        quotes.insert(0, preferred);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperProvider, SwapperQuoteAsset, config::quote_preferences, testkit::mock_quote};
    use primitives::{AssetId, Chain, asset_constants::ETHEREUM_USDC_ASSET_ID};

    fn request(from: AssetId, use_max_amount: bool) -> QuoteRequest {
        let request = mock_quote(SwapperQuoteAsset::from(from), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));
        QuoteRequest {
            options: Options { use_max_amount, ..request.options },
            ..request
        }
    }

    fn quote(request: &QuoteRequest, provider: SwapperProvider, to_value: &str) -> Quote {
        Quote {
            request: request.clone(),
            ..Quote::mock_with_provider(provider, to_value)
        }
    }

    fn deposit_only_near_intents(quote: &Quote) -> bool {
        let amount_mode = match quote.data.provider.id {
            SwapperProvider::NearIntents => SwapAmountMode::Flexible,
            _ => SwapAmountMode::Fixed,
        };
        leaves_no_dust(quote, Some(amount_mode))
    }

    fn providers(quotes: Vec<Quote>) -> Vec<SwapperProvider> {
        quotes.into_iter().map(|quote| quote.data.provider.id).collect()
    }

    #[test]
    fn test_quotes_rank_by_output_and_equal_outputs_keep_the_order_the_providers_answered_in() {
        let request = request(AssetId::from_chain(Chain::Ethereum), false);
        let rank = |outs: [(SwapperProvider, &str); 3]| providers(rank_quotes(&request, outs.into_iter().map(|(provider, out)| quote(&request, provider, out)).collect(), &[], deposit_only_near_intents));

        assert_eq!(
            rank([(SwapperProvider::UniswapV3, "101"), (SwapperProvider::UniswapV4, "100"), (SwapperProvider::PancakeswapV3, "102")]),
            vec![SwapperProvider::PancakeswapV3, SwapperProvider::UniswapV3, SwapperProvider::UniswapV4]
        );
        assert_eq!(
            rank([(SwapperProvider::UniswapV3, "100"), (SwapperProvider::UniswapV4, "100"), (SwapperProvider::PancakeswapV3, "100")]),
            vec![SwapperProvider::UniswapV3, SwapperProvider::UniswapV4, SwapperProvider::PancakeswapV3]
        );
        assert_eq!(
            rank([
                (SwapperProvider::UniswapV3, "9999999999999999999"),
                (SwapperProvider::Jupiter, "10000000000000000000"),
                (SwapperProvider::Okx, "5000000000000000000")
            ]),
            vec![SwapperProvider::Jupiter, SwapperProvider::UniswapV3, SwapperProvider::Okx],
            "whole amounts are compared, not their text"
        );
    }

    #[test]
    fn test_quotes_giving_less_than_half_of_the_best_are_dropped() {
        let request = request(AssetId::from_chain(Chain::Solana), false);
        let rank = |outs: [(SwapperProvider, &str); 3]| {
            providers(rank_quotes(
                &request,
                outs.into_iter().map(|(provider, out)| quote(&request, provider, out)).collect(),
                &quote_preferences(),
                deposit_only_near_intents,
            ))
        };

        assert_eq!(
            rank([(SwapperProvider::Jupiter, "10000"), (SwapperProvider::MeteoraDammV2, "16"), (SwapperProvider::RaydiumClmm, "4999")]),
            vec![SwapperProvider::Jupiter]
        );
        assert_eq!(
            rank([(SwapperProvider::Jupiter, "10000"), (SwapperProvider::Orca, "5000"), (SwapperProvider::Okx, "9995")]),
            vec![SwapperProvider::Jupiter, SwapperProvider::Okx, SwapperProvider::Orca],
            "exactly half of the best stays"
        );
        assert_eq!(
            rank([(SwapperProvider::PumpSwap, "1"), (SwapperProvider::Orca, "0"), (SwapperProvider::Okx, "1")]),
            vec![SwapperProvider::PumpSwap, SwapperProvider::Okx]
        );
    }

    #[test]
    fn test_a_max_swap_of_a_coin_prefers_a_provider_that_leaves_nothing_behind_when_its_quote_is_close_enough() {
        let solana = request(AssetId::from_chain(Chain::Solana), true);
        let preferences = [QuotePreference::DustFreeMaxAmount { tolerance_bps: 25 }];
        let rank = |deposit_out: &str| {
            providers(rank_quotes(
                &solana,
                vec![
                    quote(&solana, SwapperProvider::Jupiter, "10000"),
                    quote(&solana, SwapperProvider::NearIntents, deposit_out),
                    quote(&solana, SwapperProvider::Okx, "9990"),
                ],
                &preferences,
                deposit_only_near_intents,
            ))
        };

        assert_eq!(
            rank("9975"),
            vec![SwapperProvider::NearIntents, SwapperProvider::Jupiter, SwapperProvider::Okx],
            "within 25 bps of the best, the deposit provider wins because the reserve stays behind otherwise"
        );
        assert_eq!(
            rank("9974"),
            vec![SwapperProvider::Jupiter, SwapperProvider::Okx, SwapperProvider::NearIntents],
            "further away, the best quote wins and the rest keep their order"
        );
        assert_eq!(
            rank("10001"),
            vec![SwapperProvider::NearIntents, SwapperProvider::Jupiter, SwapperProvider::Okx],
            "a deposit provider that is also the best simply stays first"
        );
    }

    #[test]
    fn test_the_preference_applies_only_where_the_whole_balance_of_a_coin_is_swapped_and_a_reserve_would_stay() {
        let preferences = [QuotePreference::DustFreeMaxAmount { tolerance_bps: 25 }];
        let close = |request: &QuoteRequest, preferences: &[QuotePreference]| {
            providers(rank_quotes(
                request,
                vec![quote(request, SwapperProvider::UniswapV3, "10000"), quote(request, SwapperProvider::NearIntents, "9990")],
                preferences,
                deposit_only_near_intents,
            ))
        };
        let solana_max = request(AssetId::from_chain(Chain::Solana), true);

        assert_eq!(close(&solana_max, &preferences), vec![SwapperProvider::NearIntents, SwapperProvider::UniswapV3]);
        assert_eq!(
            close(&request(AssetId::from_chain(Chain::Solana), false), &preferences),
            vec![SwapperProvider::UniswapV3, SwapperProvider::NearIntents],
            "not a max swap"
        );
        assert_eq!(
            close(&request(ETHEREUM_USDC_ASSET_ID.clone(), true), &preferences),
            vec![SwapperProvider::UniswapV3, SwapperProvider::NearIntents],
            "a token pays its fee in the coin, nothing stays behind"
        );
        assert_eq!(
            close(&request(AssetId::from_chain(Chain::Ethereum), true), &preferences),
            vec![SwapperProvider::UniswapV3, SwapperProvider::NearIntents],
            "on an Ethereum-style network a contract call spends everything but the fee too, so the best quote wins"
        );
        assert_eq!(close(&solana_max, &[]), vec![SwapperProvider::UniswapV3, SwapperProvider::NearIntents], "with no preference configured the best quote wins");
        assert_eq!(providers(rank_quotes(&solana_max, vec![], &preferences, deposit_only_near_intents)), vec![]);
    }

    #[test]
    fn test_sunswap_is_preferred_within_100_bps_of_the_best_quote() {
        let tron = request(AssetId::from_chain(Chain::Tron), false);
        let rank = |sunswap_out: &str| {
            providers(rank_quotes(
                &tron,
                vec![
                    quote(&tron, SwapperProvider::Thorchain, "10000"),
                    quote(&tron, SwapperProvider::Relay, "9995"),
                    quote(&tron, SwapperProvider::SunswapV3, sunswap_out),
                ],
                &quote_preferences(),
                deposit_only_near_intents,
            ))
        };

        assert_eq!(rank("9900"), vec![SwapperProvider::SunswapV3, SwapperProvider::Thorchain, SwapperProvider::Relay]);
        assert_eq!(rank("9899"), vec![SwapperProvider::Thorchain, SwapperProvider::Relay, SwapperProvider::SunswapV3], "further than 100 bps, the best quote wins");
    }

    #[test]
    fn test_across_is_preferred_within_10_bps_of_the_best_quote() {
        let ethereum = request(AssetId::from_chain(Chain::Ethereum), false);
        let rank = |across_out: &str| {
            providers(rank_quotes(
                &ethereum,
                vec![
                    quote(&ethereum, SwapperProvider::Relay, "10000"),
                    quote(&ethereum, SwapperProvider::Across, across_out),
                    quote(&ethereum, SwapperProvider::Mayan, "9995"),
                ],
                &quote_preferences(),
                deposit_only_near_intents,
            ))
        };

        assert_eq!(rank("9990"), vec![SwapperProvider::Across, SwapperProvider::Relay, SwapperProvider::Mayan]);
        assert_eq!(rank("9989"), vec![SwapperProvider::Relay, SwapperProvider::Mayan, SwapperProvider::Across], "further than 10 bps, the best quote wins");
        assert_eq!(rank("10001"), vec![SwapperProvider::Across, SwapperProvider::Relay, SwapperProvider::Mayan]);
    }

    #[test]
    fn test_solana_pools_are_preferred_within_5_bps_of_the_best_quote() {
        let solana = request(AssetId::from_chain(Chain::Solana), false);
        let rank = |provider: SwapperProvider, out: &str| {
            providers(rank_quotes(
                &solana,
                vec![quote(&solana, SwapperProvider::Jupiter, "10000"), quote(&solana, provider, out), quote(&solana, SwapperProvider::Okx, "9990")],
                &quote_preferences(),
                deposit_only_near_intents,
            ))
        };

        for provider in [SwapperProvider::Orca, SwapperProvider::RaydiumClmm, SwapperProvider::PumpSwap, SwapperProvider::MeteoraDammV2] {
            assert_eq!(rank(provider, "9995"), vec![provider, SwapperProvider::Jupiter, SwapperProvider::Okx]);
            assert_eq!(rank(provider, "9994"), vec![SwapperProvider::Jupiter, provider, SwapperProvider::Okx], "further than 5 bps, the best quote wins");
        }
    }

    #[test]
    fn test_a_max_swap_that_leaves_nothing_behind_outranks_the_across_preference() {
        let solana_max = request(AssetId::from_chain(Chain::Solana), true);
        let ranked = rank_quotes(
            &solana_max,
            vec![
                quote(&solana_max, SwapperProvider::Relay, "10000"),
                quote(&solana_max, SwapperProvider::Across, "9995"),
                quote(&solana_max, SwapperProvider::NearIntents, "9980"),
            ],
            &quote_preferences(),
            deposit_only_near_intents,
        );

        assert_eq!(providers(ranked), vec![SwapperProvider::NearIntents, SwapperProvider::Across, SwapperProvider::Relay]);
    }
}
