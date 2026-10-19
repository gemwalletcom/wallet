use std::error::Error;

use crate::models::{RunGetMethodResult, StackEntry};

pub const GET_PUBLIC_KEY_METHOD: &str = "get_public_key";
const PUBLIC_KEY_HEX_LENGTH: usize = 64;

pub fn map_public_key(result: &RunGetMethodResult) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
    if result.exit_code != 0 {
        return Ok(None);
    }
    let value = result.stack.first().and_then(StackEntry::as_num).ok_or("missing TON public key stack entry")?;
    let digits = value.strip_prefix("0x").filter(|digits| digits.len() <= PUBLIC_KEY_HEX_LENGTH).ok_or_else(|| format!("invalid TON public key {value}"))?;
    let public_key = hex::decode(format!("{digits:0>PUBLIC_KEY_HEX_LENGTH$}"))?;
    Ok(Some(hex::encode(public_key)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_public_key() {
        let key = "7952e94118f34607c75e23258dd9220d66ccac5a3ee074125c25068e8107bfbf";
        assert_eq!(map_public_key(&RunGetMethodResult::mock(0, vec![StackEntry::Num(format!("0x{key}"))])).unwrap().unwrap(), key);
        assert_eq!(map_public_key(&RunGetMethodResult::mock(0, vec![StackEntry::Num("0xABC".to_string())])).unwrap().unwrap(), format!("{:0>64}", "abc"));
        assert_eq!(map_public_key(&RunGetMethodResult::mock(-13, vec![])).unwrap(), None);
        assert!(map_public_key(&RunGetMethodResult::mock(0, vec![])).is_err());
        assert!(map_public_key(&RunGetMethodResult::mock(0, vec![StackEntry::Num("12345".to_string())])).is_err());
        assert!(map_public_key(&RunGetMethodResult::mock(0, vec![StackEntry::Num(format!("0x1{key}"))])).is_err());
        assert!(map_public_key(&RunGetMethodResult::mock(0, vec![StackEntry::Num("0xzz".to_string())])).is_err());
    }
}
