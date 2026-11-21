use alloy_primitives::U256;
use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, SolanaDataSlice, anchor::AnchorAccount, models::Clock};

use super::fee::{BaseFee, DynamicFee, FeePoint};
use crate::{
    clmm::{constant_product_output, divide, fee_amount},
    solana_dex::{ActivationType, DexPool, PoolLayout},
};

pub(super) const LAYOUT: PoolLayout = PoolLayout {
    account_len: 1112,
    mint_offsets: [168, 200],
    depth: SolanaDataSlice { offset: 360, length: size_of::<u128>() },
};
const FEE_DENOMINATOR: u32 = 1_000_000_000;
const POOL_ENABLED: u8 = 0;
const BOTH_TOKEN: u8 = 0;
const ONLY_B: u8 = 1;
const COMPOUNDING: u8 = 2;
const Q128_RESOLUTION: usize = 128;

#[derive(Debug, Clone, PartialEq)]
struct PoolFees {
    base_fee: BaseFee,
    dynamic_fee: DynamicFee,
    init_sqrt_price: u128,
}

impl BorshDeserialize for PoolFees {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            base_fee: reader.read()?,
            dynamic_fee: reader.skip(16)?.read()?,
            init_sqrt_price: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pool {
    pool_fees: PoolFees,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub liquidity: u128,
    sqrt_min_price: u128,
    sqrt_max_price: u128,
    sqrt_price: u128,
    activation_point: u64,
    activation_type: ActivationType,
    pool_status: u8,
    collect_fee_mode: u8,
    fee_version: u8,
    token_a_amount: u64,
    token_b_amount: u64,
}

impl BorshDeserialize for Pool {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            pool_fees: reader.read()?,
            token_a_mint: reader.read()?,
            token_b_mint: reader.read()?,
            token_a_vault: reader.read()?,
            token_b_vault: reader.read()?,
            liquidity: reader.skip(64)?.read()?,
            sqrt_min_price: reader.skip(48)?.read()?,
            sqrt_max_price: reader.read()?,
            sqrt_price: reader.read()?,
            activation_point: reader.read()?,
            activation_type: reader.read()?,
            pool_status: reader.read()?,
            collect_fee_mode: reader.skip(2)?.read()?,
            fee_version: reader.skip(1)?.read()?,
            token_a_amount: reader.skip(193)?.read()?,
            token_b_amount: reader.read()?,
        })
    }
}

impl AnchorAccount for Pool {
    const NAME: &'static str = "Pool";
}

impl DexPool for Pool {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.token_a_mint, self.token_b_mint)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.token_a_vault, self.token_b_vault)
    }
}

impl Pool {
    pub fn is_tradable(&self) -> bool {
        self.liquidity > 0 && self.sqrt_price > 0 && self.pool_status == POOL_ENABLED
    }

    pub fn quote(&self, a_to_b: bool, amount_in: u64, clock: &Clock) -> Option<u64> {
        let current = self.activation_type.current_point(clock)?;
        if !self.is_tradable() || current < self.activation_point {
            return None;
        }
        let fee_rate = self.fee_rate(current, a_to_b)?;
        let fee_on_input = match (self.collect_fee_mode, a_to_b) {
            (BOTH_TOKEN | ONLY_B | COMPOUNDING, true) | (BOTH_TOKEN, false) => false,
            (ONLY_B | COMPOUNDING, false) => true,
            _ => return None,
        };
        let swap_in = if fee_on_input { amount_in - fee_amount(amount_in, fee_rate, FEE_DENOMINATOR) } else { amount_in };
        let swap_out = self.swap_output(a_to_b, swap_in)?;
        Some(if fee_on_input { swap_out } else { swap_out - fee_amount(swap_out, fee_rate, FEE_DENOMINATOR) })
    }

    fn fee_rate(&self, current: u64, a_to_b: bool) -> Option<u32> {
        let max_fee_rate: u32 = match self.fee_version {
            0 => 500_000_000,
            1 => 990_000_000,
            _ => return None,
        };
        let point = FeePoint {
            current,
            activation: self.activation_point,
            a_to_b,
            init_sqrt_price: self.pool_fees.init_sqrt_price,
            sqrt_price: self.sqrt_price,
        };
        let total = self.pool_fees.dynamic_fee.variable_fee()?.checked_add(u128::from(self.pool_fees.base_fee.numerator(&point)?))?;
        Some(u32::try_from(total).map_or(max_fee_rate, |total| total.min(max_fee_rate)))
    }

    fn swap_output(&self, a_to_b: bool, amount_in: u64) -> Option<u64> {
        if self.collect_fee_mode == COMPOUNDING {
            let (reserve_in, reserve_out) = if a_to_b { (self.token_a_amount, self.token_b_amount) } else { (self.token_b_amount, self.token_a_amount) };
            let output = constant_product_output(u128::from(reserve_in), u128::from(reserve_out), u128::from(amount_in))?;
            return Some(u64::try_from(output).unwrap_or_default());
        }
        let (liquidity, sqrt_price) = (U256::from(self.liquidity), U256::from(self.sqrt_price));
        let output = if a_to_b {
            let next_sqrt_price = divide(liquidity * sqrt_price, liquidity + U256::from(amount_in) * sqrt_price, true);
            if next_sqrt_price < U256::from(self.sqrt_min_price) {
                return None;
            }
            (liquidity * (sqrt_price - next_sqrt_price)) >> Q128_RESOLUTION
        } else {
            let next_sqrt_price = sqrt_price + (U256::from(amount_in) << Q128_RESOLUTION) / liquidity;
            if next_sqrt_price > U256::from(self.sqrt_max_price) {
                return None;
            }
            divide(liquidity * (next_sqrt_price - sqrt_price), sqrt_price * next_sqrt_price, false)
        };
        u64::try_from(output).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meteora_damm_v2::testkit::{TEST_BOTH_TOKEN_POOL, mock_pool_snapshot};
    use gem_solana::WSOL_TOKEN_ADDRESS;

    #[test]
    fn test_quote_pool_state() {
        let snapshot = mock_pool_snapshot(TEST_BOTH_TOKEN_POOL);
        let (pool, clock) = (snapshot.pool, snapshot.clock);
        let timestamp = clock.timestamp();

        assert_eq!(pool.quote(true, 326_754_966, &clock), Some(9_936_136));
        assert_eq!(Pool { pool_status: 1, ..pool.clone() }.quote(true, 326_754_966, &clock), None);
        assert_eq!(
            Pool {
                activation_point: timestamp + 1,
                ..pool.clone()
            }
            .quote(true, 326_754_966, &clock),
            None
        );
        assert_eq!(Pool { activation_point: timestamp, ..pool.clone() }.quote(true, 326_754_966, &clock), Some(9_936_136));
        assert_eq!(
            Pool {
                sqrt_max_price: pool.sqrt_price + 1,
                ..pool.clone()
            }
            .quote(false, 10_000_000, &clock),
            None
        );
        assert_eq!(
            Pool {
                sqrt_min_price: pool.sqrt_price,
                ..pool.clone()
            }
            .quote(true, 326_754_966, &clock),
            None
        );
        assert_eq!(Pool { liquidity: 0, ..pool }.quote(false, 10_000_000, &clock), None);
    }

    #[test]
    fn test_layout_offsets() {
        let snapshot = mock_pool_snapshot(TEST_BOTH_TOKEN_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();
        let at = |offset: usize, len: usize| &data[offset..offset + len];

        assert_eq!(u64::try_from(data.len()).unwrap(), LAYOUT.account_len);
        assert_eq!(at(LAYOUT.mint_offsets[0], 32), snapshot.pool.token_a_mint.as_bytes());
        assert_eq!(at(LAYOUT.mint_offsets[1], 32), snapshot.pool.token_b_mint.as_bytes());
        assert_eq!(at(LAYOUT.depth.offset, LAYOUT.depth.length), snapshot.pool.liquidity.to_le_bytes());
    }

    #[test]
    fn test_decode() {
        let pool = mock_pool_snapshot(TEST_BOTH_TOKEN_POOL).pool;

        assert_eq!(
            [pool.token_a_mint, pool.token_b_mint, pool.token_a_vault, pool.token_b_vault].map(|key| key.to_base58()),
            [
                "HmjCoarLh5duURfJ333DwfFiPyTCgFT35pRSAoP8pump",
                WSOL_TOKEN_ADDRESS,
                "2dc43wtcfvGVEfVmTzjJkEvZfoVPByZkM1vgaRMRTLYP",
                "BYu51Sj9vCRuUTdbzU4dNxg4jugDsCNkrNxbTFPKUAsA",
            ]
        );
    }
}
