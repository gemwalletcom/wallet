use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(Vec<u32>);

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self(vec![major, minor, patch])
    }
}

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .split('.')
            .map(|part| {
                if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("Invalid version");
                }
                part.parse().map_err(|_| "Invalid version")
            })
            .collect::<Result<Vec<u32>, _>>()
            .map(Self)
    }
}

pub fn is_version_higher(new: String, current: String) -> bool {
    match (new.parse::<Version>(), current.parse::<Version>()) {
        (Ok(new), Ok(current)) => new > current,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!("2.114.32".parse(), Ok(Version::new(2, 114, 32)));
        for value in ["", "invalid", "2..32", ".2.114", "2.114.", "2.114.32-beta", "3.invalid.0", "+3.0.0", " 3.0.0", "4294967296.0.0"] {
            assert!(value.parse::<Version>().is_err(), "{value}");
        }
    }

    #[test]
    fn test_is_version_higher() {
        assert!(is_version_higher("1.2.3".into(), "1.0.0".into()));
        assert!(is_version_higher("2.1.3.4".into(), "2.1.3".into()));
        assert!(is_version_higher("0.1".into(), "0.0.1".into()));
        assert!(is_version_higher("2.27".into(), "2.0.27".into()));
        assert!(!is_version_higher("1.0.0".into(), "1.2.3".into()));
        assert!(!is_version_higher("1.2.3".into(), "1.2.3".into()));
        assert!(!is_version_higher("2.1.3".into(), "2.1.3.4".into()));
        assert!(!is_version_higher("1".into(), "2".into()));
        assert!(!is_version_higher("1.3.100".into(), "2.0.12".into()));
        assert!(!is_version_higher("3.invalid.0".into(), "2.114.32".into()));
        assert!(!is_version_higher("3.0.0".into(), "invalid".into()));
    }
}
