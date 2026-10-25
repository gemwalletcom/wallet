use super::constants::{FEE_PENALTY_MULTIPLIER, FLOAT_SCALING};
use crate::SwapperError;
use gem_sui::coin_type_matches;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;
use serde::{Deserialize, Serialize};

const MAX_UNFILLED_BPS: u32 = 100;
const BID_ROUNDING: u64 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct Pool {
    pub pool_id: String,
    pub initial_shared_version: u64,
    pub base_coin_type: String,
    pub quote_coin_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum OrderSide {
    Ask,
    Bid,
}

impl Pool {
    pub fn side(&self, input_coin_type: &str) -> Option<OrderSide> {
        if coin_type_matches(input_coin_type, &self.base_coin_type) {
            Some(OrderSide::Ask)
        } else if coin_type_matches(input_coin_type, &self.quote_coin_type) {
            Some(OrderSide::Bid)
        } else {
            None
        }
    }

    pub fn coin_types(&self, side: OrderSide) -> (&str, &str) {
        match side {
            OrderSide::Ask => (&self.base_coin_type, &self.quote_coin_type),
            OrderSide::Bid => (&self.quote_coin_type, &self.base_coin_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct Route {
    pub pools: Vec<Pool>,
    pub amount_out: u64,
}

pub(super) fn route_legs<'a>(pools: &'a [Pool], from: &str, to: &str) -> Option<Vec<(&'a Pool, OrderSide)>> {
    let legs = pools
        .iter()
        .scan(from.to_string(), |coin_type, pool| {
            let side = pool.side(coin_type)?;
            *coin_type = pool.coin_types(side).1.to_string();
            Some((pool, side))
        })
        .collect::<Vec<_>>();
    let (pool, side) = legs.last()?;
    (legs.len() == pools.len() && coin_type_matches(pool.coin_types(*side).1, to)).then_some(legs)
}

pub(super) fn route_out(quotes: &[(OrderSide, PoolQuote)], amount_in: u64) -> Option<u64> {
    let (amount_out, unfilled_bps) = quotes.iter().try_fold((amount_in, 0u128), |(amount, unfilled_bps), (side, quote)| {
        let (amount_out, amount_left) = match side {
            OrderSide::Ask => (quote.quote, quote.base),
            OrderSide::Bid => (quote.base, quote.quote),
        };
        (amount > 0).then(|| (amount_out, unfilled_bps + (u128::from(amount_left) * u128::from(HUNDRED_PERCENT_IN_BPS)).div_ceil(u128::from(amount))))
    })?;
    (unfilled_bps <= u128::from(MAX_UNFILLED_BPS)).then_some(amount_out)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PoolQuote {
    pub base: u64,
    pub quote: u64,
    pub minimum_in: u64,
}

impl PoolQuote {
    pub fn from_return_values(side: OrderSide, values: &[Vec<u64>]) -> Result<Self, SwapperError> {
        let [quantities, book_params, minimum] = values else {
            return Err(SwapperError::ComputeQuoteError("DeepBook quote returned no quantities".into()));
        };
        let ([base, quote, _], [_, _, min_size], [first, second, _]) = (quantities.as_slice(), book_params.as_slice(), minimum.as_slice()) else {
            return Err(SwapperError::ComputeQuoteError("DeepBook quote returned no quantities".into()));
        };
        let minimum_in = match side {
            OrderSide::Ask => {
                let input_fee_rate = u128::from(*first) * FEE_PENALTY_MULTIPLIER / FLOAT_SCALING;
                u64::try_from((u128::from(*min_size) * (FLOAT_SCALING + input_fee_rate)).div_ceil(FLOAT_SCALING)).unwrap_or(u64::MAX)
            }
            OrderSide::Bid => match *second {
                0 => 0,
                quote_in => quote_in.saturating_add(BID_ROUNDING),
            },
        };
        Ok(Self { base: *base, quote: *quote, minimum_in })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_quote() {
        let book_params = vec![1_000, 100_000_000, 1_000_000_000];
        let ask = PoolQuote::from_return_values(OrderSide::Ask, &[vec![75_050_000, 466_618_278, 0], book_params.clone(), vec![1_000_000, 500_000, 0]]).unwrap();
        let bid = PoolQuote::from_return_values(OrderSide::Bid, &[vec![1_070_400_000_000, 28_582, 0], book_params.clone(), vec![1_000_000_000, 934_812, 0]]).unwrap();
        let illiquid = PoolQuote::from_return_values(OrderSide::Bid, &[vec![0, 1_000_000, 0], book_params, vec![0, 0, 0]]).unwrap();

        assert_eq!(
            ask,
            PoolQuote {
                base: 75_050_000,
                quote: 466_618_278,
                minimum_in: 1_001_250_000
            }
        );
        assert_eq!(
            bid,
            PoolQuote {
                base: 1_070_400_000_000,
                quote: 28_582,
                minimum_in: 934_815
            }
        );
        assert_eq!(illiquid, PoolQuote { base: 0, quote: 1_000_000, minimum_in: 0 });
        assert_eq!(
            PoolQuote::from_return_values(OrderSide::Ask, &[vec![75_050_000, 466_618_278, 0]]),
            Err(SwapperError::ComputeQuoteError("DeepBook quote returned no quantities".into()))
        );

        assert_eq!(route_out(&[(OrderSide::Ask, ask)], 500_000_000_000), Some(466_618_278));
        assert_eq!(route_out(&[(OrderSide::Ask, PoolQuote { base: 5_000_000, ..ask })], 500_000_000), Some(466_618_278));
        assert_eq!(route_out(&[(OrderSide::Ask, PoolQuote { base: 5_000_001, ..ask })], 500_000_000), None);
        assert_eq!(route_out(&[(OrderSide::Bid, bid)], 1_000_000_000), Some(1_070_400_000_000));
        assert_eq!(route_out(&[(OrderSide::Bid, PoolQuote { quote: 1_000_000, ..bid })], 1_000_000), None);

        let first = PoolQuote { base: 2_000_000, ..ask };
        assert_eq!(route_out(&[(OrderSide::Ask, first), (OrderSide::Bid, PoolQuote { base: 5_000, quote: 2_000_000, ..bid })], 500_000_000), Some(5_000));
        assert_eq!(route_out(&[(OrderSide::Ask, first), (OrderSide::Bid, PoolQuote { base: 5_000, quote: 3_000_000, ..bid })], 500_000_000), None);
    }
}
