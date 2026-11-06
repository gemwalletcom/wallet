use gem_hypercore::models::spot::OrderbookLevel;
use number_formatter::Decimal;

use crate::SwapperError;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SimulationResult {
    pub amount_out: Decimal,
    pub limit_price: Decimal,
}

pub(super) fn simulate_sell(amount: &Decimal, bids: &[OrderbookLevel]) -> Result<SimulationResult, SwapperError> {
    let mut remaining = amount.clone();
    let mut quote_total = Decimal::zero();
    let mut min_price: Option<Decimal> = None;

    for level in bids {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;
        if level_size <= Decimal::zero() {
            continue;
        }

        let trade_size = remaining.clone().min(level_size);
        quote_total += &trade_size * &price;
        remaining -= &trade_size;
        min_price = Some(min_price.map_or_else(|| price.clone(), |p| p.min(price.clone())));

        if remaining <= Decimal::zero() {
            return Ok(SimulationResult {
                amount_out: quote_total,
                limit_price: min_price.unwrap_or(price),
            });
        }
    }

    Err(SwapperError::NoQuoteAvailable)
}

pub(super) fn simulate_buy(amount: &Decimal, asks: &[OrderbookLevel]) -> Result<SimulationResult, SwapperError> {
    let mut remaining_quote = amount.clone();
    let mut base_total = Decimal::zero();
    let mut max_price: Option<Decimal> = None;

    for level in asks {
        let level_size = parse_decimal(&level.sz)?;
        let price = parse_decimal(&level.px)?;
        if level_size <= Decimal::zero() || price <= Decimal::zero() {
            continue;
        }

        let level_quote = &level_size * &price;
        if remaining_quote > level_quote {
            base_total += &level_size;
            remaining_quote -= level_quote;
            max_price = Some(max_price.map_or_else(|| price.clone(), |p| p.max(price.clone())));
        } else {
            base_total += &remaining_quote / &price;
            max_price = Some(max_price.map_or_else(|| price.clone(), |p| p.max(price.clone())));
            remaining_quote = Decimal::zero();
            break;
        }
    }

    let Some(limit_price) = max_price else {
        return Err(SwapperError::NoQuoteAvailable);
    };
    if remaining_quote > Decimal::zero() || base_total <= Decimal::zero() {
        return Err(SwapperError::NoQuoteAvailable);
    }

    Ok(SimulationResult { amount_out: base_total, limit_price })
}

fn parse_decimal(value: &str) -> Result<Decimal, SwapperError> {
    value.parse().map_err(|_| SwapperError::compute_quote_error("failed to parse orderbook level"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_sell() {
        let bids = vec![OrderbookLevel::mock("2", "3"), OrderbookLevel::mock("1.5", "5")];
        assert_eq!(
            simulate_sell(&Decimal::from(7), &bids),
            Ok(SimulationResult {
                amount_out: Decimal::from(12),
                limit_price: Decimal::new(15, 1),
            })
        );
        assert_eq!(simulate_sell(&Decimal::from(10), &bids), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_simulate_buy() {
        let asks = vec![OrderbookLevel::mock("2", "3"), OrderbookLevel::mock("3", "5")];
        assert_eq!(
            simulate_buy(&Decimal::from(10), &asks),
            Ok(SimulationResult {
                amount_out: Decimal::from(3) + Decimal::from(4) / Decimal::from(3),
                limit_price: Decimal::from(3),
            })
        );
        assert_eq!(simulate_buy(&Decimal::from(25), &asks), Err(SwapperError::NoQuoteAvailable));
    }
}
