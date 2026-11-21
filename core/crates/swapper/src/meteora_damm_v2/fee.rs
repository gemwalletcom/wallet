use alloy_primitives::U256;
use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

use crate::clmm::Q64_RESOLUTION;

const MAX_BASIS_POINT: u64 = 10_000;
const ONE_Q64: u128 = 1 << Q64_RESOLUTION;
const VARIABLE_FEE_SCALE: u128 = 100_000_000_000;
const TIME_SCHEDULER_LINEAR: u8 = 0;
const TIME_SCHEDULER_EXPONENTIAL: u8 = 1;
const RATE_LIMITER: u8 = 2;
const MARKET_CAP_SCHEDULER_LINEAR: u8 = 3;
const MARKET_CAP_SCHEDULER_EXPONENTIAL: u8 = 4;
const BASE_FEE_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TimeScheduler {
    cliff_fee_numerator: u64,
    mode: u8,
    number_of_period: u16,
    period_frequency: u64,
    reduction_factor: u64,
}

impl BorshDeserialize for TimeScheduler {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            cliff_fee_numerator: reader.read()?,
            mode: reader.read()?,
            number_of_period: reader.skip(5)?.read()?,
            period_frequency: reader.read()?,
            reduction_factor: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MarketCapScheduler {
    cliff_fee_numerator: u64,
    mode: u8,
    number_of_period: u16,
    sqrt_price_step_bps: u32,
    scheduler_expiration_duration: u32,
    reduction_factor: u64,
}

impl BorshDeserialize for MarketCapScheduler {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            cliff_fee_numerator: reader.read()?,
            mode: reader.read()?,
            number_of_period: reader.skip(5)?.read()?,
            sqrt_price_step_bps: reader.read()?,
            scheduler_expiration_duration: reader.read()?,
            reduction_factor: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RateLimiter {
    cliff_fee_numerator: u64,
    fee_increment_bps: u16,
    max_limiter_duration: u32,
    max_fee_bps: u32,
    reference_amount: u64,
}

impl BorshDeserialize for RateLimiter {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            cliff_fee_numerator: reader.read()?,
            fee_increment_bps: reader.skip(6)?.read()?,
            max_limiter_duration: reader.read()?,
            max_fee_bps: reader.read()?,
            reference_amount: reader.read()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum BaseFee {
    TimeScheduler(TimeScheduler),
    MarketCapScheduler(MarketCapScheduler),
    RateLimiter(RateLimiter),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DynamicFee {
    initialized: u8,
    variable_fee_control: u32,
    bin_step: u16,
    volatility_accumulator: u128,
}

impl BorshDeserialize for DynamicFee {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            initialized: reader.read()?,
            variable_fee_control: reader.skip(11)?.read()?,
            bin_step: reader.read()?,
            volatility_accumulator: reader.skip(46)?.read()?,
        };
        reader.skip(16)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FeePoint {
    pub current: u64,
    pub activation: u64,
    pub a_to_b: bool,
    pub init_sqrt_price: u128,
    pub sqrt_price: u128,
}

impl BorshDeserialize for BaseFee {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let data: [u8; BASE_FEE_LEN] = reader.bytes()?;
        match data[8] {
            TIME_SCHEDULER_LINEAR | TIME_SCHEDULER_EXPONENTIAL => Ok(Self::TimeScheduler(TimeScheduler::try_from_slice(&data)?)),
            RATE_LIMITER => Ok(Self::RateLimiter(RateLimiter::try_from_slice(&data)?)),
            MARKET_CAP_SCHEDULER_LINEAR | MARKET_CAP_SCHEDULER_EXPONENTIAL => Ok(Self::MarketCapScheduler(MarketCapScheduler::try_from_slice(&data)?)),
            mode => Err(DecodeError::UnexpectedTag(mode)),
        }
    }
}

impl BaseFee {
    pub fn numerator(&self, point: &FeePoint) -> Option<u64> {
        match self {
            Self::TimeScheduler(scheduler) => scheduler.numerator(point),
            Self::MarketCapScheduler(scheduler) => scheduler.numerator(point),
            Self::RateLimiter(limiter) => limiter.numerator(point),
        }
    }
}

impl TimeScheduler {
    fn numerator(&self, point: &FeePoint) -> Option<u64> {
        if self.period_frequency == 0 {
            return Some(self.cliff_fee_numerator);
        }
        let period = point.current.checked_sub(point.activation).map_or(u64::from(self.number_of_period), |elapsed| elapsed / self.period_frequency);
        scheduled_fee(self.mode == TIME_SCHEDULER_EXPONENTIAL, self.cliff_fee_numerator, self.reduction_factor, self.number_of_period, period)
    }
}

impl MarketCapScheduler {
    fn numerator(&self, point: &FeePoint) -> Option<u64> {
        let expiration = point.activation.checked_add(u64::from(self.scheduler_expiration_duration))?;
        let period = if point.current > expiration || point.current < point.activation {
            u64::from(self.number_of_period)
        } else if point.sqrt_price <= point.init_sqrt_price {
            0
        } else {
            let growth_bps = (U256::from(point.sqrt_price - point.init_sqrt_price) * U256::from(MAX_BASIS_POINT)).checked_div(U256::from(point.init_sqrt_price))?;
            u64::try_from(growth_bps.checked_div(U256::from(self.sqrt_price_step_bps))?).unwrap_or(u64::MAX)
        };
        scheduled_fee(self.mode == MARKET_CAP_SCHEDULER_EXPONENTIAL, self.cliff_fee_numerator, self.reduction_factor, self.number_of_period, period)
    }
}

impl RateLimiter {
    fn numerator(&self, point: &FeePoint) -> Option<u64> {
        let inactive = self.reference_amount == 0 && self.max_limiter_duration == 0 && self.max_fee_bps == 0 && self.fee_increment_bps == 0;
        let last_point = u128::from(point.activation) + u128::from(self.max_limiter_duration);
        let applied = !inactive && !point.a_to_b && point.current >= point.activation && u128::from(point.current) <= last_point;
        (!applied).then_some(self.cliff_fee_numerator)
    }
}

impl DynamicFee {
    pub fn variable_fee(&self) -> Option<u128> {
        if self.initialized == 0 {
            return Some(0);
        }
        let squared = self.volatility_accumulator.checked_mul(u128::from(self.bin_step))?.checked_pow(2)?;
        Some(squared.checked_mul(u128::from(self.variable_fee_control))?.checked_add(VARIABLE_FEE_SCALE - 1)? / VARIABLE_FEE_SCALE)
    }
}

fn scheduled_fee(exponential: bool, cliff_fee_numerator: u64, reduction_factor: u64, number_of_period: u16, period: u64) -> Option<u64> {
    let period = period.min(u64::from(number_of_period));
    if exponential {
        exponential_fee(cliff_fee_numerator, reduction_factor, u16::try_from(period).unwrap_or_default())
    } else {
        cliff_fee_numerator.checked_sub(reduction_factor.checked_mul(period)?)
    }
}

fn exponential_fee(cliff_fee_numerator: u64, reduction_factor: u64, period: u16) -> Option<u64> {
    if reduction_factor == 0 {
        return Some(cliff_fee_numerator);
    }
    let base = ONE_Q64.checked_sub((u128::from(reduction_factor) << Q64_RESOLUTION) / u128::from(MAX_BASIS_POINT))?;
    let (factor, _) = (0..u16::BITS).fold((ONE_Q64, base), |(factor, square), bit| {
        let factor = if period & (1 << bit) != 0 { (factor * square) >> Q64_RESOLUTION } else { factor };
        (factor, (square * square) >> Q64_RESOLUTION)
    });
    (factor > 0).then(|| u64::try_from((factor * u128::from(cliff_fee_numerator)) >> Q64_RESOLUTION).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIFF: u64 = 500_000_000;

    fn time_scheduler(mode: u8, period_frequency: u64) -> TimeScheduler {
        TimeScheduler {
            cliff_fee_numerator: CLIFF,
            mode,
            number_of_period: 100,
            period_frequency,
            reduction_factor: 100,
        }
    }

    fn point(current: u64, sqrt_price: u128) -> FeePoint {
        FeePoint {
            current,
            activation: 1_000,
            a_to_b: false,
            init_sqrt_price: 1 << 64,
            sqrt_price,
        }
    }

    #[test]
    fn test_time_scheduler() {
        let linear = time_scheduler(TIME_SCHEDULER_LINEAR, 10);
        let exponential = time_scheduler(TIME_SCHEDULER_EXPONENTIAL, 10);

        assert_eq!(linear.numerator(&point(1_000, 0)), Some(CLIFF));
        assert_eq!(linear.numerator(&point(1_055, 0)), Some(CLIFF - 500));
        assert_eq!(linear.numerator(&point(100_000, 0)), Some(CLIFF - 10_000));
        assert_eq!(exponential.numerator(&point(1_010, 0)), Some(495_000_000));
        assert_eq!(exponential.numerator(&point(1_020, 0)), Some(490_050_000));
        assert_eq!(exponential.numerator(&point(100_000, 0)), Some(183_016_170));
        assert_eq!(time_scheduler(TIME_SCHEDULER_LINEAR, 0).numerator(&point(1_055, 0)), Some(CLIFF));
    }

    #[test]
    fn test_market_cap_scheduler() {
        let scheduler = MarketCapScheduler {
            cliff_fee_numerator: CLIFF,
            mode: MARKET_CAP_SCHEDULER_LINEAR,
            number_of_period: 100,
            sqrt_price_step_bps: 50,
            scheduler_expiration_duration: 86_400,
            reduction_factor: 1_000_000,
        };

        assert_eq!(scheduler.numerator(&point(2_000, 1 << 63)), Some(CLIFF));
        assert_eq!(scheduler.numerator(&point(2_000, (1 << 64) + (1 << 60))), Some(CLIFF - 12_000_000));
        assert_eq!(scheduler.numerator(&point(2_000, 1 << 70)), Some(CLIFF - 100_000_000));
        assert_eq!(scheduler.numerator(&point(100_000, 1 << 63)), Some(CLIFF - 100_000_000));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter {
            cliff_fee_numerator: CLIFF,
            fee_increment_bps: 10,
            max_limiter_duration: 600,
            max_fee_bps: 5_000,
            reference_amount: 1_000_000_000,
        };

        assert_eq!(limiter.numerator(&point(1_500, 0)), None);
        assert_eq!(limiter.numerator(&FeePoint { a_to_b: true, ..point(1_500, 0) }), Some(CLIFF));
        assert_eq!(limiter.numerator(&point(1_601, 0)), Some(CLIFF));
    }

    #[test]
    fn test_variable_fee() {
        let fee = DynamicFee {
            initialized: 1,
            variable_fee_control: 956,
            bin_step: 1,
            volatility_accumulator: 60_000,
        };

        assert_eq!(fee.variable_fee(), Some(35));
        assert_eq!(DynamicFee { initialized: 0, ..fee }.variable_fee(), Some(0));
        assert_eq!(DynamicFee { volatility_accumulator: u128::MAX, ..fee }.variable_fee(), None);
    }
}
