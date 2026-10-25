use gem_sui::coin_type_matches;
use serde::{Deserialize, Serialize};

use crate::fees::ReferralTransfer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct Hop {
    pub pool_id: String,
    pub pool_init_version: u64,
    pub coin_a: String,
    pub coin_b: String,
    pub a2b: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub after_sqrt_price: u128,
}

impl Hop {
    pub fn input_coin_type(&self) -> &str {
        if self.a2b { &self.coin_a } else { &self.coin_b }
    }

    pub fn output_coin_type(&self) -> &str {
        if self.a2b { &self.coin_b } else { &self.coin_a }
    }

    pub fn order_by_direction<T>(&self, side_a: T, side_b: T) -> (T, T) {
        if self.a2b { (side_a, side_b) } else { (side_b, side_a) }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct PoolRoute {
    pub hops: Vec<Hop>,
    pub fee: ReferralTransfer,
}

impl PoolRoute {
    pub fn input_coin_type(&self) -> &str {
        self.hops.first().expect("PoolRoute always has at least one hop").input_coin_type()
    }

    pub fn output_coin_type(&self) -> &str {
        self.hops.last().expect("PoolRoute always has at least one hop").output_coin_type()
    }

    pub fn gross_amount_out(&self) -> u64 {
        self.hops.last().map(|h| h.amount_out).unwrap_or_default()
    }

    pub fn net_amount_out(&self) -> u64 {
        self.fee.received_value(self.gross_amount_out())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct DiscoveredPool {
    pub pool_id: String,
    pub pool_init_version: u64,
    pub coin_a: String,
    pub coin_b: String,
}

impl DiscoveredPool {
    pub fn into_hop(self, input_coin_type: &str, amount_in: u64) -> Hop {
        let a2b = coin_type_matches(input_coin_type, &self.coin_a);
        Hop {
            pool_id: self.pool_id,
            pool_init_version: self.pool_init_version,
            coin_a: self.coin_a,
            coin_b: self.coin_b,
            a2b,
            amount_in,
            amount_out: 0,
            after_sqrt_price: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::FeeSide;
    use primitives::asset_constants::SUI_USDC_TOKEN_ID;

    const SUI_FULL: &str = gem_sui::SUI_COIN_TYPE_FULL;
    const USDC: &str = SUI_USDC_TOKEN_ID;

    #[test]
    fn test_pool_route_traversal() {
        let hop = Hop {
            coin_a: SUI_FULL.into(),
            coin_b: USDC.into(),
            amount_out: 800_000,
            ..Hop::mock()
        };
        let route = PoolRoute {
            hops: vec![hop.clone()],
            fee: ReferralTransfer { side: FeeSide::Input, value: 5_000 },
        };
        assert_eq!(route.input_coin_type(), SUI_FULL);
        assert_eq!(route.output_coin_type(), USDC);
        assert_eq!(route.gross_amount_out(), 800_000);
        assert_eq!(route.net_amount_out(), 800_000);

        let route_output_fee = PoolRoute {
            hops: vec![hop],
            fee: ReferralTransfer { side: FeeSide::Output, value: 5_000 },
        };
        assert_eq!(route_output_fee.net_amount_out(), 795_000);
    }
}
