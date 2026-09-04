use primitives::Chain;
use primitives::name::NameRecord;

use super::model::{GemRecipientError, GemRecipientValidation};
use crate::address::{checksum_address, validate_address};
use crate::services::name::rules::is_name_supported;
use crate::services::transfer::GemRecipient;

pub fn validation(chain: Chain, input: &str, name_record: Option<&NameRecord>) -> GemRecipientValidation {
    let is_valid = is_valid(chain, input, name_record);
    GemRecipientValidation {
        is_valid,
        address: address(chain, input, name_record),
        shows_error: shows_error(input, is_valid),
    }
}

pub fn recipient_id(recipient: &GemRecipient) -> String {
    [
        recipient.name.as_deref().unwrap_or_default(),
        &recipient.address,
        recipient.memo.as_deref().unwrap_or_default(),
    ]
    .join("_")
}

pub fn recipient(chain: Chain, input: &str, name_record: Option<&NameRecord>, memo: Option<String>, references: Vec<String>) -> Result<GemRecipient, GemRecipientError> {
    if name_record.is_some_and(|record| !matches_input(record, chain, input)) {
        return Err(GemRecipientError::NameRecordMismatch);
    }
    if !is_valid(chain, input, name_record) {
        return Err(GemRecipientError::InvalidAddress);
    }
    Ok(GemRecipient {
        address: address(chain, input, name_record),
        name: name_record.map(|record| record.name.clone()),
        memo,
        references,
    })
}

fn is_valid(chain: Chain, input: &str, name_record: Option<&NameRecord>) -> bool {
    match name_record {
        Some(record) => matches_input(record, chain, input) && validate_address(&record.address, chain),
        None => !input.trim().is_empty() && validate_address(&checksum_address(input, chain), chain),
    }
}

fn matches_input(record: &NameRecord, chain: Chain, input: &str) -> bool {
    record.name == input && record.chain == chain
}

fn address(chain: Chain, input: &str, name_record: Option<&NameRecord>) -> String {
    let address = name_record.map(|record| record.address.as_str()).filter(|address| !address.is_empty()).unwrap_or(input);
    checksum_address(address, chain)
}

fn shows_error(input: &str, is_valid: bool) -> bool {
    !input.trim().is_empty() && !is_name_supported(input) && !is_valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::name::NameProvider;

    const ADDRESS: &str = "0x1f9090aae28b8a3dceadf281b0f12828e676c326";
    const CHECKSUMMED: &str = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326";

    fn record(name: &str, address: &str, chain: Chain) -> NameRecord {
        NameRecord {
            name: name.to_string(),
            chain,
            address: address.to_string(),
            provider: NameProvider::Ens,
        }
    }

    #[test]
    fn test_address_input_is_checksummed_and_validated() {
        let valid = validation(Chain::Ethereum, ADDRESS, None);
        assert!(valid.is_valid);
        assert_eq!(valid.address, CHECKSUMMED);
        assert!(!valid.shows_error);

        let invalid = validation(Chain::Ethereum, "0xinvalid", None);
        assert!(!invalid.is_valid);
        assert!(invalid.shows_error);
        assert!(!validation(Chain::Ethereum, "", None).shows_error);
        assert!(!validation(Chain::Ethereum, "vitalik.eth", None).shows_error);
    }

    #[test]
    fn test_name_record_must_match_input_and_chain() {
        let ens = record("vitalik.eth", ADDRESS, Chain::Ethereum);
        let valid = validation(Chain::Ethereum, "vitalik.eth", Some(&ens));
        assert!(valid.is_valid);
        assert_eq!(valid.address, CHECKSUMMED);

        assert!(!validation(Chain::Ethereum, "other.eth", Some(&ens)).is_valid);
        assert!(!validation(Chain::Polygon, "vitalik.eth", Some(&ens)).is_valid);
        assert!(!validation(Chain::Ethereum, "vitalik.eth", Some(&record("vitalik.eth", "0xbad", Chain::Ethereum))).is_valid);
    }

    #[test]
    fn test_recipient_builds_from_record_or_address() {
        let ens = record("vitalik.eth", ADDRESS, Chain::Ethereum);
        let named = recipient(Chain::Ethereum, "vitalik.eth", Some(&ens), Some("memo".into()), vec!["ref".into()]).unwrap();
        assert_eq!(named.address, CHECKSUMMED);
        assert_eq!(named.name.as_deref(), Some("vitalik.eth"));
        assert_eq!(named.memo.as_deref(), Some("memo"));
        assert_eq!(named.references, vec!["ref".to_string()]);

        let plain = recipient(Chain::Ethereum, ADDRESS, None, None, vec![]).unwrap();
        assert_eq!(plain.address, CHECKSUMMED);
        assert_eq!(plain.name, None);
        assert_eq!(
            recipient(Chain::Ethereum, "other.eth", Some(&ens), None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(recipient(Chain::Ethereum, "0xinvalid", None, None, vec![]), Err(GemRecipientError::InvalidAddress));
    }

    #[test]
    fn test_input_is_trimmed_and_prefix_is_case_sensitive() {
        let padded = format!("  {ADDRESS} \n");
        assert!(validation(Chain::Ethereum, &padded, None).is_valid);
        assert_eq!(recipient(Chain::Ethereum, &padded, None, None, vec![]).unwrap().address, CHECKSUMMED);

        let upper_prefix = ADDRESS.replacen("0x", "0X", 1);
        assert!(!validation(Chain::Ethereum, &upper_prefix, None).is_valid);
        assert_eq!(recipient(Chain::Ethereum, &upper_prefix, None, None, vec![]), Err(GemRecipientError::InvalidAddress));
    }

    #[test]
    fn test_name_record_matching_is_exact() {
        let ens = record("vitalik.eth", ADDRESS, Chain::Ethereum);
        assert!(!validation(Chain::Ethereum, "Vitalik.eth", Some(&ens)).is_valid);
        assert_eq!(
            recipient(Chain::Ethereum, "Vitalik.eth", Some(&ens), None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(
            recipient(Chain::Ethereum, " vitalik.eth", Some(&ens), None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(
            recipient(Chain::Polygon, "vitalik.eth", Some(&ens), None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );

        let empty = record("vitalik.eth", "", Chain::Ethereum);
        assert_eq!(
            recipient(Chain::Ethereum, "vitalik.eth", Some(&empty), None, vec![]),
            Err(GemRecipientError::InvalidAddress)
        );
        let fallback = validation(Chain::Ethereum, "vitalik.eth", Some(&empty));
        assert!(!fallback.is_valid);
        assert_eq!(fallback.address, "vitalik.eth");
    }

    #[test]
    fn test_non_evm_addresses_keep_their_case() {
        let near = recipient(Chain::Near, "h3rman.near", None, None, vec![]).unwrap();
        assert_eq!(near.address, "h3rman.near");
        assert_eq!(near.name, None);

        let solana = "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2";
        assert_eq!(validation(Chain::Solana, solana, None).address, solana);
        assert!(validation(Chain::Solana, solana, None).is_valid);

        let tron = "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC";
        let tron_recipient = recipient(Chain::Tron, tron, None, Some("  memo ".into()), vec!["a".into(), "b".into()]).unwrap();
        assert_eq!(tron_recipient.address, tron);
        assert_eq!(tron_recipient.memo.as_deref(), Some("  memo "));
        assert_eq!(tron_recipient.references, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(recipient(Chain::Tron, &tron.to_lowercase(), None, None, vec![]), Err(GemRecipientError::InvalidAddress));
    }
}
