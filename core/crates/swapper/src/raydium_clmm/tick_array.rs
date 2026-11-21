use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::raydium_clmm_program};

use super::pool::{PoolState, SwapTickArrays, TICK_ARRAY_SIZE};
use crate::SwapperError;

#[derive(Default)]
struct TickState {
    liquidity_net: i128,
    liquidity_gross: u128,
    orders_amount: u64,
    part_filled_orders_remaining: u64,
}

impl BorshDeserialize for TickState {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            liquidity_net: reader.skip(4)?.read()?,
            liquidity_gross: reader.read()?,
            orders_amount: reader.skip(88)?.read()?,
            part_filled_orders_remaining: reader.read()?,
        };
        reader.skip(28)?;
        Ok(value)
    }
}

struct TickArrayState {
    pool_id: Pubkey,
    start_tick_index: i32,
    ticks: [TickState; TICK_ARRAY_SIZE as usize],
}

impl BorshDeserialize for TickArrayState {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            pool_id: reader.read()?,
            start_tick_index: reader.read()?,
            ticks: reader.read()?,
        })
    }
}

impl AnchorAccount for TickArrayState {
    const NAME: &'static str = "TickArrayState";
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
