use primitives::ChainAddress;
use primitives::name::NameRecord;

use super::model::GemNameRecordState;
use crate::services::collections::unique_by;

const NAME_RECORD_DEBOUNCE_MILLISECONDS: u64 = 250;

pub fn name_record_debounce_milliseconds() -> u64 {
    NAME_RECORD_DEBOUNCE_MILLISECONDS
}

pub fn is_name_supported(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2 && parts.last().is_some_and(|suffix| !suffix.is_empty())
}

pub fn resolved(record: Option<NameRecord>) -> GemNameRecordState {
    match record {
        Some(record) if !record.name.is_empty() && !record.address.is_empty() => GemNameRecordState::Complete { record },
        Some(_) | None => GemNameRecordState::Error,
    }
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
    fn test_resolved_completes_only_with_a_name_and_an_address() {
        let record = |name: &str, address: &str| NameRecord {
            name: name.into(),
            chain: Chain::Ethereum,
            address: address.into(),
            provider: primitives::name::NameProvider::Ens,
        };
        let complete = resolved(Some(record("vitalik.eth", "0x1")));

        assert_eq!(complete.requested_name().as_deref(), Some("vitalik.eth"));
        assert!(complete.record().is_some());
        assert_eq!(resolved(Some(record("vitalik.eth", ""))), GemNameRecordState::Error);
        assert_eq!(resolved(Some(record("", "0x1"))), GemNameRecordState::Error);
        assert_eq!(resolved(None), GemNameRecordState::Error);
        assert_eq!(GemNameRecordState::Loading { name: "vitalik.eth".into() }.requested_name().as_deref(), Some("vitalik.eth"));
        assert_eq!(GemNameRecordState::None.requested_name(), None);
    }

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
