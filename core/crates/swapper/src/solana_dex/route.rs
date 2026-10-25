use gem_solana::Pubkey;
use serde::{Deserialize, Serialize};

use super::token::SwapToken;
use crate::{
    SwapperError,
    fees::{FeeSide, ReferralTransfer, SOLANA_FEE_MINTS},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct DexHop {
    pub pool: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DexRoute {
    pub hops: Vec<DexHop>,
    pub swap_value: u64,
    pub output_value: u64,
    pub fee: ReferralTransfer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intermediate_value: Option<u64>,
}

impl DexRoute {
    pub fn intermediate(&self, input_mint: &Pubkey, output_mint: &Pubkey, tokens: &'static [SwapToken]) -> Result<Option<&'static SwapToken>, SwapperError> {
        match self.hops.as_slice() {
            [hop] if hop.input_mint == *input_mint && hop.output_mint == *output_mint => Ok(None),
            [first, second] if first.input_mint == *input_mint && second.output_mint == *output_mint && first.output_mint == second.input_mint => {
                tokens.iter().find(|token| token.mint == first.output_mint).map(Some).ok_or(SwapperError::InvalidRoute)
            }
            _ => Err(SwapperError::InvalidRoute),
        }
    }

    pub fn input_value(&self) -> u64 {
        self.fee.input_value(self.swap_value)
    }

    pub fn received_value(&self) -> u64 {
        self.fee.received_value(self.output_value)
    }

    pub fn minimum_output(&self, slippage_bps: u32) -> u64 {
        self.fee.minimum_output(self.output_value, slippage_bps)
    }
}

pub(crate) fn preferred_fee_side(input_mint: &Pubkey, output_mint: &Pubkey) -> Option<FeeSide> {
    FeeSide::by_priority(&SOLANA_FEE_MINTS, |mint| input_mint.to_base58() == *mint, |mint| output_mint.to_base58() == *mint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::token::{INTERMEDIATE_TOKENS, SOL_INTERMEDIATE};
    use gem_solana::{USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS};

    const BONK: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";

    fn mint(address: &str) -> Pubkey {
        Pubkey::from_base58(address).unwrap()
    }

    #[test]
    fn test_preferred_fee_side() {
        assert_eq!(preferred_fee_side(&mint(WSOL_TOKEN_ADDRESS), &mint(USDC_TOKEN_MINT)), Some(FeeSide::Input));
        assert_eq!(preferred_fee_side(&mint(USDC_TOKEN_MINT), &mint(WSOL_TOKEN_ADDRESS)), Some(FeeSide::Output));
        assert_eq!(preferred_fee_side(&mint(BONK), &mint(USDT_TOKEN_MINT)), Some(FeeSide::Output));
        assert_eq!(preferred_fee_side(&mint(USDC_TOKEN_MINT), &mint(BONK)), Some(FeeSide::Input));
        assert_eq!(preferred_fee_side(&mint(BONK), &mint(USDS_TOKEN_MINT)), Some(FeeSide::Output));
        assert_eq!(preferred_fee_side(&mint(BONK), &mint("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm")), None);
    }

    #[test]
    fn test_intermediate() {
        let (sol, usdc, bonk) = (mint(WSOL_TOKEN_ADDRESS), mint(USDC_TOKEN_MINT), mint(BONK));
        let hop = |input_mint, output_mint| DexHop {
            pool: Pubkey::new([1; 32]),
            input_mint,
            output_mint,
        };
        let route = |hops| DexRoute {
            hops,
            swap_value: 1,
            output_value: 1,
            fee: ReferralTransfer { side: FeeSide::Input, value: 0 },
            intermediate_value: None,
        };

        let tokens = &INTERMEDIATE_TOKENS;
        assert_eq!(route(vec![hop(bonk, sol)]).intermediate(&bonk, &sol, tokens), Ok(None));
        assert_eq!(route(vec![hop(bonk, usdc), hop(usdc, sol)]).intermediate(&bonk, &sol, tokens), Ok(Some(&INTERMEDIATE_TOKENS[1])));
        assert_eq!(
            route(vec![hop(bonk, usdc), hop(usdc, sol)]).intermediate(&bonk, &sol, &SOL_INTERMEDIATE),
            Err(SwapperError::InvalidRoute),
            "a protocol that routes only through SOL rejects USDC"
        );
        assert_eq!(route(vec![hop(bonk, sol)]).intermediate(&sol, &bonk, tokens), Err(SwapperError::InvalidRoute));
        assert_eq!(route(vec![hop(bonk, usdc), hop(sol, usdc)]).intermediate(&bonk, &usdc, tokens), Err(SwapperError::InvalidRoute));
        assert_eq!(route(vec![hop(sol, bonk), hop(bonk, usdc)]).intermediate(&sol, &usdc, tokens), Err(SwapperError::InvalidRoute));
        assert_eq!(route(vec![]).intermediate(&sol, &usdc, tokens), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_minimum_output() {
        let route = DexRoute {
            hops: vec![DexHop {
                pool: Pubkey::new([1; 32]),
                input_mint: mint(WSOL_TOKEN_ADDRESS),
                output_mint: mint(USDC_TOKEN_MINT),
            }],
            swap_value: 995_000,
            output_value: 1_000_000,
            fee: ReferralTransfer { side: FeeSide::Input, value: 5_000 },
            intermediate_value: None,
        };
        let output_fee = DexRoute {
            fee: ReferralTransfer { side: FeeSide::Output, value: 5_000 },
            ..route.clone()
        };

        assert_eq!(route.minimum_output(100), 990_000);
        assert_eq!(output_fee.minimum_output(100), 990_050);
    }

    #[test]
    fn test_intermediate_value_round_trips_and_is_left_out_when_absent() {
        let route = DexRoute {
            hops: vec![],
            swap_value: 1,
            output_value: 1,
            fee: ReferralTransfer { side: FeeSide::Input, value: 0 },
            intermediate_value: None,
        };
        let chained = DexRoute {
            intermediate_value: Some(7),
            ..route.clone()
        };
        let json = |route: &DexRoute| serde_json::to_value(route).unwrap();

        assert_eq!(json(&route).get("intermediate_value"), None);
        assert_eq!(json(&chained)["intermediate_value"], 7);
        assert_eq!(serde_json::from_value::<DexRoute>(json(&route)).unwrap(), route);
        assert_eq!(serde_json::from_value::<DexRoute>(json(&chained)).unwrap(), chained);
    }
}
