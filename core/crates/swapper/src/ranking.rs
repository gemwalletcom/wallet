use num_bigint::BigUint;

use crate::config::QuotePreference;
use crate::fees::max_amount_spends_all_but_fee;
use crate::{Quote, QuoteRequest, SwapAmountMode};

pub(crate) struct RankedQuote {
    pub quote: Quote,
    pub amount_mode: SwapAmountMode,
}

impl RankedQuote {
    fn leaves_no_dust(&self) -> bool {
        match self.amount_mode {
            SwapAmountMode::Flexible => true,
            SwapAmountMode::Fixed => max_amount_spends_all_but_fee(self.quote.request.from_asset.chain()),
        }
    }
}

pub(crate) fn rank_quotes(request: &QuoteRequest, mut quotes: Vec<RankedQuote>, preferences: &[QuotePreference]) -> Vec<Quote> {
    quotes.sort_by(|a, b| b.quote.to_value.cmp(&a.quote.to_value));
    for preference in preferences {
        match preference {
            QuotePreference::DustFreeMaxAmount { tolerance_bps } => prefer_dust_free_max_amount(request, &mut quotes, *tolerance_bps),
        }
    }
    quotes.into_iter().map(|ranked| ranked.quote).collect()
}

fn prefer_dust_free_max_amount(request: &QuoteRequest, quotes: &mut Vec<RankedQuote>, tolerance_bps: u32) {
    if !request.options.use_max_amount || !request.from_asset.is_native() {
        return;
    }
    let Some(best) = quotes.first().map(|ranked| ranked.quote.to_value.clone()) else {
        return;
    };
    let floor = best * BigUint::from(10_000 - tolerance_bps.min(10_000)) / BigUint::from(10_000u32);
    if let Some(index) = quotes.iter().position(|ranked| ranked.leaves_no_dust() && ranked.quote.to_value >= floor) {
        let preferred = quotes.remove(index);
        quotes.insert(0, preferred);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperProvider, SwapperQuoteAsset, testkit::mock_quote};
    use primitives::{AssetId, Chain, asset_constants::ETHEREUM_USDC_ASSET_ID};

    fn request(from: AssetId, use_max_amount: bool) -> QuoteRequest {
        let request = mock_quote(SwapperQuoteAsset::from(from), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));
        QuoteRequest {
            options: Options { use_max_amount, ..request.options },
            ..request
        }
    }

    fn ranked(request: &QuoteRequest, provider: SwapperProvider, amount_mode: SwapAmountMode, to_value: &str) -> RankedQuote {
        RankedQuote {
            quote: Quote {
                request: request.clone(),
                ..Quote::mock_with_provider(provider, to_value)
            },
            amount_mode,
        }
    }

    fn providers(quotes: Vec<Quote>) -> Vec<SwapperProvider> {
        quotes.into_iter().map(|quote| quote.data.provider.id).collect()
    }

    #[test]
    fn test_quotes_rank_by_output_and_equal_outputs_keep_the_order_the_providers_answered_in() {
        let request = request(AssetId::from_chain(Chain::Ethereum), false);
        let rank = |outs: [(SwapperProvider, &str); 3]| providers(rank_quotes(&request, outs.into_iter().map(|(provider, out)| ranked(&request, provider, SwapAmountMode::Fixed, out)).collect(), &[]));

        assert_eq!(
            rank([(SwapperProvider::UniswapV3, "101"), (SwapperProvider::UniswapV4, "100"), (SwapperProvider::PancakeswapV3, "102")]),
            vec![SwapperProvider::PancakeswapV3, SwapperProvider::UniswapV3, SwapperProvider::UniswapV4]
        );
        assert_eq!(
            rank([(SwapperProvider::UniswapV3, "100"), (SwapperProvider::UniswapV4, "100"), (SwapperProvider::PancakeswapV3, "100")]),
            vec![SwapperProvider::UniswapV3, SwapperProvider::UniswapV4, SwapperProvider::PancakeswapV3]
        );
        assert_eq!(
            rank([(SwapperProvider::UniswapV3, "9999999999999999999"), (SwapperProvider::Jupiter, "10000000000000000000"), (SwapperProvider::Okx, "1")]),
            vec![SwapperProvider::Jupiter, SwapperProvider::UniswapV3, SwapperProvider::Okx],
            "whole amounts are compared, not their text"
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
                    ranked(&solana, SwapperProvider::Jupiter, SwapAmountMode::Fixed, "10000"),
                    ranked(&solana, SwapperProvider::NearIntents, SwapAmountMode::Flexible, deposit_out),
                    ranked(&solana, SwapperProvider::Okx, SwapAmountMode::Fixed, "9990"),
                ],
                &preferences,
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
                vec![
                    ranked(request, SwapperProvider::UniswapV3, SwapAmountMode::Fixed, "10000"),
                    ranked(request, SwapperProvider::NearIntents, SwapAmountMode::Flexible, "9990"),
                ],
                preferences,
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
        assert_eq!(providers(rank_quotes(&solana_max, vec![], &preferences)), vec![]);
    }
}
