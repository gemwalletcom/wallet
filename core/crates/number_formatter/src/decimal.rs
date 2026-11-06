use std::borrow::Borrow;
use std::cmp::Ordering;
use std::f64::consts::LOG10_2;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::str::FromStr;

use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{Pow, Signed, ToPrimitive, Zero};

use crate::NumberFormatterError;

const DIVISION_PRECISION: usize = 100;
const EXPONENT_LEADING_ZEROS: i128 = 5;
const EXPONENT_TRAILING_ZEROS: i128 = 15;
const F64_KEPT_DIGITS: u64 = 25;
const F64_DIGIT_CHUNK: u64 = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Down,
    Up,
    Floor,
    HalfUp,
    HalfEven,
}

#[derive(Debug, Clone)]
pub struct Decimal {
    digits: BigInt,
    scale: i64,
}

impl Decimal {
    pub fn new(digits: impl Into<BigInt>, scale: i64) -> Self {
        Self { digits: digits.into(), scale }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn digits(&self) -> &BigInt {
        &self.digits
    }

    pub fn scale(&self) -> i64 {
        self.scale
    }

    pub fn into_parts(self) -> (BigInt, i64) {
        (self.digits, self.scale)
    }

    pub fn is_zero(&self) -> bool {
        self.digits.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.digits.is_negative()
    }

    pub fn abs(&self) -> Self {
        Self::new(self.digits.abs(), self.scale)
    }

    pub fn normalized(&self) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        let text = self.digits.magnitude().to_string();
        let zeros = text.len() - text.trim_end_matches('0').len();
        Self::new(&self.digits / ten_pow(zeros as u64), self.scale - zeros as i64)
    }

    pub fn is_integer(&self) -> bool {
        self.scale <= 0 || self.is_zero() || (self.scale < digit_count(self.digits.magnitude()) as i64 && (&self.digits % ten_pow(self.scale.unsigned_abs())).is_zero())
    }

    pub fn with_scale(&self, scale: i64) -> Self {
        if self.is_zero() {
            return Self::new(0, scale);
        }
        match scale.cmp(&self.scale) {
            Ordering::Greater => Self::new(&self.digits * ten_pow(scale.abs_diff(self.scale)), scale),
            Ordering::Less => Self::new(&self.digits / ten_pow(scale.abs_diff(self.scale)), scale),
            Ordering::Equal => self.clone(),
        }
    }

    pub fn with_scale_round(&self, scale: i64, rounding: Rounding) -> Self {
        if scale >= self.scale || self.is_zero() {
            return self.with_scale(scale);
        }
        let shift = scale.abs_diff(self.scale);
        if shift > digit_count(self.digits.magnitude()) as u64 {
            let away_from_zero = match rounding {
                Rounding::Down | Rounding::HalfUp | Rounding::HalfEven => false,
                Rounding::Up => true,
                Rounding::Floor => self.is_negative(),
            };
            return Self::new(if away_from_zero { self.digits.signum() } else { BigInt::zero() }, scale);
        }
        let divisor = ten_pow(shift);
        let quotient = &self.digits / &divisor;
        let remainder = &self.digits % &divisor;
        if remainder.is_zero() {
            return Self::new(quotient, scale);
        }
        let away_from_zero = match rounding {
            Rounding::Down => false,
            Rounding::Up => true,
            Rounding::Floor => self.is_negative(),
            Rounding::HalfUp | Rounding::HalfEven => match (remainder.magnitude() * 2u32).cmp(divisor.magnitude()) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => rounding == Rounding::HalfUp || quotient.magnitude().bit(0),
            },
        };
        if away_from_zero { Self::new(quotient + self.digits.signum(), scale) } else { Self::new(quotient, scale) }
    }

    pub fn with_precision_round(&self, precision: u64, rounding: Rounding) -> Self {
        self.with_scale_round(self.scale + precision as i64 - digit_count(self.digits.magnitude()) as i64, rounding)
    }

    pub fn to_plain_string(&self) -> String {
        let text = plain(&self.digits.magnitude().to_string(), self.scale);
        if self.is_negative() { format!("-{text}") } else { text }
    }

    pub fn to_u64(&self) -> Option<u64> {
        if self.is_negative() {
            return None;
        }
        self.with_scale(0).digits.to_u64()
    }

    pub fn to_i64(&self) -> Option<i64> {
        self.with_scale(0).digits.to_i64()
    }

    pub fn to_f64(&self) -> Option<f64> {
        if self.is_zero() {
            return Some(0.0);
        }
        if self.scale == 0 {
            return self.digits.to_f64();
        }
        let estimated_digits = ((self.digits.bits() + 1) as f64 * LOG10_2).floor() as u64;
        let trimmed = estimated_digits.saturating_sub(F64_KEPT_DIGITS) / F64_DIGIT_CHUNK * F64_DIGIT_CHUNK;
        let magnitude = self.digits.magnitude() / ten_pow(trimmed).magnitude();
        let scale = self.scale - trimmed as i64;
        let value = match i32::try_from(scale).ok().and_then(i32::checked_neg) {
            Some(power) if power >= 0 => magnitude.to_f64()? * 10f64.powi(power),
            Some(exponent) => format!("{magnitude}e{exponent}").parse().ok()?,
            None if scale > 0 => 0.0,
            None => f64::INFINITY,
        };
        Some(if self.is_negative() { -value } else { value })
    }

    fn divide(&self, divisor: &Decimal) -> Decimal {
        assert!(!divisor.is_zero(), "division by zero");
        if self.is_zero() || *divisor == Decimal::from(1) {
            return self.clone();
        }
        let scale = self.scale - divisor.scale;
        if self.digits == divisor.digits {
            return Decimal::new(1, scale);
        }
        let (quotient, scale) = divide_magnitude(self.digits.magnitude().clone(), divisor.digits.magnitude(), scale);
        let sign = if self.is_negative() == divisor.is_negative() { Sign::Plus } else { Sign::Minus };
        Decimal::new(BigInt::from_biguint(sign, quotient), scale)
    }

    fn aligned(&self, other: &Decimal) -> (BigInt, BigInt, i64) {
        let scale = self.scale.max(other.scale);
        (self.with_scale(scale).digits, other.with_scale(scale).digits, scale)
    }
}

fn divide_magnitude(mut numerator: BigUint, denominator: &BigUint, mut scale: i64) -> (BigUint, i64) {
    while &numerator < denominator {
        numerator *= 10u32;
        scale += 1;
    }
    let mut quotient = &numerator / denominator;
    let mut remainder = &numerator % denominator;
    if remainder.is_zero() {
        return (quotient, scale);
    }
    let mut precision = digit_count(&quotient);
    remainder *= 10u32;
    while !remainder.is_zero() && precision < DIVISION_PRECISION {
        quotient = quotient * 10u32 + &remainder / denominator;
        remainder = (&remainder % denominator) * 10u32;
        precision += 1;
        scale += 1;
    }
    if !remainder.is_zero() && &remainder / denominator >= BigUint::from(5u32) {
        quotient += 1u32;
    }
    (quotient, scale)
}

pub fn f64_parts(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & 0x000F_FFFF_FFFF_FFFF;
    if exponent == 0 { (fraction, -1074) } else { (fraction | 1 << 52, exponent - 1075) }
}

fn ten_pow(exponent: u64) -> BigInt {
    BigInt::from(Pow::pow(BigUint::from(10u32), exponent))
}

fn digit_count(value: &BigUint) -> usize {
    value.to_string().len()
}

fn plain(digits: &str, scale: i64) -> String {
    let length = digits.len() as i64;
    if scale <= 0 {
        format!("{digits}{}", "0".repeat(scale.unsigned_abs() as usize))
    } else if scale < length {
        let (integer, fraction) = digits.split_at((length - scale) as usize);
        format!("{integer}.{fraction}")
    } else {
        format!("0.{}{digits}", "0".repeat((scale - length) as usize))
    }
}

fn exponential(digits: &str, exponent: i128) -> String {
    let (first, rest) = digits.split_at(1);
    if rest.is_empty() { format!("{first}E{exponent:+}") } else { format!("{first}.{rest}E{exponent:+}") }
}

impl fmt::Display for Decimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.digits.magnitude().to_string();
        let scale = if self.is_zero() { 0 } else { self.scale };
        let exponent_scale = i128::from(scale);
        let length = digits.len() as i128;
        let text = if exponent_scale - length > EXPONENT_LEADING_ZEROS {
            exponential(&digits, length - exponent_scale - 1)
        } else if -exponent_scale > EXPONENT_TRAILING_ZEROS {
            format!("{digits}e{:+}", -exponent_scale)
        } else {
            plain(&digits, scale)
        };
        formatter.pad_integral(!self.is_negative(), "", &text)
    }
}

impl FromStr for Decimal {
    type Err = NumberFormatterError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid = || NumberFormatterError::InvalidNumber(value.to_string());
        let (base, exponent) = match value.find(['e', 'E']) {
            Some(index) => (&value[..index], value[index + 1..].parse::<i64>().map_err(|_| invalid())?),
            None => (value, 0),
        };
        let (integer, fraction) = base.split_once('.').unwrap_or((base, ""));
        let (sign, integer) = match integer.as_bytes().first() {
            Some(b'-') => (Sign::Minus, &integer[1..]),
            Some(b'+') => (Sign::Plus, &integer[1..]),
            _ => (Sign::Plus, integer),
        };
        let digits = [integer, fraction].concat();
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid());
        }
        let magnitude = BigUint::parse_bytes(digits.as_bytes(), 10).ok_or_else(invalid)?;
        let scale = (fraction.len() as i64).checked_sub(exponent).ok_or_else(invalid)?;
        Ok(Self::new(BigInt::from_biguint(sign, magnitude), scale))
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Self::new(value, 0)
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.digits.sign().cmp(&other.digits.sign()) {
            Ordering::Equal if !self.is_zero() => {
                let magnitude = compare_magnitude(self, other);
                if self.is_negative() { magnitude.reverse() } else { magnitude }
            }
            ordering => ordering,
        }
    }
}

fn compare_magnitude(left: &Decimal, right: &Decimal) -> Ordering {
    let left_exponent = digit_count(left.digits.magnitude()) as i128 - i128::from(left.scale);
    let right_exponent = digit_count(right.digits.magnitude()) as i128 - i128::from(right.scale);
    left_exponent.cmp(&right_exponent).then_with(|| {
        let (left, right, _) = left.abs().aligned(&right.abs());
        left.cmp(&right)
    })
}

impl<T: Borrow<Decimal>> Add<T> for &Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Decimal {
        let other = other.borrow();
        if other.is_zero() {
            return self.clone();
        }
        if self.is_zero() {
            return other.clone();
        }
        let (left, right, scale) = self.aligned(other);
        Decimal::new(left + right, scale)
    }
}

impl<T: Borrow<Decimal>> Add<T> for Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Decimal {
        &self + other
    }
}

impl<T: Borrow<Decimal>> Sub<T> for &Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Decimal {
        let other = other.borrow();
        if other.is_zero() {
            return self.clone();
        }
        if self.is_zero() {
            return Decimal::new(-&other.digits, other.scale);
        }
        let (left, right, scale) = self.aligned(other);
        Decimal::new(left - right, scale)
    }
}

impl<T: Borrow<Decimal>> Sub<T> for Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Decimal {
        &self - other
    }
}

impl<T: Borrow<Decimal>> Mul<T> for &Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Decimal {
        let other = other.borrow();
        Decimal::new(&self.digits * &other.digits, self.scale + other.scale)
    }
}

impl<T: Borrow<Decimal>> Mul<T> for Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Decimal {
        &self * other
    }
}

impl<T: Borrow<Decimal>> Div<T> for &Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Decimal {
        self.divide(other.borrow())
    }
}

impl<T: Borrow<Decimal>> Div<T> for Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Decimal {
        self.divide(other.borrow())
    }
}

impl<T: Borrow<Decimal>> AddAssign<T> for Decimal {
    fn add_assign(&mut self, other: T) {
        *self = &*self + other;
    }
}

impl<T: Borrow<Decimal>> SubAssign<T> for Decimal {
    fn sub_assign(&mut self, other: T) {
        *self = &*self - other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(value: &str) -> (String, i64) {
        let decimal: Decimal = value.parse().unwrap();
        (decimal.digits().to_string(), decimal.scale())
    }

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    #[test]
    fn test_from_str() {
        assert_eq!(parts("0"), ("0".to_string(), 0));
        assert_eq!(parts("-0"), ("0".to_string(), 0));
        assert_eq!(parts("+5"), ("5".to_string(), 0));
        assert_eq!(parts("5."), ("5".to_string(), 0));
        assert_eq!(parts(".5"), ("5".to_string(), 1));
        assert_eq!(parts("-.5"), ("-5".to_string(), 1));
        assert_eq!(parts("1.50"), ("150".to_string(), 2));
        assert_eq!(parts("001.2300"), ("12300".to_string(), 4));
        assert_eq!(parts("1e3"), ("1".to_string(), -3));
        assert_eq!(parts("1E-3"), ("1".to_string(), 3));
        assert_eq!(parts("1.23e+2"), ("123".to_string(), 0));
        assert_eq!(parts("-1.5e-10"), ("-15".to_string(), 11));
        assert_eq!(parts("12345678901234567890.123456789"), ("12345678901234567890123456789".to_string(), 9));

        for invalid in [
            "",
            ".",
            "-",
            "+",
            "-.",
            "e5",
            "1e",
            "1e+",
            "1e5.5",
            "1.2.3",
            "1_000",
            " 1",
            "1 ",
            "abc",
            "1.-5",
            ".-5",
            "--1",
            "+-1",
            "0x10",
            "NaN",
            "inf",
            "١",
            "1e9223372036854775808",
        ] {
            assert_eq!(invalid.parse::<Decimal>(), Err(NumberFormatterError::InvalidNumber(invalid.to_string())), "{invalid}");
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(Decimal::new(5, 0).to_string(), "5");
        assert_eq!(Decimal::new(100, 2).to_string(), "1.00");
        assert_eq!(Decimal::new(-123, 2).to_string(), "-1.23");
        assert_eq!(Decimal::new(0, 5).to_string(), "0");
        assert_eq!(Decimal::new(1, 6).to_string(), "0.000001");
        assert_eq!(Decimal::new(1, 7).to_string(), "1E-7");
        assert_eq!(Decimal::new(15, 7).to_string(), "0.0000015");
        assert_eq!(Decimal::new(15, 8).to_string(), "1.5E-7");
        assert_eq!(Decimal::new(-15, 8).to_string(), "-1.5E-7");
        assert_eq!(Decimal::new(1, 18).to_string(), "1E-18");
        assert_eq!(Decimal::new(123, -15).to_string(), "123000000000000000");
        assert_eq!(Decimal::new(123, -16).to_string(), "123e+16");
        assert_eq!(Decimal::new(-123, -16).to_string(), "-123e+16");
    }

    #[test]
    fn test_to_plain_string() {
        assert_eq!(Decimal::new(0, 3).to_plain_string(), "0.000");
        assert_eq!(Decimal::new(1, 18).to_plain_string(), "0.000000000000000001");
        assert_eq!(Decimal::new(123, -16).to_plain_string(), "1230000000000000000");
        assert_eq!(Decimal::new(-5, 1).to_plain_string(), "-0.5");
        assert_eq!(Decimal::new(12345, 2).to_plain_string(), "123.45");
    }

    #[test]
    fn test_normalized() {
        assert_eq!(Decimal::new(1200, 2).normalized().into_parts(), (BigInt::from(12), 0));
        assert_eq!(Decimal::new(0, 5).normalized().into_parts(), (BigInt::from(0), 0));
        assert_eq!(Decimal::new(-1500, -1).normalized().into_parts(), (BigInt::from(-15), -3));
        assert_eq!(Decimal::new(7, 3).normalized().into_parts(), (BigInt::from(7), 3));
    }

    #[test]
    fn test_with_scale() {
        assert_eq!(Decimal::new(12345, 2).with_scale(1).into_parts(), (BigInt::from(1234), 1));
        assert_eq!(Decimal::new(12345, 2).with_scale(4).into_parts(), (BigInt::from(1_234_500), 4));
        assert_eq!(Decimal::new(-12345, 2).with_scale(0).into_parts(), (BigInt::from(-123), 0));
        assert_eq!(Decimal::new(0, 2).with_scale(-3).into_parts(), (BigInt::from(0), -3));
    }

    #[test]
    fn test_with_scale_round() {
        let rounded = |value: &str, rounding: Rounding| decimal(value).with_scale_round(0, rounding).to_plain_string();
        let values = ["2.5", "3.5", "-2.5", "-3.5", "2.51", "-2.51", "2.49", "-2.49", "2.0", "0.4"];
        let expected = [
            (Rounding::Down, ["2", "3", "-2", "-3", "2", "-2", "2", "-2", "2", "0"]),
            (Rounding::Up, ["3", "4", "-3", "-4", "3", "-3", "3", "-3", "2", "1"]),
            (Rounding::Floor, ["2", "3", "-3", "-4", "2", "-3", "2", "-3", "2", "0"]),
            (Rounding::HalfUp, ["3", "4", "-3", "-4", "3", "-3", "2", "-2", "2", "0"]),
            (Rounding::HalfEven, ["2", "4", "-2", "-4", "3", "-3", "2", "-2", "2", "0"]),
        ];
        for (rounding, results) in expected {
            assert_eq!(values.map(|value| rounded(value, rounding)), results.map(String::from), "{rounding:?}");
        }
        assert_eq!(decimal("1.005").with_scale_round(2, Rounding::HalfEven).to_plain_string(), "1.00");
        assert_eq!(decimal("1.015").with_scale_round(2, Rounding::HalfEven).to_plain_string(), "1.02");
        assert_eq!(decimal("1.5").with_scale_round(3, Rounding::Down).to_plain_string(), "1.500");
        assert_eq!(decimal("1250").with_scale_round(-2, Rounding::HalfEven).to_plain_string(), "1200");
        assert_eq!(decimal("0").with_scale_round(-2, Rounding::Up).into_parts(), (BigInt::from(0), -2));
    }

    #[test]
    fn test_extreme_exponents() {
        let tiny = decimal("1e-999999999");
        assert_eq!(tiny.with_scale_round(8, Rounding::Down).into_parts(), (BigInt::from(0), 8));
        assert_eq!(tiny.with_scale_round(8, Rounding::HalfEven).into_parts(), (BigInt::from(0), 8));
        assert_eq!(tiny.with_scale_round(8, Rounding::Up).into_parts(), (BigInt::from(1), 8));
        assert_eq!(decimal("-1e-999999999").with_scale_round(8, Rounding::Floor).into_parts(), (BigInt::from(-1), 8));
        assert_eq!(decimal("0e-999999999").with_scale(0).into_parts(), (BigInt::from(0), 0));
        assert_eq!(decimal("0e-999999999") + decimal("5"), decimal("5"));
        assert_eq!(decimal("5") - decimal("0e999999999"), decimal("5"));
        assert_eq!(decimal("0e-999999999") - decimal("5"), decimal("-5"));
        assert!(!tiny.is_integer());
        assert!(decimal("0e-999999999").is_integer());
        assert!(decimal("10e9223372036854775807") > decimal("1"));
        assert!(decimal("-10e9223372036854775807") < decimal("-1"));
        assert_eq!(Decimal::new(1, i64::MIN).to_string(), "1e+9223372036854775808");
        assert_eq!(Decimal::new(1, i64::MAX).to_string(), "1E-9223372036854775807");
    }

    #[test]
    fn test_with_precision_round() {
        assert_eq!(decimal("0.012345").with_precision_round(4, Rounding::Down).to_plain_string(), "0.01234");
        assert_eq!(decimal("123456").with_precision_round(2, Rounding::Down).to_plain_string(), "120000");
        assert_eq!(decimal("9.999").with_precision_round(2, Rounding::HalfEven).normalized().to_plain_string(), "10");
        assert_eq!(decimal("-0.00012399").with_precision_round(3, Rounding::Down).to_plain_string(), "-0.000123");
        assert_eq!(decimal("1.2").with_precision_round(4, Rounding::Down).to_plain_string(), "1.200");
    }

    #[test]
    fn test_is_integer() {
        assert!(decimal("5.000").is_integer());
        assert!(!decimal("5.001").is_integer());
        assert!(decimal("1e3").is_integer());
        assert!(decimal("0.0").is_integer());
        assert!(!decimal("-0.5").is_integer());
    }

    #[test]
    fn test_cmp() {
        assert_eq!(decimal("1.0"), decimal("1"));
        assert_eq!(decimal("1e2"), decimal("100"));
        assert_eq!(decimal("0"), decimal("-0.00"));
        assert!(decimal("1.10") > decimal("1.09"));
        assert!(decimal("-1") < decimal("0"));
        assert!(decimal("-1.5") < decimal("-1.4"));
        assert!(decimal("0.001") < decimal("1e-2"));
        assert!(decimal("99999") < decimal("1e5"));
        assert!(decimal("-1e5") < decimal("-99999"));
        assert_eq!(decimal("2.5").max(decimal("2.50")), decimal("2.5"));
        assert_eq!(decimal("3").min(decimal("-3")), decimal("-3"));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!((decimal("1.5") + decimal("2.25")).into_parts(), (BigInt::from(375), 2));
        assert_eq!((&decimal("1") - &decimal("2.5")).into_parts(), (BigInt::from(-15), 1));
        assert_eq!((decimal("1.5") * &decimal("-2")).into_parts(), (BigInt::from(-30), 1));
        assert_eq!((&decimal("0.1") * decimal("0.1")).into_parts(), (BigInt::from(1), 2));

        let mut total = decimal("10");
        total += decimal("0.5");
        total += &decimal("1e-3");
        total -= &decimal("20");
        assert_eq!(total.into_parts(), (BigInt::from(-9499), 3));
    }

    #[test]
    fn test_div() {
        let one_third = format!("0.{}", "3".repeat(100));
        let two_thirds = format!("0.{}7", "6".repeat(99));
        assert_eq!((decimal("1") / decimal("3")).to_plain_string(), one_third);
        assert_eq!((decimal("2") / decimal("3")).to_plain_string(), two_thirds);
        assert_eq!((decimal("-2") / decimal("3")).to_plain_string(), format!("-{two_thirds}"));
        assert_eq!((decimal("2") / decimal("-3")).to_plain_string(), format!("-{two_thirds}"));
        assert_eq!((decimal("-2") / decimal("-3")).to_plain_string(), two_thirds);
        assert_eq!((decimal("1") / decimal("4")).to_plain_string(), "0.25");
        assert_eq!((decimal("-1") / decimal("8")).to_plain_string(), "-0.125");
        assert_eq!((decimal("10") / decimal("4")).to_plain_string(), "2.5");
        assert_eq!((decimal("5.00") / decimal("1.0")).to_plain_string(), "5.00");
        assert_eq!((decimal("7.5") / decimal("7.5")).into_parts(), (BigInt::from(1), 0));
        assert_eq!((decimal("0.00") / decimal("7")).to_plain_string(), "0.00");
        assert_eq!((decimal("100") / decimal("0.25")).to_plain_string(), "400");
        assert_eq!((decimal("1e30") / decimal("3")).to_plain_string(), format!("{}.{}", "3".repeat(30), "3".repeat(70)));
        assert_eq!((decimal("1") / decimal("7")).to_plain_string(), format!("0.{}9", &"142857".repeat(17)[..99]));
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_div_by_zero() {
        let _ = decimal("1") / decimal("0.000");
    }

    #[test]
    fn test_to_f64() {
        assert_eq!(decimal("0").to_f64(), Some(0.0));
        assert_eq!(decimal("0.1").to_f64(), Some(0.1));
        assert_eq!(decimal("-2.5").to_f64(), Some(-2.5));
        assert_eq!(decimal("1e-18").to_f64(), Some(1e-18));
        assert_eq!(decimal("123456789").to_f64(), Some(123_456_789.0));
        assert_eq!(decimal("1.5e20").to_f64(), Some(1.5e20));
        assert_eq!(decimal("0.30000000000000004").to_f64(), Some(0.1 + 0.2));
        assert_eq!(decimal("123456789012345678901234567890.123456789012345678901").to_f64(), Some(123_456_789_012_345_680_000_000_000_000.0));
        assert_eq!(decimal("1e400").to_f64(), Some(f64::INFINITY));
        assert_eq!(decimal("-1e400").to_f64(), Some(f64::NEG_INFINITY));
        assert_eq!(decimal("1e-400").to_f64(), Some(0.0));
        assert_eq!(Decimal::new(1, 3_000_000_000).to_f64(), Some(0.0));
        assert_eq!(Decimal::new(1, -3_000_000_000).to_f64(), Some(f64::INFINITY));
    }

    #[test]
    fn test_to_integers() {
        assert_eq!(decimal("12.9").to_u64(), Some(12));
        assert_eq!(decimal("-0.5").to_u64(), None);
        assert_eq!(decimal("18446744073709551615").to_u64(), Some(u64::MAX));
        assert_eq!(decimal("18446744073709551616").to_u64(), None);
        assert_eq!(decimal("-12.9").to_i64(), Some(-12));
        assert_eq!(decimal("9.9e18").to_i64(), None);
    }
}
