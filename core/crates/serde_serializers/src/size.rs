use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    parse_size(&value).map_err(serde::de::Error::custom)
}

fn parse_size(value: &str) -> Result<usize, String> {
    let value = value.trim();
    let unit_start = value.find(|character: char| !character.is_ascii_digit()).unwrap_or(value.len());
    let (number, unit) = value.split_at(unit_start);

    if number.is_empty() {
        return Err("no number found in size".to_string());
    }

    let number = number.parse::<u64>().map_err(|_| format!("invalid size: {number}"))?;
    let multiplier: u64 = match unit.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1,
        "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        "KIB" => 1_024,
        "MIB" => 1_048_576,
        "GIB" => 1_073_741_824,
        "TIB" => 1_099_511_627_776,
        unit => return Err(format!("unknown size unit '{unit}', supported: B, KB, MB, GB, TB, KiB, MiB, GiB, TiB")),
    };

    let bytes = number.checked_mul(multiplier).ok_or_else(|| "size exceeds u64 limit".to_string())?;
    usize::try_from(bytes).map_err(|_| "size exceeds platform limit".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1 B").unwrap(), 1);
        assert_eq!(parse_size("64 MB").unwrap(), 64_000_000);
        assert_eq!(parse_size("1GB").unwrap(), 1_000_000_000);
        assert_eq!(parse_size("2 GiB").unwrap(), 2_147_483_648);
        assert_eq!(parse_size("1 kib").unwrap(), 1_024);
        assert!(parse_size("").is_err());
        assert!(parse_size("GB").is_err());
        assert!(parse_size("1 XB").is_err());
        assert!(parse_size("1.5 GB").is_err());
        assert!(parse_size(&format!("{} TB", usize::MAX)).is_err());
    }
}
