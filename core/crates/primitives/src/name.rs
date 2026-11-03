use crate::chain::Chain;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameRecord {
    pub name: String,
    pub chain: Chain,
    pub address: String,
    pub provider: NameProvider,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NameProvider {
    Ud,
    Ens,
    Sns,
    Ton,
    Spaceid,
    Did,
    Suins,
    Aptos,
    Injective,
    Icns,
    Lens,
    Basenames,
    Hyperliquid,
    AllDomains,
    Near,
}

pub fn is_name(value: &str) -> bool {
    let labels: Vec<&str> = value.split('.').collect();
    labels.len() >= 2 && labels.iter().all(|label| !label.is_empty() && label.chars().all(is_name_char))
}

fn is_name_char(char: char) -> bool {
    !char.is_ascii() || char.is_ascii_alphanumeric() || char == '-' || char == '_'
}

#[cfg(test)]
mod tests {
    use super::is_name;

    #[test]
    fn test_is_name() {
        assert!(is_name("vitalik.eth"));
        assert!(is_name("a.b.sol"));
        assert!(is_name("alice-bob_1.base.eth"));
        assert!(is_name("🐈.hl"));
        assert!(!is_name("vitalik"));
        assert!(!is_name("vitalik."));
        assert!(!is_name("vitalik..eth"));
        assert!(!is_name("0x1234"));
        assert!(!is_name("yyy@gmail.com"));
        assert!(!is_name("vitalik .eth"));
    }
}
