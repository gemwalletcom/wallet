use std::iter;

const HIDDEN_BIT: u64 = 0x0010_0000_0000_0000;
const SIGNIFICAND_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;
const SIGNIFICAND_BITS: i32 = 52;
const EXPONENT_BIAS: i32 = 0x3FF;
const DIY_SIGNIFICAND_BITS: i32 = 64;
const MIN_CACHED_POWER: i32 = -348;
const CACHED_POWER_STEP: i32 = 8;
const SCALED_DISTANCE_DIGITS: usize = 9;
const U64_DIGITS: usize = 20;
const LOG10_2: f64 = 0.30102999566398114;
const MAX_INTEGER_DIGITS: i32 = 21;
const MIN_FRACTION_EXPONENT: i32 = -6;
const FRACTION_PREFIX: &str = "0.00000";
const POWERS_OF_TEN: [u64; 10] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000];

#[rustfmt::skip]
const CACHED_POWERS_SIGNIFICAND: [u64; 87] = [
    0xfa8fd5a0081c0288, 0xbaaee17fa23ebf76, 0x8b16fb203055ac76, 0xcf42894a5dce35ea, 0x9a6bb0aa55653b2d, 0xe61acf033d1a45df,
    0xab70fe17c79ac6ca, 0xff77b1fcbebcdc4f, 0xbe5691ef416bd60c, 0x8dd01fad907ffc3c, 0xd3515c2831559a83, 0x9d71ac8fada6c9b5,
    0xea9c227723ee8bcb, 0xaecc49914078536d, 0x823c12795db6ce57, 0xc21094364dfb5637, 0x9096ea6f3848984f, 0xd77485cb25823ac7,
    0xa086cfcd97bf97f4, 0xef340a98172aace5, 0xb23867fb2a35b28e, 0x84c8d4dfd2c63f3b, 0xc5dd44271ad3cdba, 0x936b9fcebb25c996,
    0xdbac6c247d62a584, 0xa3ab66580d5fdaf6, 0xf3e2f893dec3f126, 0xb5b5ada8aaff80b8, 0x87625f056c7c4a8b, 0xc9bcff6034c13053,
    0x964e858c91ba2655, 0xdff9772470297ebd, 0xa6dfbd9fb8e5b88f, 0xf8a95fcf88747d94, 0xb94470938fa89bcf, 0x8a08f0f8bf0f156b,
    0xcdb02555653131b6, 0x993fe2c6d07b7fac, 0xe45c10c42a2b3b06, 0xaa242499697392d3, 0xfd87b5f28300ca0e, 0xbce5086492111aeb,
    0x8cbccc096f5088cc, 0xd1b71758e219652c, 0x9c40000000000000, 0xe8d4a51000000000, 0xad78ebc5ac620000, 0x813f3978f8940984,
    0xc097ce7bc90715b3, 0x8f7e32ce7bea5c70, 0xd5d238a4abe98068, 0x9f4f2726179a2245, 0xed63a231d4c4fb27, 0xb0de65388cc8ada8,
    0x83c7088e1aab65db, 0xc45d1df942711d9a, 0x924d692ca61be758, 0xda01ee641a708dea, 0xa26da3999aef774a, 0xf209787bb47d6b85,
    0xb454e4a179dd1877, 0x865b86925b9bc5c2, 0xc83553c5c8965d3d, 0x952ab45cfa97a0b3, 0xde469fbd99a05fe3, 0xa59bc234db398c25,
    0xf6c69a72a3989f5c, 0xb7dcbf5354e9bece, 0x88fcf317f22241e2, 0xcc20ce9bd35c78a5, 0x98165af37b2153df, 0xe2a0b5dc971f303a,
    0xa8d9d1535ce3b396, 0xfb9b7cd9a4a7443c, 0xbb764c4ca7a44410, 0x8bab8eefb6409c1a, 0xd01fef10a657842c, 0x9b10a4e5e9913129,
    0xe7109bfba19c0c9d, 0xac2820d9623bf429, 0x80444b5e7aa7cf85, 0xbf21e44003acdd2d, 0x8e679c2f5e44ff8f, 0xd433179d9c8cb841,
    0x9e19db92b4e31ba9, 0xeb96bf6ebadf77d9, 0xaf87023b9bf0ee6b,
];

#[rustfmt::skip]
const CACHED_POWERS_EXPONENT: [i16; 87] = [
    -1220, -1193, -1166, -1140, -1113, -1087, -1060, -1034, -1007, -980, -954, -927, -901, -874, -847, -821, -794, -768,
    -741, -715, -688, -661, -635, -608, -582, -555, -529, -502, -475, -449, -422, -396, -369, -343, -316, -289, -263, -236,
    -210, -183, -157, -130, -103, -77, -50, -24, 3, 30, 56, 83, 109, 136, 162, 189, 216, 242, 269, 295, 322, 348, 375, 402,
    428, 455, 481, 508, 534, 561, 588, 614, 641, 667, 694, 720, 747, 774, 800, 827, 853, 880, 907, 933, 960, 986, 1013,
    1039, 1066,
];

#[derive(Clone, Copy)]
struct DiyFp {
    significand: u64,
    exponent: i32,
}

impl DiyFp {
    fn new(value: f64) -> Self {
        let bits = value.to_bits();
        let biased_exponent = ((bits & EXPONENT_MASK) >> SIGNIFICAND_BITS) as i32;
        let significand = bits & SIGNIFICAND_MASK;
        if biased_exponent == 0 {
            Self {
                significand,
                exponent: 1 - EXPONENT_BIAS - SIGNIFICAND_BITS,
            }
        } else {
            Self {
                significand: significand + HIDDEN_BIT,
                exponent: biased_exponent - EXPONENT_BIAS - SIGNIFICAND_BITS,
            }
        }
    }

    fn normalize(self) -> Self {
        let shift = self.significand.leading_zeros();
        Self {
            significand: self.significand << shift,
            exponent: self.exponent - shift as i32,
        }
    }

    fn boundaries(self) -> (Self, Self) {
        let plus = Self {
            significand: (self.significand << 1) + 1,
            exponent: self.exponent - 1,
        }
        .normalize();
        let minus = if self.significand == HIDDEN_BIT {
            Self {
                significand: (self.significand << 2) - 1,
                exponent: self.exponent - 2,
            }
        } else {
            Self {
                significand: (self.significand << 1) - 1,
                exponent: self.exponent - 1,
            }
        };
        let minus = Self {
            significand: minus.significand << (minus.exponent - plus.exponent),
            exponent: plus.exponent,
        };
        (minus, plus)
    }

    fn multiply(self, other: Self) -> Self {
        let product = u128::from(self.significand) * u128::from(other.significand) + (1 << (DIY_SIGNIFICAND_BITS - 1));
        Self {
            significand: (product >> DIY_SIGNIFICAND_BITS) as u64,
            exponent: self.exponent + other.exponent + DIY_SIGNIFICAND_BITS,
        }
    }

    fn cached_power(exponent: i32) -> (Self, i32) {
        let estimate = f64::from(3 - DIY_SIGNIFICAND_BITS - exponent) * LOG10_2 - f64::from(MIN_CACHED_POWER + 1);
        let truncated = estimate as i32;
        let power = if estimate - f64::from(truncated) > 0.0 { truncated + 1 } else { truncated };
        let index = power / CACHED_POWER_STEP + 1;
        let cached = Self {
            significand: CACHED_POWERS_SIGNIFICAND[index as usize],
            exponent: i32::from(CACHED_POWERS_EXPONENT[index as usize]),
        };
        (cached, -(MIN_CACHED_POWER + index * CACHED_POWER_STEP))
    }
}

pub(crate) fn push_u64(out: &mut String, value: u64) {
    let mut digits = [0u8; U64_DIGITS];
    let mut start = digits.len();
    let mut rest = value;
    loop {
        start -= 1;
        digits[start] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    out.extend(digits[start..].iter().map(|digit| char::from(*digit)));
}

pub(crate) fn push_i64(out: &mut String, value: i64) {
    if value < 0 {
        out.push('-');
    }
    push_u64(out, value.unsigned_abs());
}

pub(crate) fn push_f64(out: &mut String, value: f64) {
    if value.is_nan() {
        out.push_str("NaN");
    } else if value.is_infinite() {
        out.push_str(if value < 0.0 { "-inf" } else { "inf" });
    } else if value == 0.0 {
        out.push_str(if value.is_sign_negative() { "-0.0" } else { "0.0" });
    } else {
        if value < 0.0 {
            out.push('-');
        }
        let (digits, exponent) = shortest_digits(value.abs());
        let start = out.len();
        push_u64(out, digits);
        push_layout(out, start, exponent);
    }
}

fn shortest_digits(value: f64) -> (u64, i32) {
    let value = DiyFp::new(value);
    let (minus, plus) = value.boundaries();
    let (power, exponent) = DiyFp::cached_power(plus.exponent);
    let scaled = value.normalize().multiply(power);
    let upper = plus.multiply(power);
    let lower = minus.multiply(power);
    let upper = DiyFp {
        significand: upper.significand - 1,
        exponent: upper.exponent,
    };
    generate_digits(scaled, upper, upper.significand - (lower.significand + 1), exponent)
}

fn generate_digits(scaled: DiyFp, upper: DiyFp, mut delta: u64, exponent: i32) -> (u64, i32) {
    let shift = -upper.exponent;
    let one = 1u64 << shift;
    let distance = upper.significand - scaled.significand;
    let mut integral = (upper.significand >> shift) as u32;
    let mut fractional = upper.significand & (one - 1);
    let mut kappa = decimal_length(integral);
    let mut digits = 0u64;
    while kappa > 0 {
        let divisor = POWERS_OF_TEN[kappa - 1] as u32;
        digits = digits * 10 + u64::from(integral / divisor);
        integral %= divisor;
        kappa -= 1;
        let rest = (u64::from(integral) << shift) + fractional;
        if rest <= delta {
            let digits = round_digits(digits, delta, rest, POWERS_OF_TEN[kappa] << shift, distance);
            return (digits, exponent + kappa as i32);
        }
    }
    let mut fraction_digits = 0;
    loop {
        fractional *= 10;
        delta *= 10;
        digits = digits * 10 + (fractional >> shift);
        fractional &= one - 1;
        fraction_digits += 1;
        if fractional < delta {
            let scale = if fraction_digits < SCALED_DISTANCE_DIGITS { POWERS_OF_TEN[fraction_digits] } else { 0 };
            let digits = round_digits(digits, delta, fractional, one, distance * scale);
            return (digits, exponent - fraction_digits as i32);
        }
    }
}

fn round_digits(mut digits: u64, delta: u64, mut rest: u64, ten_kappa: u64, distance: u64) -> u64 {
    while rest < distance && delta - rest >= ten_kappa && (rest + ten_kappa < distance || distance - rest > rest + ten_kappa - distance) {
        digits -= 1;
        rest += ten_kappa;
    }
    digits
}

fn decimal_length(value: u32) -> usize {
    POWERS_OF_TEN[1..9].iter().take_while(|power| u64::from(value) >= **power).count() + 1
}

fn push_layout(out: &mut String, start: usize, exponent: i32) {
    let length = (out.len() - start) as i32;
    let decimal_exponent = length + exponent;
    if exponent >= 0 && decimal_exponent <= MAX_INTEGER_DIGITS {
        out.extend(iter::repeat_n('0', exponent as usize));
        out.push_str(".0");
    } else if decimal_exponent > 0 && decimal_exponent <= MAX_INTEGER_DIGITS {
        out.insert(start + decimal_exponent as usize, '.');
    } else if decimal_exponent > MIN_FRACTION_EXPONENT && decimal_exponent <= 0 {
        out.insert_str(start, &FRACTION_PREFIX[..(2 - decimal_exponent) as usize]);
    } else {
        if length > 1 {
            out.insert(start + 1, '.');
        }
        out.push('e');
        push_i64(out, i64::from(decimal_exponent - 1));
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::E;

    use super::*;

    fn format_f64(value: f64) -> String {
        let mut out = String::new();
        push_f64(&mut out, value);
        out
    }

    #[test]
    fn test_push_f64() {
        assert_eq!(format_f64(1.234e20), "123400000000000000000.0");
        assert_eq!(format_f64(1.234e21), "1.234e21");
        assert_eq!(format_f64(6.02214), "6.02214");
        assert_eq!(format_f64(0.0), "0.0");
        assert_eq!(format_f64(-0.0), "-0.0");
        assert_eq!(format_f64(1.1e128), "1.1e128");
        assert_eq!(format_f64(1.1e-64), "1.1e-64");
        assert_eq!(format_f64(E), "2.718281828459045");
        assert_eq!(format_f64(5e-324), "5e-324");
        assert_eq!(format_f64(f64::MAX), "1.7976931348623157e308");
        assert_eq!(format_f64(f64::MIN), "-1.7976931348623157e308");
        assert_eq!(format_f64(f64::NAN), "NaN");
        assert_eq!(format_f64(f64::INFINITY), "inf");
        assert_eq!(format_f64(f64::NEG_INFINITY), "-inf");
        assert_eq!(format_f64(1.0), "1.0");
        assert_eq!(format_f64(0.001), "0.001");
        assert_eq!(format_f64(0.000001), "0.000001");
        assert_eq!(format_f64(0.0000001), "1e-7");
        assert_eq!(format_f64(0.00000012), "1.2e-7");
        assert_eq!(format_f64(62500.0), "62500.0");
        assert_eq!(format_f64(5.3944220000000005), "5.3944220000000009");
        assert_eq!(format_f64(22.868759999999998), "22.868759999999999");
    }

    #[test]
    fn test_push_integers() {
        let mut out = String::new();
        push_u64(&mut out, 0);
        out.push(' ');
        push_u64(&mut out, u64::MAX);
        out.push(' ');
        push_i64(&mut out, i64::MIN);
        out.push(' ');
        push_i64(&mut out, -7);
        assert_eq!(out, "0 18446744073709551615 -9223372036854775808 -7");
    }
}
