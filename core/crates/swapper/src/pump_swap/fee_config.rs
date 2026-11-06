use borsh::BorshDeserialize;
use gem_solana::{Pubkey, anchor::AnchorAccount, wrapped_sol::wrapped_sol_mint};
use primitives::decode_hex_literal;

pub(super) const PUMP_FEES_PROGRAM: Pubkey = Pubkey::new(decode_hex_literal("0c35ffa9055a8e568da8f7bc075615274cf1c92ca41f40009c516aa414c27c70"));
pub(super) const FEE_CONFIG: Pubkey = Pubkey::new(decode_hex_literal("41246ecc7d78fe81e41773a46965419937923a07644797df6f3eb514426010cb"));
const BPS: u128 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize)]
pub(super) struct Fees {
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub creator_fee_bps: u64,
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize)]
struct FeeTier {
    market_cap_lamports_threshold: u128,
    fees: Fees,
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize)]
pub(super) struct FeeConfig {
    _bump: u8,
    _admin: Pubkey,
    _flat_fees: Fees,
    fee_tiers: Vec<FeeTier>,
    stable_fee_tiers: Vec<FeeTier>,
}

impl AnchorAccount for FeeConfig {
    const NAME: &'static str = "FeeConfig";
}

impl FeeConfig {
    pub fn tier_fees(&self, quote_mint: &Pubkey, market_cap: u128) -> Option<Fees> {
        let tiers = if *quote_mint == wrapped_sol_mint() { &self.fee_tiers } else { &self.stable_fee_tiers };
        let first = tiers.first()?;
        if market_cap < first.market_cap_lamports_threshold {
            return Some(first.fees);
        }
        tiers.iter().rev().find(|tier| market_cap >= tier.market_cap_lamports_threshold).map(|tier| tier.fees)
    }
}

impl Fees {
    pub fn quote_after_fees(&self, quote_in: u128) -> Option<u128> {
        let total_bps = u128::from(self.lp_fee_bps) + u128::from(self.protocol_fee_bps) + u128::from(self.creator_fee_bps);
        let quote = quote_in * BPS / (BPS + total_bps);
        let charged = quote + self.lp_fee(quote) + self.protocol_and_creator_fees(quote);
        quote.checked_sub(charged.saturating_sub(quote_in))
    }

    pub fn lp_fee(&self, value: u128) -> u128 {
        fee(value, self.lp_fee_bps)
    }

    pub fn protocol_and_creator_fees(&self, value: u128) -> u128 {
        fee(value, self.protocol_fee_bps) + fee(value, self.creator_fee_bps)
    }
}

fn fee(value: u128, bps: u64) -> u128 {
    (value * u128::from(bps)).div_ceil(BPS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pump_swap::testkit::{TEST_SOL_POOL, mock_fee_config};
    use gem_solana::USDC_TOKEN_MINT;

    #[test]
    fn test_tier_fees() {
        let fee_config = mock_fee_config(TEST_SOL_POOL);
        let sol = wrapped_sol_mint();
        let usdc = Pubkey::from_base58(USDC_TOKEN_MINT).unwrap();
        let first = Fees {
            lp_fee_bps: 2,
            protocol_fee_bps: 93,
            creator_fee_bps: 30,
        };

        assert_eq!(fee_config.tier_fees(&sol, 419_999_999_999), Some(first));
        assert_eq!(
            fee_config.tier_fees(&sol, 420_000_000_000),
            Some(Fees {
                lp_fee_bps: 20,
                protocol_fee_bps: 5,
                creator_fee_bps: 95,
            })
        );
        assert_eq!(
            fee_config.tier_fees(&sol, u128::MAX),
            Some(Fees {
                lp_fee_bps: 20,
                protocol_fee_bps: 5,
                creator_fee_bps: 5,
            })
        );
        assert_eq!(fee_config.tier_fees(&usdc, 58_999_999_999), Some(first));
        assert_eq!(
            fee_config.tier_fees(&usdc, 420_000_000_000),
            Some(Fees {
                lp_fee_bps: 20,
                protocol_fee_bps: 5,
                creator_fee_bps: 90,
            })
        );
    }
}
