use gem_encoding::borsh::{DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::raydium_clmm_program};

use super::tick_math::{MAX_SQRT_PRICE, MIN_SQRT_PRICE};
use crate::{SwapperError, clmm::ClmmSwap, solana_dex::DexPool};

const AMM_CONFIG_INDEXES: [u16; 8] = [4, 1, 19, 10, 5, 3, 8, 15];
pub(super) const TICK_ARRAY_SIZE: i32 = 60;
const TICK_ARRAY_BITMAP_SIZE: i32 = 512;
const SWAP_TICK_ARRAYS: usize = 3;
const SWAP_DISABLED: u8 = 1 << 4;
const DYNAMIC_FEE_INFO_LEN: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DynamicFeeInfo {
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub dynamic_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub tick_spacing_index_reference: i32,
    pub volatility_reference: u32,
    pub volatility_accumulator: u32,
    pub last_update_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum SwapTickArrays {
    Initialized(Vec<i32>),
    Consecutive(Vec<i32>),
}

impl SwapTickArrays {
    pub fn starts(&self) -> &[i32] {
        match self {
            Self::Initialized(starts) | Self::Consecutive(starts) => starts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AmmConfig {
    _bump: u8,
    _index: u16,
    _owner: Pubkey,
    _protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PoolState {
    _bump: [u8; 1],
    pub amm_config: Pubkey,
    _owner: Pubkey,
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub observation_key: Pubkey,
    _mint_decimals: [u8; 2],
    pub tick_spacing: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current: i32,
    _fee_growths_and_protocol_fees: [u8; 52],
    _padding: [u8; 64],
    status: u8,
    fee_on: u8,
    _seed_index_and_padding: [u8; 6],
    _reward_infos: [u8; 507],
    tick_array_bitmap: [u64; 16],
    _padding_and_fund_fees: [u8; 48],
    open_time: u64,
    _recent_epoch: u64,
    dynamic_fee_info: [u8; DYNAMIC_FEE_INFO_LEN],
}

impl DynamicFeeInfo {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            filter_period: reader.u16()?,
            decay_period: reader.u16()?,
            reduction_factor: reader.u16()?,
            dynamic_fee_control: reader.u32()?,
            max_volatility_accumulator: reader.u32()?,
            tick_spacing_index_reference: reader.i32()?,
            volatility_reference: reader.u32()?,
            volatility_accumulator: reader.u32()?,
            last_update_timestamp: reader.u64()?,
        })
    }
}

impl AnchorAccount for AmmConfig {
    const NAME: &'static str = "AmmConfig";

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            _bump: reader.u8()?,
            _index: reader.u16()?,
            _owner: Pubkey::read(reader)?,
            _protocol_fee_rate: reader.u32()?,
            trade_fee_rate: reader.u32()?,
        })
    }
}

impl AnchorAccount for PoolState {
    const NAME: &'static str = "PoolState";

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            _bump: reader.bytes()?,
            amm_config: Pubkey::read(reader)?,
            _owner: Pubkey::read(reader)?,
            token_mint_0: Pubkey::read(reader)?,
            token_mint_1: Pubkey::read(reader)?,
            token_vault_0: Pubkey::read(reader)?,
            token_vault_1: Pubkey::read(reader)?,
            observation_key: Pubkey::read(reader)?,
            _mint_decimals: reader.bytes()?,
            tick_spacing: reader.u16()?,
            liquidity: reader.u128()?,
            sqrt_price: reader.u128()?,
            tick_current: reader.i32()?,
            _fee_growths_and_protocol_fees: reader.bytes()?,
            _padding: reader.bytes()?,
            status: reader.u8()?,
            fee_on: reader.u8()?,
            _seed_index_and_padding: reader.bytes()?,
            _reward_infos: reader.bytes()?,
            tick_array_bitmap: reader.array(Reader::u64)?,
            _padding_and_fund_fees: reader.bytes()?,
            open_time: reader.u64()?,
            _recent_epoch: reader.u64()?,
            dynamic_fee_info: reader.bytes()?,
        })
    }
}

impl DexPool for PoolState {
    fn mints(&self) -> (Pubkey, Pubkey) {
        (self.token_mint_0, self.token_mint_1)
    }

    fn token_vaults(&self) -> (Pubkey, Pubkey) {
        (self.token_vault_0, self.token_vault_1)
    }
}

impl PoolState {
    pub fn addresses(mint_a: &Pubkey, mint_b: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        let (mint_0, mint_1) = if mint_a < mint_b { (mint_a, mint_b) } else { (mint_b, mint_a) };
        AMM_CONFIG_INDEXES
            .iter()
            .map(|index| {
                let config = find_program_address(&raydium_clmm_program(), &[b"amm_config".as_slice(), &index.to_be_bytes()])?.0;
                Ok(find_program_address(&raydium_clmm_program(), &[b"pool".as_slice(), config.as_bytes(), mint_0.as_bytes(), mint_1.as_bytes()])?.0)
            })
            .collect()
    }

    pub fn clmm_swap(&self, zero_for_one: bool) -> ClmmSwap {
        ClmmSwap {
            sqrt_price: self.sqrt_price,
            liquidity: self.liquidity,
            a_to_b: zero_for_one,
            fee_on_input: self.is_fee_on_input(zero_for_one),
            sqrt_price_limit: if zero_for_one { MIN_SQRT_PRICE + 1 } else { MAX_SQRT_PRICE - 1 },
        }
    }

    fn is_fee_on_input(&self, zero_for_one: bool) -> bool {
        match self.fee_on {
            1 => zero_for_one,
            2 => !zero_for_one,
            _ => true,
        }
    }

    pub fn is_tradable(&self, timestamp: u64) -> bool {
        self.liquidity > 0 && self.status & SWAP_DISABLED == 0 && timestamp > self.open_time
    }

    pub(super) fn dynamic_fee(&self) -> Result<Option<DynamicFeeInfo>, SwapperError> {
        if self.dynamic_fee_info.iter().all(|byte| *byte == 0) {
            return Ok(None);
        }
        Reader::decode_prefix(&self.dynamic_fee_info, DynamicFeeInfo::read).map(Some).map_err(SwapperError::compute_quote_error)
    }

    pub fn ticks_in_array(&self) -> i32 {
        TICK_ARRAY_SIZE * i32::from(self.tick_spacing)
    }

    pub(super) fn swap_tick_arrays(&self, zero_for_one: bool) -> Result<SwapTickArrays, SwapperError> {
        let ticks_in_array = self.ticks_in_array();
        let current_array = self.tick_current.div_euclid(ticks_in_array);
        let current = current_array + TICK_ARRAY_BITMAP_SIZE;
        if !(0..2 * TICK_ARRAY_BITMAP_SIZE).contains(&current) {
            let direction = if zero_for_one { -1 } else { 1 };
            return Ok(SwapTickArrays::Consecutive((0..SWAP_TICK_ARRAYS as i32).map(|offset| (current_array + offset * direction) * ticks_in_array).collect()));
        }
        let initialized = |position: &i32| self.is_tick_array_initialized(*position);
        let positions = if zero_for_one {
            (0..=current).rev().filter(initialized).take(SWAP_TICK_ARRAYS).collect::<Vec<_>>()
        } else {
            (current..2 * TICK_ARRAY_BITMAP_SIZE).filter(initialized).take(SWAP_TICK_ARRAYS).collect()
        };
        if positions.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        Ok(SwapTickArrays::Initialized(positions.into_iter().map(|position| (position - TICK_ARRAY_BITMAP_SIZE) * ticks_in_array).collect()))
    }

    pub fn bitmap_extension_address(address: &Pubkey) -> Result<Pubkey, SwapperError> {
        Ok(find_program_address(&raydium_clmm_program(), &[b"pool_tick_array_bitmap_extension".as_slice(), address.as_bytes()])?.0)
    }

    fn is_tick_array_initialized(&self, position: i32) -> bool {
        let position = usize::try_from(position).unwrap_or_default();
        self.tick_array_bitmap[position / 64] & (1 << (position % 64)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raydium_clmm::testkit::{TEST_EXTENSION_POOL, TEST_LIMIT_ORDER_POOL, mock_pool_snapshot};
    use gem_solana::{USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS};

    #[test]
    fn test_addresses() {
        let key = |address: &str| Pubkey::from_base58(address).unwrap();
        let sol_usdc = PoolState::addresses(&key(USDC_TOKEN_MINT), &key(WSOL_TOKEN_ADDRESS)).unwrap();
        let xdc_usdc = PoolState::addresses(&key("4TaFLLV3uhgu3HUTgbyLTuumBKwJ3DufyuLLnVN8YLwp"), &key(USDC_TOKEN_MINT)).unwrap();

        assert_eq!(sol_usdc, PoolState::addresses(&key(WSOL_TOKEN_ADDRESS), &key(USDC_TOKEN_MINT)).unwrap());
        assert_eq!(sol_usdc[6], key("3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv"));
        assert_eq!(xdc_usdc[0], key("2pyECnq9q8atTZvCmx1FarbjNJrYg9xye1UNz6An2wgM"));
    }

    #[test]
    fn test_swap_tick_arrays() {
        let limit_orders = mock_pool_snapshot(TEST_LIMIT_ORDER_POOL).pool;
        let extension = mock_pool_snapshot(TEST_EXTENSION_POOL).pool;
        let ticks_in_array = extension.ticks_in_array();
        let current = extension.tick_current.div_euclid(ticks_in_array) * ticks_in_array;

        assert_eq!(limit_orders.swap_tick_arrays(true).unwrap(), SwapTickArrays::Initialized(vec![0, -60, -120]));
        assert_eq!(extension.swap_tick_arrays(false).unwrap(), SwapTickArrays::Consecutive(vec![current, current + ticks_in_array, current + 2 * ticks_in_array]));
        assert_eq!(extension.swap_tick_arrays(true).unwrap(), SwapTickArrays::Consecutive(vec![current, current - ticks_in_array, current - 2 * ticks_in_array]));
    }

    #[test]
    fn test_is_fee_on_input() {
        let extension = mock_pool_snapshot(TEST_EXTENSION_POOL).pool;
        let limit_orders = mock_pool_snapshot(TEST_LIMIT_ORDER_POOL).pool;

        assert!(extension.is_fee_on_input(true));
        assert!(!extension.is_fee_on_input(false));
        assert!(limit_orders.is_fee_on_input(true));
        assert!(limit_orders.is_fee_on_input(false));
    }
}
