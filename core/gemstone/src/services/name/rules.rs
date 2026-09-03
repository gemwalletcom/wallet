use primitives::ChainAddress;

use crate::services::collections::unique_by;

const NAME_RECORD_DEBOUNCE_MILLISECONDS: u64 = 250;

pub fn name_record_debounce_milliseconds() -> u64 {
    NAME_RECORD_DEBOUNCE_MILLISECONDS
}

pub fn is_name_supported(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2 && parts.last().is_some_and(|suffix| !suffix.is_empty())
}

pub fn unique_requests(requests: Vec<ChainAddress>) -> Vec<ChainAddress> {
    unique_by(requests.into_iter().filter(|request| !request.address.is_empty()), |request| {
        (request.chain, request.address.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_is_name_supported() {
        assert!(is_name_supported("vitalik.eth"));
        assert!(is_name_supported("a.b.sol"));
        assert!(!is_name_supported("vitalik"));
        assert!(!is_name_supported("vitalik."));
        assert!(!is_name_supported("0x1234"));
    }

    #[test]
    fn test_unique_requests() {
        let requests = vec![
            ChainAddress::new(Chain::Ethereum, "0xa".to_string()),
            ChainAddress::new(Chain::Ethereum, "0xa".to_string()),
            ChainAddress::new(Chain::Bitcoin, "0xa".to_string()),
            ChainAddress::new(Chain::Ethereum, String::new()),
        ];
        let unique = unique_requests(requests);
        assert_eq!(unique.len(), 2);
        assert_eq!(unique[0], ChainAddress::new(Chain::Ethereum, "0xa".to_string()));
        assert_eq!(unique[1], ChainAddress::new(Chain::Bitcoin, "0xa".to_string()));
    }
}
