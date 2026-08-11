use primitives::Address as AddressTrait;

pub struct NearAddress([u8; 32]);

impl AddressTrait for NearAddress {
    fn try_parse(address: &str) -> Option<Self> {
        hex::decode(address).ok()?.try_into().ok().map(Self)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        hex::encode(self.0)
    }
}

pub fn validate_address(address: &str) -> bool {
    validate_account_id(address)
}

pub fn validate_account_id(account_id: &str) -> bool {
    if !(2..=64).contains(&account_id.len()) {
        return false;
    }

    let mut previous_is_separator = true;
    for character in account_id.bytes() {
        match character {
            b'a'..=b'z' | b'0'..=b'9' => previous_is_separator = false,
            b'-' | b'_' | b'.' if !previous_is_separator => previous_is_separator = true,
            _ => return false,
        }
    }
    !previous_is_separator
}

pub(crate) fn is_implicit_address(address: &str) -> bool {
    NearAddress::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_account() {
        let implicit_address = "e3ac115fd911eb985ffd884ee60302c84dc94df52127ccde8d6fb97ad6d22945";
        let eth_implicit_address = "0x85f17cf997934a597031b2e18a9ab6ebd4b9f6a4";
        let deterministic_address = "0s85f17cf997934a597031b2e18a9ab6ebd4b9f6a4";

        for address in ["aa", "alice-near_1.testnet", "h3rman.near", implicit_address, eth_implicit_address, deterministic_address] {
            assert!(validate_account_id(address));
        }
        for address in ["a", "Alice.near", "ƒelicia.near", ".near", "alice..near", "alice.near-"] {
            assert!(!validate_account_id(address));
        }
        assert!(!validate_account_id(&"a".repeat(65)));
    }

    #[test]
    fn test_near_address() {
        let implicit_address = "e3ac115fd911eb985ffd884ee60302c84dc94df52127ccde8d6fb97ad6d22945";
        let parsed = NearAddress::try_parse(implicit_address).unwrap();

        assert!(is_implicit_address(implicit_address));
        assert!(!is_implicit_address("h3rman.near"));
        assert!(!is_implicit_address("0x85f17cf997934a597031b2e18a9ab6ebd4b9f6a4"));
        assert_eq!(parsed.as_bytes().len(), 32);
        assert_eq!(parsed.encode(), implicit_address);
    }
}
