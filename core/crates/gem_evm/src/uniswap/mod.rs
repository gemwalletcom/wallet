use alloy_primitives::aliases::{I24, U24};

pub mod actions;
pub mod command;
pub mod contracts;
pub mod deployment;
pub mod path;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FeeTier {
    One = 1,
    Five = 5,
    Ten = 10,
    Fifty = 50,
    Hundred = 100,
    TwoHundred = 200,
    FiveHundred = 500,
    ThousandFiveHundred = 1500,
    TwoThousand = 2000,
    TwoThousandFiveHundred = 2500,
    ThreeThousand = 3000,
    TenThousand = 10000,
}

impl FeeTier {
    pub fn as_u24(&self) -> U24 {
        let fee_bytes = (*self as u32).to_le_bytes();
        U24::from_le_bytes([fee_bytes[0], fee_bytes[1], fee_bytes[2]])
    }

    pub fn as_tick_spacing(&self) -> I24 {
        I24::unchecked_from(*self as u32)
    }

    pub fn default_tick_spacing(&self) -> Option<I24> {
        match self {
            FeeTier::Five | FeeTier::Hundred => Some(I24::unchecked_from(1)),
            FeeTier::FiveHundred => Some(I24::unchecked_from(10)),
            FeeTier::ThreeThousand => Some(I24::unchecked_from(60)),
            FeeTier::TenThousand => Some(I24::unchecked_from(200)),
            FeeTier::One | FeeTier::Ten | FeeTier::Fifty | FeeTier::TwoHundred | FeeTier::ThousandFiveHundred | FeeTier::TwoThousand | FeeTier::TwoThousandFiveHundred => None,
        }
    }
}

impl TryFrom<&str> for FeeTier {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let u32_value = value.parse::<u32>()?;
        Self::try_from(u32_value)
    }
}

impl TryFrom<u32> for FeeTier {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FeeTier::One),
            5 => Ok(FeeTier::Five),
            10 => Ok(FeeTier::Ten),
            50 => Ok(FeeTier::Fifty),
            100 => Ok(FeeTier::Hundred),
            200 => Ok(FeeTier::TwoHundred),
            500 => Ok(FeeTier::FiveHundred),
            1500 => Ok(FeeTier::ThousandFiveHundred),
            2000 => Ok(FeeTier::TwoThousand),
            2500 => Ok(FeeTier::TwoThousandFiveHundred),
            3000 => Ok(FeeTier::ThreeThousand),
            10000 => Ok(FeeTier::TenThousand),
            _ => Err(format!("Invalid fee tier: {}", value).into()),
        }
    }
}
