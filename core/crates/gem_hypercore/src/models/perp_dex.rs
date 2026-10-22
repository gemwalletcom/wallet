use serde::Deserialize;
use strum::{AsRefStr, EnumString};

const HIP3_PERP_ASSET_OFFSET: u32 = 100_000;
const HIP3_PERP_ASSET_STRIDE: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, EnumString, Deserialize)]
#[serde(try_from = "String")]
pub enum PerpDex {
    #[strum(serialize = "")]
    Main,
    #[strum(serialize = "xyz")]
    Xyz,
}

impl TryFrom<String> for PerpDex {
    type Error = strum::ParseError;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        code.parse()
    }
}

impl PerpDex {
    pub fn name(self) -> Option<String> {
        match self {
            Self::Main => None,
            dex => Some(dex.as_ref().to_string()),
        }
    }

    pub fn holds(self, coin: &str) -> bool {
        coin.split_once(':').map_or("", |(dex, _)| dex) == self.as_ref()
    }

    pub fn is_isolated_only(self) -> bool {
        match self {
            Self::Main => false,
            Self::Xyz => true,
        }
    }

    pub fn asset_index(self, meta_index: u32) -> u32 {
        match self {
            Self::Main => meta_index,
            Self::Xyz => HIP3_PERP_ASSET_OFFSET + HIP3_PERP_ASSET_STRIDE + meta_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holds() {
        assert!(PerpDex::Main.holds("BTC"));
        assert!(!PerpDex::Main.holds("xyz:NVDA"));
        assert!(PerpDex::Xyz.holds("xyz:NVDA"));
        assert!(!PerpDex::Xyz.holds("BTC"));
        assert!(!PerpDex::Xyz.holds("flx:TSLA"));
    }

    #[test]
    fn test_parse_socket_dex() {
        assert_eq!(serde_json::from_str::<PerpDex>("\"\"").unwrap(), PerpDex::Main);
        assert_eq!(serde_json::from_str::<PerpDex>("\"xyz\"").unwrap(), PerpDex::Xyz);
        assert!(serde_json::from_str::<PerpDex>("\"flx\"").is_err(), "a dex the app does not load is not read");
    }

    #[test]
    fn test_asset_index() {
        assert_eq!(PerpDex::Main.asset_index(5), 5);
        assert_eq!(PerpDex::Xyz.asset_index(0), 110_000, "xyz is Hyperliquid's perp dex 1, so its first market is 100000 + 1 * 10000");
        assert_eq!(PerpDex::Xyz.asset_index(3), 110_003);
    }
}
