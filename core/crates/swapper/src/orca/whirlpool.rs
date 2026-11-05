use borsh::BorshDeserialize;
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::orca_whirlpool_program};
use primitives::decode_hex_literal;

use crate::{
    SwapperError,
    clmm::{ClmmSwap, MAX_SQRT_PRICE, MAX_TICK_INDEX, MIN_SQRT_PRICE, MIN_TICK_INDEX},
    solana_dex::DexPool,
};

const WHIRLPOOLS_CONFIG: Pubkey = Pubkey::new(decode_hex_literal("13e441f83913ca68b0634fb025fdeaa88737e84110d1255e357b3377ddee1ccd"));
pub(super) const FEE_TIER_INDEXES: [u16; 9] = [1, 4, 8, 16, 64, 1025, 1026, 1028, 1030];
pub(super) const TICK_ARRAY_SIZE: i32 = 88;

#[derive(Debug, Clone, PartialEq, BorshDeserialize)]
pub(crate) struct Whirlpool {
    _whirlpools_config_and_bump: [u8; 33],
    pub tick_spacing: u16,
    fee_tier_index: u16,
    pub fee_rate: u16,
    _protocol_fee_rate: u16,
    pub liquidity: u128,
    sqrt_price: u128,
    pub tick_current_index: i32,
    _protocol_fees_owed: [u8; 16],
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    _fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
}

impl AnchorAccount for Whirlpool {
    const NAME: &'static str = "Whirlpool";
}

impl DexPool for Whirlpool {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.token_mint_a, self.token_mint_b)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.token_vault_a, self.token_vault_b)
    }
}

impl Whirlpool {
    pub fn addresses(mint_0: &Pubkey, mint_1: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        let (mint_a, mint_b) = if mint_0 < mint_1 { (mint_0, mint_1) } else { (mint_1, mint_0) };
        FEE_TIER_INDEXES
            .iter()
            .map(|fee_tier_index| {
                let seeds = [b"whirlpool".as_slice(), WHIRLPOOLS_CONFIG.as_bytes(), mint_a.as_bytes(), mint_b.as_bytes(), &fee_tier_index.to_le_bytes()];
                Ok(find_program_address(&orca_whirlpool_program(), &seeds)?.0)
            })
            .collect()
    }

    pub fn has_adaptive_fee(&self) -> bool {
        self.fee_tier_index != self.tick_spacing
    }

    pub fn oracle_address(address: &Pubkey) -> Result<Pubkey, SwapperError> {
        Ok(find_program_address(&orca_whirlpool_program(), &[b"oracle".as_slice(), address.as_bytes()])?.0)
    }

    pub fn ticks_in_array(&self) -> i32 {
        TICK_ARRAY_SIZE * i32::from(self.tick_spacing)
    }

    pub fn clmm_swap(&self, a_to_b: bool) -> ClmmSwap {
        ClmmSwap {
            sqrt_price: self.sqrt_price,
            liquidity: self.liquidity,
            a_to_b,
            fee_on_input: true,
            sqrt_price_limit: if a_to_b { MIN_SQRT_PRICE } else { MAX_SQRT_PRICE },
        }
    }

    pub fn swap_start_tick_indexes(&self, a_to_b: bool) -> Vec<i32> {
        let ticks_in_array = self.ticks_in_array();
        let current_start = self.tick_current_index.div_euclid(ticks_in_array) * ticks_in_array;
        let shifted = !a_to_b && self.tick_current_index + i32::from(self.tick_spacing) >= current_start + ticks_in_array;
        let offsets = match (a_to_b, shifted) {
            (true, _) => [0, -1, -2],
            (false, true) => [1, 2, 3],
            (false, false) => [0, 1, 2],
        };
        offsets.iter().map(|offset| current_start + offset * ticks_in_array).filter(|start| self.is_valid_start_tick(*start)).collect()
    }

    pub fn supplemental_start_tick_indexes(&self, a_to_b: bool) -> Vec<i32> {
        let step = if a_to_b { -self.ticks_in_array() } else { self.ticks_in_array() };
        let swap_starts = self.swap_start_tick_indexes(a_to_b);
        let (Some(first), Some(last)) = (swap_starts.first(), swap_starts.last()) else {
            return vec![];
        };
        [first - step, last + step].into_iter().filter(|start| self.is_valid_start_tick(*start)).collect()
    }

    fn is_valid_start_tick(&self, start_tick_index: i32) -> bool {
        let ticks_in_array = self.ticks_in_array();
        if (MIN_TICK_INDEX..=MAX_TICK_INDEX).contains(&start_tick_index) {
            return start_tick_index % ticks_in_array == 0;
        }
        start_tick_index < MIN_TICK_INDEX && start_tick_index == MIN_TICK_INDEX - (MIN_TICK_INDEX % ticks_in_array + ticks_in_array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS};

    use crate::orca::testkit::{TEST_WHIRLPOOL, TEST_XAUT0_MINT, TEST_XAUT0_SOL_WHIRLPOOL, TEST_XAUT0_USDC_WHIRLPOOL};

    #[test]
    fn test_addresses() {
        let sol = Pubkey::from_base58(WSOL_TOKEN_ADDRESS).unwrap();
        let usdc = Pubkey::from_base58(USDC_TOKEN_MINT).unwrap();
        let xaut0 = Pubkey::from_base58(TEST_XAUT0_MINT).unwrap();
        let addresses = Whirlpool::addresses(&usdc, &sol).unwrap();
        let contains = |mint_a: &Pubkey, mint_b: &Pubkey, pool: &str| Whirlpool::addresses(mint_a, mint_b).unwrap().contains(&Pubkey::from_base58(pool).unwrap());

        assert_eq!(addresses, Whirlpool::addresses(&sol, &usdc).unwrap());
        assert_eq!(addresses[4].to_base58(), TEST_WHIRLPOOL);
        assert!(contains(&sol, &xaut0, TEST_XAUT0_SOL_WHIRLPOOL));
        assert!(contains(&xaut0, &usdc, TEST_XAUT0_USDC_WHIRLPOOL));
    }

    #[test]
    fn test_swap_start_tick_indexes() {
        let whirlpool = Whirlpool {
            tick_current_index: -11_000,
            ..Whirlpool::mock()
        };

        assert_eq!(whirlpool.swap_start_tick_indexes(true), vec![-11_264, -16_896, -22_528]);
        assert_eq!(whirlpool.swap_start_tick_indexes(false), vec![-11_264, -5_632, 0]);
        assert_eq!(whirlpool.supplemental_start_tick_indexes(true), vec![-5_632, -28_160]);
        assert_eq!(Whirlpool { tick_current_index: -5_690, ..whirlpool }.swap_start_tick_indexes(false), vec![-5_632, 0, 5_632]);
        assert_eq!(Whirlpool { tick_current_index: -443_000, ..whirlpool }.swap_start_tick_indexes(true), vec![-444_928]);
        assert_eq!(Whirlpool { tick_current_index: 443_000, ..whirlpool }.swap_start_tick_indexes(false), vec![439_296]);
    }
}
