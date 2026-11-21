use gem_encoding::borsh::{DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::raydium_clmm_program};

use super::pool::{PoolState, SwapTickArrays, TICK_ARRAY_SIZE};
use crate::SwapperError;

struct TickState {
    _tick: i32,
    liquidity_net: i128,
    liquidity_gross: u128,
    _fee_and_reward_growths: [u8; 80],
    _order_phase: u64,
    orders_amount: u64,
    part_filled_orders_remaining: u64,
    _unfilled_ratio_and_padding: [u8; 28],
}

struct TickArrayState {
    pool_id: Pubkey,
    start_tick_index: i32,
    ticks: Vec<TickState>,
}

impl TickState {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            _tick: reader.i32()?,
            liquidity_net: reader.i128()?,
            liquidity_gross: reader.u128()?,
            _fee_and_reward_growths: reader.bytes()?,
            _order_phase: reader.u64()?,
            orders_amount: reader.u64()?,
            part_filled_orders_remaining: reader.u64()?,
            _unfilled_ratio_and_padding: reader.bytes()?,
        })
    }
}

impl AnchorAccount for TickArrayState {
    const NAME: &'static str = "TickArrayState";

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            pool_id: Pubkey::read(reader)?,
            start_tick_index: reader.i32()?,
            ticks: reader.list(TICK_ARRAY_SIZE as usize, TickState::read)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PoolTick {
    pub index: i32,
    pub liquidity_net: i128,
    pub has_liquidity: bool,
    pub limit_orders: u64,
}

pub(super) struct TickArray;

impl TickArray {
    pub fn address(pool: &Pubkey, start_tick_index: i32) -> Result<Pubkey, SwapperError> {
        Ok(find_program_address(&raydium_clmm_program(), &[b"tick_array".as_slice(), pool.as_bytes(), &start_tick_index.to_be_bytes()])?.0)
    }

    pub fn addresses(pool: &Pubkey, start_tick_indexes: &[i32]) -> Result<Vec<Pubkey>, SwapperError> {
        start_tick_indexes.iter().map(|start| Self::address(pool, *start)).collect()
    }
}

pub(super) fn swap_ticks(address: &Pubkey, pool: &PoolState, zero_for_one: bool, accounts: &[Option<Vec<u8>>]) -> Result<Vec<PoolTick>, SwapperError> {
    let tick_arrays = pool.swap_tick_arrays(zero_for_one)?;
    if tick_arrays.starts().len() != accounts.len() {
        return Err(SwapperError::compute_quote_error("Unexpected tick arrays"));
    }
    let present = match tick_arrays {
        SwapTickArrays::Initialized(_) => accounts
            .iter()
            .map(|account| account.as_ref().ok_or_else(|| SwapperError::compute_quote_error("Missing tick array")))
            .collect::<Result<Vec<_>, _>>()?,
        SwapTickArrays::Consecutive(_) => accounts.iter().map_while(Option::as_ref).collect(),
    };
    let arrays = tick_arrays
        .starts()
        .iter()
        .zip(present)
        .map(|(start, data)| {
            let array = TickArrayState::decode(data)?;
            if array.start_tick_index != *start || array.pool_id != *address {
                return Err(SwapperError::compute_quote_error("Unexpected tick array"));
            }
            Ok(array)
        })
        .collect::<Result<Vec<_>, SwapperError>>()?;
    let spacing = i32::from(pool.tick_spacing);
    let ticks = arrays.iter().flat_map(|array| {
        let initialized = (0..TICK_ARRAY_SIZE).zip(&array.ticks).filter_map(move |(offset, tick)| {
            let limit_orders = tick.orders_amount.saturating_add(tick.part_filled_orders_remaining);
            (tick.liquidity_gross > 0 || limit_orders > 0).then_some(PoolTick {
                index: array.start_tick_index + offset * spacing,
                liquidity_net: tick.liquidity_net,
                has_liquidity: tick.liquidity_gross > 0,
                limit_orders,
            })
        });
        let ordered: Vec<_> = if zero_for_one { initialized.rev().collect() } else { initialized.collect() };
        ordered
    });
    Ok(ticks.filter(|tick| if zero_for_one { tick.index <= pool.tick_current } else { tick.index > pool.tick_current }).collect())
}
