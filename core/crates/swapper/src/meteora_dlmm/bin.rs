use alloy_primitives::U256;
use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::meteora_dlmm_program};

use crate::{SwapperError, clmm::divide};

pub(super) const BINS_PER_ARRAY: i32 = 70;
const SCALE_OFFSET: usize = 64;
const ONE: u128 = 1 << SCALE_OFFSET;
const BASIS_POINT_MAX: u128 = 10_000;
const MAX_EXPONENT: u32 = 0x80000;
const LIMIT_ORDER_VERSION: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BinArray {
    pub index: i64,
    version: u8,
    pub lb_pair: Pubkey,
    bins: [Bin; BINS_PER_ARRAY as usize],
}

impl BorshDeserialize for BinArray {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            index: reader.read()?,
            version: reader.read()?,
            lb_pair: reader.skip(7)?.read()?,
            bins: reader.read()?,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Bin {
    amount_x: u64,
    amount_y: u64,
    price: u128,
    open_order_amount: u64,
    processed_order_remaining_amount: u64,
    limit_order_ask_side: u8,
}

impl BorshDeserialize for Bin {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            amount_x: reader.read()?,
            amount_y: reader.read()?,
            price: reader.read()?,
            open_order_amount: reader.skip(80)?.read()?,
            processed_order_remaining_amount: reader.skip(8)?.read()?,
            limit_order_ask_side: reader.skip(4)?.read()?,
        };
        reader.skip(3)?;
        Ok(value)
    }
}

impl AnchorAccount for BinArray {
    const NAME: &'static str = "BinArray";
}

pub(super) fn array_index(bin_id: i32) -> i64 {
    i64::from(bin_id).div_euclid(i64::from(BINS_PER_ARRAY))
}

pub(super) fn price_from_id(bin_id: i32, bin_step: u16) -> Option<u128> {
    pow(ONE + (u128::from(bin_step) << SCALE_OFFSET) / BASIS_POINT_MAX, bin_id)
}

fn pow(base: u128, exponent: i32) -> Option<u128> {
    let power = exponent.unsigned_abs();
    if power == 0 {
        return Some(ONE);
    }
    if power >= MAX_EXPONENT {
        return None;
    }
    let (square, invert) = if base >= ONE { (u128::MAX / base, exponent > 0) } else { (base, exponent < 0) };
    let (_, result) = (0..19).try_fold((square, ONE), |(square, result), bit| {
        let square = if bit > 0 { square.checked_mul(square)? >> SCALE_OFFSET } else { square };
        let result = if power & (1 << bit) != 0 { result.checked_mul(square)? >> SCALE_OFFSET } else { result };
        Some((square, result))
    })?;
    match (result, invert) {
        (0, _) => None,
        (result, true) => Some(u128::MAX / result),
        (result, false) => Some(result),
    }
}

fn to_amount(value: U256) -> Option<u64> {
    u64::try_from(value).ok()
}

fn amount_out(amount_in: u64, price: u128, swap_for_y: bool) -> Option<u64> {
    let (amount_in, price) = (U256::from(amount_in), U256::from(price));
    match swap_for_y {
        true => to_amount((price * amount_in) >> SCALE_OFFSET),
        false => to_amount(divide(amount_in << SCALE_OFFSET, price, false)),
    }
}

fn amount_in(amount_out: u64, price: u128, swap_for_y: bool) -> Option<u64> {
    let (amount_out, price) = (U256::from(amount_out), U256::from(price));
    match swap_for_y {
        true => to_amount(divide(amount_out << SCALE_OFFSET, price, true)),
        false => to_amount(divide(amount_out * price, U256::from(ONE), true)),
    }
}

fn fill(amount: u64, max_out: u64, price: u128, swap_for_y: bool) -> Option<(u64, u64)> {
    if max_out == 0 {
        return Some((amount, 0));
    }
    let max_in = amount_in(max_out, price, swap_for_y)?;
    match amount >= max_in {
        true => Some((amount - max_in, max_out)),
        false => Some((0, amount_out(amount, price, swap_for_y)?)),
    }
}

impl BinArray {
    pub fn address(pair: &Pubkey, index: i64) -> Result<Pubkey, SwapperError> {
        Ok(find_program_address(&meteora_dlmm_program(), &[b"bin_array".as_slice(), pair.as_bytes(), &index.to_le_bytes()])?.0)
    }

    pub fn bin(&self, bin_id: i32) -> Option<(&Bin, bool)> {
        let offset = usize::try_from(i64::from(bin_id) - self.index * i64::from(BINS_PER_ARRAY)).ok()?;
        Some((self.bins.get(offset)?, self.version >= LIMIT_ORDER_VERSION))
    }

    pub fn bin_ids(&self, active_id: i32, swap_for_y: bool) -> Vec<i32> {
        let lower = i32::try_from(self.index * i64::from(BINS_PER_ARRAY)).unwrap_or_default();
        let upper = lower + BINS_PER_ARRAY - 1;
        match (array_index(active_id) == self.index, swap_for_y) {
            (true, true) => (lower..=active_id).rev().collect(),
            (true, false) => (active_id..=upper).collect(),
            (false, true) => (lower..=upper).rev().collect(),
            (false, false) => (lower..=upper).collect(),
        }
    }
}

impl Bin {
    pub fn price(&self, bin_id: i32, bin_step: u16) -> Option<u128> {
        match self.price {
            0 => price_from_id(bin_id, bin_step),
            price => Some(price),
        }
    }

    fn limit_orders(&self, swap_for_y: bool, has_limit_orders: bool) -> (u64, u64) {
        let fills_side = (swap_for_y && self.limit_order_ask_side == 0) || (!swap_for_y && self.limit_order_ask_side != 0);
        match has_limit_orders && fills_side {
            true => (self.processed_order_remaining_amount, self.open_order_amount),
            false => (0, 0),
        }
    }

    fn market_maker_amount(&self, swap_for_y: bool) -> u64 {
        if swap_for_y { self.amount_y } else { self.amount_x }
    }

    pub fn max_out(&self, swap_for_y: bool, has_limit_orders: bool) -> u64 {
        let (processed, open) = self.limit_orders(swap_for_y, has_limit_orders);
        self.market_maker_amount(swap_for_y).saturating_add(processed).saturating_add(open)
    }

    pub fn fill(&self, amount: u64, price: u128, swap_for_y: bool, has_limit_orders: bool) -> Option<(u64, u64)> {
        let (processed, open) = self.limit_orders(swap_for_y, has_limit_orders);
        [self.market_maker_amount(swap_for_y), processed, open].into_iter().try_fold((amount, 0u64), |(left, out), max_out| {
            if left == 0 {
                return Some((left, out));
            }
            let (left, filled) = fill(left, max_out, price, swap_for_y)?;
            Some((left, out.checked_add(filled)?))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_from_id() {
        assert_eq!(price_from_id(0, 1), Some(ONE));
        assert_eq!(price_from_id(1, 100), Some(ONE + ONE / 100));
        assert_eq!(price_from_id(-1, 100), Some(u128::MAX / (ONE + ONE / 100)));
        assert_eq!(price_from_id(i32::try_from(MAX_EXPONENT).unwrap(), 1), None);
    }

    #[test]
    fn test_array_index() {
        assert_eq!(array_index(0), 0);
        assert_eq!(array_index(69), 0);
        assert_eq!(array_index(70), 1);
        assert_eq!(array_index(-1), -1);
        assert_eq!(array_index(-70), -1);
        assert_eq!(array_index(-71), -2);
    }

    #[test]
    fn test_amounts_round_against_the_trader() {
        let price = ONE + ONE / 3;

        assert_eq!(amount_out(3, price, true), Some(3));
        assert_eq!(amount_in(4, price, true), Some(4));
        assert_eq!(amount_out(4, price, false), Some(3));
        assert_eq!(amount_in(3, price, false), Some(4));
        assert_eq!(fill(10, 0, price, true), Some((10, 0)));
        assert_eq!(fill(10, 4, price, true), Some((6, 4)));
        assert_eq!(fill(2, 4, price, true), Some((0, 2)));
    }
}
