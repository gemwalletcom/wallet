use primitives::name::NameRecord;
use primitives::{AddressName, AddressType, Chain, ChainAddress};

use super::model::{GemAddressNameUpdate, GemNameInputStep, GemNameRecordState};
use crate::services::collections::unique_by;

const NAME_RECORD_DEBOUNCE_MILLISECONDS: u64 = 250;

fn name_record_debounce_milliseconds() -> u64 {
    NAME_RECORD_DEBOUNCE_MILLISECONDS
}

pub fn address_name_update(name: AddressName) -> GemAddressNameUpdate {
    let replacement = name.address_type.clone();
    GemAddressNameUpdate {
        replaces_types: AddressType::all().into_iter().filter(|stored| *stored == replacement || !names_the_user_owns(stored)).collect(),
        name,
    }
}

fn names_the_user_owns(address_type: &AddressType) -> bool {
    match address_type {
        AddressType::Contact | AddressType::InternalWallet => true,
        AddressType::Address | AddressType::Contract | AddressType::Asset | AddressType::Validator => false,
    }
}

pub fn is_name_supported(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2 && parts.last().is_some_and(|suffix| !suffix.is_empty())
}

pub fn name_input_step(state: &GemNameRecordState, name: &str, chain: Option<Chain>) -> GemNameInputStep {
    if name.is_empty() {
        return GemNameInputStep::Reset;
    }
    let Some(chain) = chain else {
        return GemNameInputStep::Reset;
    };
    if state.requested() == Some((name.to_string(), chain)) {
        return GemNameInputStep::Unchanged;
    }
    if !is_name_supported(name) {
        return GemNameInputStep::Reset;
    }
    GemNameInputStep::Resolve {
        name: name.to_string(),
        debounce_milliseconds: name_record_debounce_milliseconds(),
    }
}

pub fn resolved_state(state: &GemNameRecordState, name: &str, chain: Chain, resolved: GemNameRecordState) -> GemNameRecordState {
    match state {
        GemNameRecordState::Loading { name: loading, chain: loading_chain } if loading == name && *loading_chain == chain => resolved,
        _ => state.clone(),
    }
}

pub fn resolved(record: Option<NameRecord>) -> GemNameRecordState {
    match record {
        Some(record) if !record.name.is_empty() && !record.address.is_empty() => GemNameRecordState::Complete { record },
        Some(_) | None => GemNameRecordState::Error,
    }
}

pub fn unique_requests(requests: Vec<ChainAddress>) -> Vec<ChainAddress> {
    unique_by(requests.into_iter().filter(|request| !request.address.is_empty()), |request| (request.chain, request.address.clone()))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_an_empty_or_unsupported_name_resets_and_a_repeat_changes_nothing() {
        let idle = GemNameRecordState::None;

        assert_eq!(name_input_step(&idle, "", Some(Chain::Ethereum)), GemNameInputStep::Reset);
        assert_eq!(name_input_step(&idle, "vitalik", Some(Chain::Ethereum)), GemNameInputStep::Reset, "a name without a suffix resolves nowhere");
        assert_eq!(name_input_step(&idle, "vitalik.eth", None), GemNameInputStep::Reset, "no chain, nothing to resolve against");
        assert_eq!(
            name_input_step(&loading("vitalik.eth", Chain::Ethereum), "vitalik.eth", Some(Chain::Ethereum)),
            GemNameInputStep::Unchanged,
            "the name already being resolved is not resolved twice"
        );
        assert_eq!(
            name_input_step(&loading("vitalik.eth", Chain::Ethereum), "vitalik.eth", Some(Chain::Solana)),
            GemNameInputStep::Resolve {
                name: "vitalik.eth".to_string(),
                debounce_milliseconds: name_record_debounce_milliseconds(),
            },
            "the same name on another chain is another question"
        );
    }

    #[test]
    fn test_a_supported_name_resolves_after_the_debounce() {
        assert_eq!(
            name_input_step(&GemNameRecordState::None, "vitalik.eth", Some(Chain::Ethereum)),
            GemNameInputStep::Resolve {
                name: "vitalik.eth".to_string(),
                debounce_milliseconds: name_record_debounce_milliseconds(),
            }
        );
    }

    #[test]
    fn test_a_result_for_a_question_no_longer_being_asked_is_dropped() {
        let pending = loading("vitalik.eth", Chain::Ethereum);

        assert_eq!(resolved_state(&pending, "vitalik.eth", Chain::Ethereum, GemNameRecordState::Error), GemNameRecordState::Error);
        assert_eq!(
            resolved_state(&pending, "other.eth", Chain::Ethereum, GemNameRecordState::Error),
            pending,
            "the answer to an old query does not replace the current one"
        );
        assert_eq!(
            resolved_state(&pending, "vitalik.eth", Chain::Solana, GemNameRecordState::Error),
            pending,
            "the answer for the chain the user left does not consume the chain they moved to"
        );
    }
    use super::*;
    use primitives::Chain;

    fn loading(name: &str, chain: Chain) -> GemNameRecordState {
        GemNameRecordState::Loading { name: name.to_string(), chain }
    }

    #[test]
    fn test_resolved_completes_only_with_a_name_and_an_address() {
        let complete = resolved(Some(NameRecord::mock("vitalik.eth", "0x1")));

        assert_eq!(complete.requested(), Some(("vitalik.eth".to_string(), NameRecord::mock("vitalik.eth", "0x1").chain)));
        assert!(complete.record().is_some());
        assert_eq!(resolved(Some(NameRecord::mock("vitalik.eth", ""))), GemNameRecordState::Error);
        assert_eq!(resolved(Some(NameRecord::mock("", "0x1"))), GemNameRecordState::Error);
        assert_eq!(resolved(None), GemNameRecordState::Error);
        assert_eq!(loading("vitalik.eth", Chain::Ethereum).requested(), Some(("vitalik.eth".to_string(), Chain::Ethereum)));
        assert_eq!(GemNameRecordState::None.requested(), None);
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

    #[test]
    fn test_a_name_the_user_owns_is_only_replaced_by_its_own_kind() {
        let update = |address_type: AddressType| {
            address_name_update(AddressName {
                chain: Chain::Ethereum,
                address: "0xa".to_string(),
                name: "name".to_string(),
                address_type,
                status: primitives::VerificationStatus::Unverified,
                image_url: None,
            })
            .replaces_types
        };

        assert_eq!(update(AddressType::Address), vec![AddressType::Address, AddressType::Contract, AddressType::Asset, AddressType::Validator]);
        assert_eq!(
            update(AddressType::Contact),
            vec![AddressType::Address, AddressType::Contract, AddressType::Asset, AddressType::Validator, AddressType::Contact],
            "a contact replaces a remote name and its own, never the wallet's"
        );
        assert_eq!(
            update(AddressType::InternalWallet),
            vec![AddressType::Address, AddressType::Contract, AddressType::Asset, AddressType::Validator, AddressType::InternalWallet]
        );
    }
}
