use gem_encoding::borsh::{DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount};

use crate::{
    SwapperError,
    clmm::{MAX_FEE_RATE, MAX_TICK_INDEX, MIN_TICK_INDEX, SwapFee, VOLATILITY_SCALE, accumulated_volatility, decayed_volatility, sqrt_price_from_tick_index, tick_index_from_sqrt_price, volatility_fee_rate},
};

const MAX_REFERENCE_AGE: u64 = 3_600;

#[derive(Debug, Clone, Copy, PartialEq)]
struct AdaptiveFeeConstants {
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    adaptive_fee_control_factor: u32,
    max_volatility_accumulator: u32,
    tick_group_size: u16,
    _major_swap_threshold_ticks: u16,
    _reserved: [u8; 16],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AdaptiveFeeVariables {
    last_reference_update_timestamp: u64,
    last_major_swap_timestamp: u64,
    volatility_reference: u32,
    tick_group_index_reference: i32,
    volatility_accumulator: u32,
    _reserved: [u8; 16],
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Oracle {
    whirlpool: Pubkey,
    trade_enable_timestamp: u64,
    constants: AdaptiveFeeConstants,
    variables: AdaptiveFeeVariables,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct AdaptiveFee {
    a_to_b: bool,
    tick_group_index: i32,
    static_fee_rate: u16,
    constants: AdaptiveFeeConstants,
    variables: AdaptiveFeeVariables,
    lower_bound: Option<(i32, u128)>,
    upper_bound: Option<(i32, u128)>,
}

impl AdaptiveFeeConstants {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            filter_period: reader.u16()?,
            decay_period: reader.u16()?,
            reduction_factor: reader.u16()?,
            adaptive_fee_control_factor: reader.u32()?,
            max_volatility_accumulator: reader.u32()?,
            tick_group_size: reader.u16()?,
            _major_swap_threshold_ticks: reader.u16()?,
            _reserved: reader.bytes()?,
        })
    }
}

impl AnchorAccount for Oracle {
    const NAME: &'static str = "Oracle";

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            whirlpool: Pubkey::read(reader)?,
            trade_enable_timestamp: reader.u64()?,
            constants: AdaptiveFeeConstants::read(reader)?,
            variables: AdaptiveFeeVariables::read(reader)?,
        })
    }
}

impl Oracle {
    pub fn decode_for_whirlpool(data: &[u8], whirlpool: &Pubkey) -> Result<Self, SwapperError> {
        let oracle = Self::decode(data)?;
        if oracle.whirlpool != *whirlpool {
            return Err(SwapperError::compute_quote_error("Unexpected oracle"));
        }
        Ok(oracle)
    }

    pub fn adaptive_fee(&self, static_fee_rate: u16, tick_current_index: i32, a_to_b: bool, timestamp: u64) -> Result<AdaptiveFee, SwapperError> {
        if self.trade_enable_timestamp > timestamp {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let group_size = i32::from(self.constants.tick_group_size);
        let tick_group_index = tick_current_index.div_euclid(group_size);
        let variables = self.variables.referenced(tick_group_index, timestamp, &self.constants)?;
        let core_range = self.constants.max_volatility_accumulator.saturating_sub(variables.volatility_reference).div_ceil(VOLATILITY_SCALE);
        let core_range = i32::try_from(core_range).map_err(SwapperError::compute_quote_error)?;
        let lower_index = variables.tick_group_index_reference - core_range;
        let upper_index = variables.tick_group_index_reference + core_range;
        let lower_tick = lower_index * group_size;
        let upper_tick = upper_index * group_size + group_size;
        Ok(AdaptiveFee {
            a_to_b,
            tick_group_index,
            static_fee_rate,
            constants: self.constants,
            variables,
            lower_bound: (lower_tick > MIN_TICK_INDEX).then(|| (lower_index, sqrt_price_from_tick_index(lower_tick))),
            upper_bound: (upper_tick < MAX_TICK_INDEX).then(|| (upper_index, sqrt_price_from_tick_index(upper_tick))),
        })
    }
}

impl AdaptiveFeeVariables {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            last_reference_update_timestamp: reader.u64()?,
            last_major_swap_timestamp: reader.u64()?,
            volatility_reference: reader.u32()?,
            tick_group_index_reference: reader.i32()?,
            volatility_accumulator: reader.u32()?,
            _reserved: reader.bytes()?,
        })
    }

    fn referenced(self, tick_group_index: i32, timestamp: u64, constants: &AdaptiveFeeConstants) -> Result<Self, SwapperError> {
        let latest_timestamp = self.last_reference_update_timestamp.max(self.last_major_swap_timestamp);
        let elapsed = timestamp.checked_sub(latest_timestamp).ok_or_else(|| SwapperError::compute_quote_error("Oracle is ahead of the chain clock"))?;
        let reset = Self {
            tick_group_index_reference: tick_group_index,
            volatility_reference: 0,
            last_reference_update_timestamp: timestamp,
            ..self
        };
        if timestamp - self.last_reference_update_timestamp > MAX_REFERENCE_AGE || elapsed >= u64::from(constants.decay_period) {
            return Ok(reset);
        }
        if elapsed < u64::from(constants.filter_period) {
            return Ok(self);
        }
        Ok(Self {
            volatility_reference: decayed_volatility(self.volatility_accumulator, constants.reduction_factor),
            ..reset
        })
    }

    fn accumulated(self, tick_group_index: i32, constants: &AdaptiveFeeConstants) -> Self {
        Self {
            volatility_accumulator: accumulated_volatility(self.volatility_reference, self.tick_group_index_reference, tick_group_index, constants.max_volatility_accumulator),
            ..self
        }
    }
}

impl AdaptiveFee {
    fn direction(&self) -> i32 {
        if self.a_to_b { -1 } else { 1 }
    }
}

impl SwapFee for AdaptiveFee {
    fn accumulate(self) -> Self {
        Self {
            variables: self.variables.accumulated(self.tick_group_index, &self.constants),
            ..self
        }
    }

    fn rate(&self) -> u32 {
        let adaptive_fee_rate = volatility_fee_rate(self.constants.adaptive_fee_control_factor, self.variables.volatility_accumulator, self.constants.tick_group_size);
        (u32::from(self.static_fee_rate) + adaptive_fee_rate).min(MAX_FEE_RATE)
    }

    fn bounded_target(&self, target: u128, liquidity: u128) -> (u128, bool) {
        if self.constants.adaptive_fee_control_factor == 0 || liquidity == 0 {
            return (target, true);
        }
        if let Some((lower_index, lower_sqrt_price)) = self.lower_bound
            && self.tick_group_index < lower_index
        {
            return (if self.a_to_b { target } else { target.min(lower_sqrt_price) }, true);
        }
        if let Some((upper_index, upper_sqrt_price)) = self.upper_bound
            && self.tick_group_index > upper_index
        {
            return (if self.a_to_b { target.max(upper_sqrt_price) } else { target }, true);
        }
        let group_size = i32::from(self.constants.tick_group_size);
        let boundary_tick = if self.a_to_b { self.tick_group_index * group_size } else { self.tick_group_index * group_size + group_size };
        let boundary_sqrt_price = sqrt_price_from_tick_index(boundary_tick.clamp(MIN_TICK_INDEX, MAX_TICK_INDEX));
        (if self.a_to_b { target.max(boundary_sqrt_price) } else { target.min(boundary_sqrt_price) }, false)
    }

    fn advance(self, skipped: bool, sqrt_price: u128, tick_sqrt_price: u128, tick_index: i32) -> Self {
        if !skipped {
            return Self {
                tick_group_index: self.tick_group_index + self.direction(),
                ..self
            };
        }
        let group_size = i32::from(self.constants.tick_group_size);
        let (tick_index, on_boundary) = if sqrt_price == tick_sqrt_price {
            (tick_index, tick_index % group_size == 0)
        } else {
            let tick_index = tick_index_from_sqrt_price(sqrt_price);
            (tick_index, tick_index % group_size == 0 && sqrt_price == sqrt_price_from_tick_index(tick_index))
        };
        let traversed_group = if on_boundary && !self.a_to_b { tick_index / group_size - 1 } else { tick_index.div_euclid(group_size) };
        let moved = if self.a_to_b { traversed_group < self.tick_group_index } else { traversed_group > self.tick_group_index };
        let (tick_group_index, variables) = if moved {
            (traversed_group, self.variables.accumulated(traversed_group, &self.constants))
        } else {
            (self.tick_group_index, self.variables)
        };
        Self {
            tick_group_index: tick_group_index + self.direction(),
            variables,
            ..self
        }
    }
}
