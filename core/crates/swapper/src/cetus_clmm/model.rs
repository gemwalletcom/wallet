use gem_sui::{SUI_COIN_TYPE_FULL, coin_type_matches};
use primitives::asset_constants::SUI_USDC_TOKEN_ID;
use serde::{Deserialize, Serialize};

pub(super) const INTERMEDIATE_COIN_TYPES: &[&str] = &[SUI_COIN_TYPE_FULL, SUI_USDC_TOKEN_ID];

const FEE_PRIORITY_COINS: &[&str] = &[SUI_COIN_TYPE_FULL, SUI_USDC_TOKEN_ID];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum FeeSide {
    Input,
    Output,
}

impl FeeSide {
    pub fn select(input_coin_type: &str, output_coin_type: &str) -> Self {
        for preferred in FEE_PRIORITY_COINS {
            if coin_type_matches(input_coin_type, preferred) {
                return Self::Input;
            }
            if coin_type_matches(output_coin_type, preferred) {
                return Self::Output;
            }
        }
        Self::Output
    }
}

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
    pub fee_amount: u64,
    pub fee_side: FeeSide,
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
        match self.fee_side {
            FeeSide::Input => self.gross_amount_out(),
            FeeSide::Output => self.gross_amount_out().saturating_sub(self.fee_amount),
        }
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

    const SUI_FULL: &str = gem_sui::SUI_COIN_TYPE_FULL;
    const USDC: &str = SUI_USDC_TOKEN_ID;
    const BLUE: &str = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

    fn hop(coin_a: &str, coin_b: &str, a2b: bool, amount_out: u64) -> Hop {
        Hop {
            pool_id: "0xpool".into(),
            pool_init_version: 1,
            coin_a: coin_a.into(),
            coin_b: coin_b.into(),
            a2b,
            amount_in: 1_000,
            amount_out,
            after_sqrt_price: 0,
        }
    }

    #[test]
    fn test_fee_side_preference() {
        assert_eq!(FeeSide::select(SUI_FULL, USDC), FeeSide::Input);
        assert_eq!(FeeSide::select(USDC, SUI_FULL), FeeSide::Output);
        assert_eq!(FeeSide::select(SUI_FULL, BLUE), FeeSide::Input);
        assert_eq!(FeeSide::select(BLUE, SUI_FULL), FeeSide::Output);
        assert_eq!(FeeSide::select(USDC, BLUE), FeeSide::Input);
        assert_eq!(FeeSide::select(BLUE, USDC), FeeSide::Output);
        assert_eq!(FeeSide::select(BLUE, "0xfoo::bar::BAR"), FeeSide::Output);
    }

    #[test]
    fn test_pool_route_traversal() {
        let route = PoolRoute {
            hops: vec![hop(SUI_FULL, USDC, true, 800_000)],
            fee_amount: 5_000,
            fee_side: FeeSide::Input,
        };
        assert_eq!(route.input_coin_type(), SUI_FULL);
        assert_eq!(route.output_coin_type(), USDC);
        assert_eq!(route.gross_amount_out(), 800_000);
        assert_eq!(route.net_amount_out(), 800_000);

        let route_output_fee = PoolRoute {
            hops: vec![hop(SUI_FULL, USDC, true, 800_000)],
            fee_amount: 5_000,
            fee_side: FeeSide::Output,
        };
        assert_eq!(route_output_fee.net_amount_out(), 795_000);
    }
}
