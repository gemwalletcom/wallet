use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRefStr, EnumIter, EnumString, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Currency {
    MXN,
    CHF,
    CNY,
    THB,
    HUF,
    AUD,
    IDR,
    RUB,
    ZAR,
    EUR,
    NZD,
    SAR,
    SGD,
    BMD,
    KWD,
    HKD,
    JPY,
    GBP,
    DKK,
    KRW,
    PHP,
    CLP,
    TWD,
    PKR,
    BRL,
    CAD,
    BHD,
    MMK,
    VEF,
    VND,
    CZK,
    TRY,
    INR,
    ARS,
    BDT,
    NOK,
    USD,
    LKR,
    ILS,
    PLN,
    NGN,
    UAH,
    XDR,
    MYR,
    AED,
    SEK,
}

impl Currency {
    pub fn flag(&self) -> &'static str {
        match self {
            Self::MXN => "🇲🇽",
            Self::CHF => "🇨🇭",
            Self::CNY => "🇨🇳",
            Self::THB => "🇹🇭",
            Self::HUF => "🇭🇺",
            Self::AUD => "🇦🇺",
            Self::IDR => "🇮🇩",
            Self::RUB => "🇷🇺",
            Self::ZAR => "🇿🇦",
            Self::EUR => "🇪🇺",
            Self::NZD => "🇳🇿",
            Self::SAR => "🇸🇦",
            Self::SGD => "🇸🇬",
            Self::BMD => "🇧🇲",
            Self::KWD => "🇰🇼",
            Self::HKD => "🇭🇰",
            Self::JPY => "🇯🇵",
            Self::GBP => "🇬🇧",
            Self::DKK => "🇩🇰",
            Self::KRW => "🇰🇷",
            Self::PHP => "🇵🇭",
            Self::CLP => "🇨🇱",
            Self::TWD => "🇹🇼",
            Self::PKR => "🇵🇰",
            Self::BRL => "🇧🇷",
            Self::CAD => "🇨🇦",
            Self::BHD => "🇧🇭",
            Self::MMK => "🇲🇲",
            Self::VEF => "🇻🇪",
            Self::VND => "🇻🇳",
            Self::CZK => "🇨🇿",
            Self::TRY => "🇹🇷",
            Self::INR => "🇮🇳",
            Self::ARS => "🇦🇷",
            Self::BDT => "🇧🇩",
            Self::NOK => "🇳🇴",
            Self::USD => "🇺🇸",
            Self::LKR => "🇱🇰",
            Self::ILS => "🇮🇱",
            Self::PLN => "🇵🇱",
            Self::NGN => "🇳🇬",
            Self::UAH => "🇺🇦",
            Self::XDR => "🏳️",
            Self::MYR => "🇲🇾",
            Self::AED => "🇦🇪",
            Self::SEK => "🇸🇪",
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
