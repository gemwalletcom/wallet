use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, SolanaDataSlice};

use crate::{
    SwapperError,
    clmm::{constant_product_output, fee_amount},
    solana_dex::{DexPool, PoolLayout},
};

pub(super) const LAYOUT: PoolLayout = PoolLayout {
    account_len: 752,
    mint_offsets: [400, 432],
    depth: SolanaDataSlice { offset: 720, length: size_of::<u64>() },
};
const SWAP_STATUSES: [u64; 3] = [1, 6, 7];
const WAITING_TRADE: u64 = 7;
pub(super) const AUTHORITY_NONCE: u64 = 254;

#[derive(Debug, Clone, PartialEq)]
struct Fees {
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
}

impl BorshDeserialize for Fees {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            swap_fee_numerator: reader.skip(48)?.read()?,
            swap_fee_denominator: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StateData {
    need_take_pnl_coin: u64,
    need_take_pnl_pc: u64,
    pool_open_time: u64,
}

impl BorshDeserialize for StateData {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            need_take_pnl_coin: reader.read()?,
            need_take_pnl_pc: reader.read()?,
            pool_open_time: reader.skip(16)?.read()?,
        };
        reader.skip(104)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pool {
    status: u64,
    nonce: u64,
    fees: Fees,
    state_data: StateData,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    pub lp_amount: u64,
}

impl BorshDeserialize for Pool {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            status: reader.read()?,
            nonce: reader.read()?,
            fees: reader.skip(112)?.read()?,
            state_data: reader.read()?,
            coin_vault: reader.read()?,
            pc_vault: reader.read()?,
            coin_mint: reader.read()?,
            pc_mint: reader.read()?,
            lp_amount: reader.skip(256)?.read()?,
        })
    }
}

impl DexPool for Pool {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.coin_mint, self.pc_mint)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.coin_vault, self.pc_vault)
    }
}

impl Pool {
    pub fn decode(data: &[u8]) -> Result<Self, SwapperError> {
        Self::deserialize(&mut &data[..]).map_err(SwapperError::compute_quote_error)
    }

    pub fn is_tradable(&self, timestamp: u64) -> bool {
        SWAP_STATUSES.contains(&self.status) && (self.status != WAITING_TRADE || timestamp >= self.state_data.pool_open_time) && self.coin_mint != self.pc_mint && self.lp_amount > 0 && self.nonce == AUTHORITY_NONCE
    }

    pub fn quote(&self, coin_to_pc: bool, amount_in: u64, coin_vault: u64, pc_vault: u64) -> Option<u64> {
        let coin = u128::from(coin_vault.checked_sub(self.state_data.need_take_pnl_coin)?);
        let pc = u128::from(pc_vault.checked_sub(self.state_data.need_take_pnl_pc)?);
        let (reserve_in, reserve_out) = if coin_to_pc { (coin, pc) } else { (pc, coin) };
        let denominator = u32::try_from(self.fees.swap_fee_denominator).ok().filter(|denominator| *denominator > 0)?;
        let fee = fee_amount(amount_in, u32::try_from(self.fees.swap_fee_numerator).ok()?, denominator);
        let output = constant_product_output(reserve_in, reserve_out, u128::from(amount_in.checked_sub(fee)?))?;
        u64::try_from(output).ok().filter(|output| *output > 0 && u128::from(*output) < reserve_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raydium_amm::testkit::{TEST_SOL_USDC_POOL, mock_pool_snapshot};
    use gem_solana::{USDC_TOKEN_MINT, models::TokenAccount, wrapped_sol::wrapped_sol_mint};

    #[test]
    fn test_layout_offsets() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();
        let at = |offset: usize, len: usize| &data[offset..offset + len];

        assert_eq!(u64::try_from(data.len()).unwrap(), LAYOUT.account_len);
        assert_eq!(at(LAYOUT.mint_offsets[0], 32), wrapped_sol_mint().as_bytes());
        assert_eq!(at(LAYOUT.mint_offsets[1], 32), Pubkey::from_base58(USDC_TOKEN_MINT).unwrap().as_bytes());
        assert_eq!((snapshot.pool.coin_mint, snapshot.pool.pc_mint), (wrapped_sol_mint(), Pubkey::from_base58(USDC_TOKEN_MINT).unwrap()));
        assert_eq!(at(LAYOUT.depth.offset, LAYOUT.depth.length), snapshot.pool.lp_amount.to_le_bytes());
        assert_eq!(
            (snapshot.pool.status, snapshot.pool.nonce, snapshot.pool.fees.swap_fee_numerator, snapshot.pool.fees.swap_fee_denominator),
            (6, AUTHORITY_NONCE, 25, 10_000)
        );
    }

    #[test]
    fn test_quote_matches_the_program() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let pool = snapshot.pool;
        let vault = |address: &Pubkey| TokenAccount::decode(snapshot.accounts[address].as_ref().unwrap()).unwrap().amount;
        let (coin, pc) = (vault(&pool.coin_vault), vault(&pool.pc_vault));
        let quote = |coin_to_pc: bool, amount_in: u64| pool.quote(coin_to_pc, amount_in, coin, pc);

        assert_eq!(quote(true, 1_000_000_000), Some(103_142_762));
        assert_eq!(quote(true, 10_000_000), Some(1_031_437));
        assert_eq!(quote(true, 1_000_000_000_000), Some(102_178_160_868));
        assert_eq!(quote(false, 100_000_000), Some(964_670_451));
        assert_eq!(quote(false, 1_000_000), Some(9_646_791));
        assert_eq!(pool.quote(true, 220_644_943, 105_543_926_999_285, 10_916_154_684_359), Some(22_763_676));
        assert_eq!(quote(true, 0), None);
        assert_eq!(
            Pool {
                state_data: StateData {
                    need_take_pnl_pc: pc + 1,
                    ..pool.state_data.clone()
                },
                ..pool.clone()
            }
            .quote(true, 1_000_000_000, coin, pc),
            None
        );
        assert_eq!(
            Pool {
                fees: Fees {
                    swap_fee_denominator: 0,
                    ..pool.fees.clone()
                },
                ..pool
            }
            .quote(true, 1_000_000_000, coin, pc),
            None
        );
    }

    #[test]
    fn test_is_tradable() {
        let pool = mock_pool_snapshot(TEST_SOL_USDC_POOL).pool;
        let opens_at = |status: u64, pool_open_time: u64| Pool {
            status,
            state_data: StateData { pool_open_time, ..pool.state_data.clone() },
            ..pool.clone()
        };

        assert!(pool.is_tradable(0));
        assert!(opens_at(1, 0).is_tradable(0));
        assert!(opens_at(7, 100).is_tradable(100));
        assert!(!opens_at(7, 100).is_tradable(99));
        assert!(opens_at(6, 100).is_tradable(99));
        assert!([0, 2, 3, 4, 5, 8].iter().all(|status| !opens_at(*status, 0).is_tradable(0)));
        assert!(!Pool { lp_amount: 0, ..pool.clone() }.is_tradable(0));
        assert!(!Pool { nonce: 253, ..pool }.is_tradable(0));
    }
}
