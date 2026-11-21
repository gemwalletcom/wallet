use gem_encoding::borsh::{DecodeError, Reader};
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
    _min_separate_numerator: u64,
    _min_separate_denominator: u64,
    _trade_fee_numerator: u64,
    _trade_fee_denominator: u64,
    _pnl_numerator: u64,
    _pnl_denominator: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct StateData {
    need_take_pnl_coin: u64,
    need_take_pnl_pc: u64,
    _total_pnl_pc: u64,
    _total_pnl_coin: u64,
    pool_open_time: u64,
    _padding: [u64; 2],
    _orderbook_to_init_time: u64,
    _swap_coin_in_amount: u128,
    _swap_pc_out_amount: u128,
    _swap_acc_pc_fee: u64,
    _swap_pc_in_amount: u128,
    _swap_coin_out_amount: u128,
    _swap_acc_coin_fee: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pool {
    status: u64,
    nonce: u64,
    _order_num: u64,
    _depth: u64,
    _coin_decimals: u64,
    _pc_decimals: u64,
    _state: u64,
    _reset_flag: u64,
    _min_size: u64,
    _vol_max_cut_ratio: u64,
    _amount_wave: u64,
    _coin_lot_size: u64,
    _pc_lot_size: u64,
    _min_price_multiplier: u64,
    _max_price_multiplier: u64,
    _sys_decimal_value: u64,
    fees: Fees,
    state_data: StateData,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    _lp_mint: Pubkey,
    _open_orders: Pubkey,
    _market: Pubkey,
    _market_program: Pubkey,
    _target_orders: Pubkey,
    _padding1: [u64; 8],
    _amm_owner: Pubkey,
    pub lp_amount: u64,
}

impl Fees {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            _min_separate_numerator: reader.u64()?,
            _min_separate_denominator: reader.u64()?,
            _trade_fee_numerator: reader.u64()?,
            _trade_fee_denominator: reader.u64()?,
            _pnl_numerator: reader.u64()?,
            _pnl_denominator: reader.u64()?,
            swap_fee_numerator: reader.u64()?,
            swap_fee_denominator: reader.u64()?,
        })
    }
}

impl StateData {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            need_take_pnl_coin: reader.u64()?,
            need_take_pnl_pc: reader.u64()?,
            _total_pnl_pc: reader.u64()?,
            _total_pnl_coin: reader.u64()?,
            pool_open_time: reader.u64()?,
            _padding: reader.array(Reader::u64)?,
            _orderbook_to_init_time: reader.u64()?,
            _swap_coin_in_amount: reader.u128()?,
            _swap_pc_out_amount: reader.u128()?,
            _swap_acc_pc_fee: reader.u64()?,
            _swap_pc_in_amount: reader.u128()?,
            _swap_coin_out_amount: reader.u128()?,
            _swap_acc_coin_fee: reader.u64()?,
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
        Reader::decode_prefix(data, Self::read).map_err(SwapperError::compute_quote_error)
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            status: reader.u64()?,
            nonce: reader.u64()?,
            _order_num: reader.u64()?,
            _depth: reader.u64()?,
            _coin_decimals: reader.u64()?,
            _pc_decimals: reader.u64()?,
            _state: reader.u64()?,
            _reset_flag: reader.u64()?,
            _min_size: reader.u64()?,
            _vol_max_cut_ratio: reader.u64()?,
            _amount_wave: reader.u64()?,
            _coin_lot_size: reader.u64()?,
            _pc_lot_size: reader.u64()?,
            _min_price_multiplier: reader.u64()?,
            _max_price_multiplier: reader.u64()?,
            _sys_decimal_value: reader.u64()?,
            fees: Fees::read(reader)?,
            state_data: StateData::read(reader)?,
            coin_vault: Pubkey::read(reader)?,
            pc_vault: Pubkey::read(reader)?,
            coin_mint: Pubkey::read(reader)?,
            pc_mint: Pubkey::read(reader)?,
            _lp_mint: Pubkey::read(reader)?,
            _open_orders: Pubkey::read(reader)?,
            _market: Pubkey::read(reader)?,
            _market_program: Pubkey::read(reader)?,
            _target_orders: Pubkey::read(reader)?,
            _padding1: reader.array(Reader::u64)?,
            _amm_owner: Pubkey::read(reader)?,
            lp_amount: reader.u64()?,
        })
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
