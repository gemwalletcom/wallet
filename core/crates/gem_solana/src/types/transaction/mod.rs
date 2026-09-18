use crate::{
    Result, encode_length_to_compact_u16_bytes,
    instructions::{
        compute_budget::{
            find_unique_compute_unit_limit, find_unique_compute_unit_price, parse_compute_unit_limit_data, parse_compute_unit_price_data, set_compute_unit_limit,
            set_compute_unit_price,
        },
        program_ids::compute_budget_program,
    },
    types::{CompiledInstruction, MAX_V1_TRANSACTION_SIZE, Message, Pubkey, SignatureBytes, TransactionConfig, VersionedMessageV0, VersionedMessageV1, invalid_transaction},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedTransaction {
    Legacy { signatures: Vec<SignatureBytes>, message: Message },
    V0 { signatures: Vec<SignatureBytes>, message: VersionedMessageV0 },
    V1 { signatures: Vec<SignatureBytes>, message: VersionedMessageV1 },
}

impl VersionedTransaction {
    pub fn message(&self) -> &Message {
        match self {
            Self::Legacy { message, .. } => message,
            Self::V0 { message, .. } => &message.message,
            Self::V1 { message, .. } => &message.message,
        }
    }

    pub fn message_mut(&mut self) -> &mut Message {
        match self {
            Self::Legacy { message, .. } => message,
            Self::V0 { message, .. } => &mut message.message,
            Self::V1 { message, .. } => &mut message.message,
        }
    }

    pub fn num_required_signatures(&self) -> u8 {
        self.message().header.num_required_signatures
    }

    pub fn account_keys(&self) -> &[Pubkey] {
        &self.message().account_keys
    }

    pub fn recent_blockhash(&self) -> &[u8; 32] {
        &self.message().recent_blockhash
    }

    pub fn instructions(&self) -> &[CompiledInstruction] {
        &self.message().instructions
    }

    pub fn signatures(&self) -> &[SignatureBytes] {
        match self {
            Self::Legacy { signatures, .. } => signatures,
            Self::V0 { signatures, .. } => signatures,
            Self::V1 { signatures, .. } => signatures,
        }
    }

    pub fn signatures_mut(&mut self) -> &mut Vec<SignatureBytes> {
        match self {
            Self::Legacy { signatures, .. } => signatures,
            Self::V0 { signatures, .. } => signatures,
            Self::V1 { signatures, .. } => signatures,
        }
    }

    pub fn add_signature(&mut self, signature: SignatureBytes) {
        self.signatures_mut().push(signature);
    }

    pub fn get_compute_unit_price(&self) -> Option<u64> {
        match self {
            Self::Legacy { .. } | Self::V0 { .. } => find_unique_compute_unit_price(self.compute_budget_data()),
            Self::V1 { .. } => None,
        }
    }

    pub fn set_compute_unit_price(&mut self, micro_lamports: u64) -> bool {
        match self {
            Self::Legacy { .. } | Self::V0 { .. } => {
                self.replace_compute_budget_data(|data| parse_compute_unit_price_data(data).is_some(), set_compute_unit_price(micro_lamports).data)
            }
            Self::V1 { .. } => false,
        }
    }

    pub fn get_priority_fee(&self) -> Option<u64> {
        self.transaction_config().and_then(|config| config.priority_fee)
    }

    pub fn transaction_config(&self) -> Option<&TransactionConfig> {
        match self {
            Self::Legacy { .. } | Self::V0 { .. } => None,
            Self::V1 { message, .. } => Some(&message.config),
        }
    }

    pub fn transaction_config_mut(&mut self) -> Option<&mut TransactionConfig> {
        match self {
            Self::Legacy { .. } | Self::V0 { .. } => None,
            Self::V1 { message, .. } => Some(&mut message.config),
        }
    }

    pub fn get_compute_unit_limit(&self) -> Option<u32> {
        match self {
            Self::V1 { message, .. } => message.config.compute_unit_limit,
            Self::Legacy { .. } | Self::V0 { .. } => find_unique_compute_unit_limit(self.compute_budget_data()),
        }
    }

    pub fn set_compute_unit_limit(&mut self, units: u32) -> bool {
        match self {
            Self::V1 { message, .. } => {
                message.config.compute_unit_limit = Some(units);
                true
            }
            Self::Legacy { .. } | Self::V0 { .. } => self.replace_compute_budget_data(|data| parse_compute_unit_limit_data(data).is_some(), set_compute_unit_limit(units).data),
        }
    }

    pub fn serialize_message(&self) -> Result<Vec<u8>> {
        match self {
            Self::Legacy { message, .. } => message.serialize_for_signing(),
            Self::V0 { message, .. } => message.serialize_for_signing(),
            Self::V1 { message, .. } => message.serialize_for_signing(),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Self::Legacy { .. } | Self::V0 { .. } => {
                let mut bytes = encode_length_to_compact_u16_bytes(self.signatures().len())?;
                for signature in self.signatures() {
                    bytes.extend_from_slice(signature.as_bytes());
                }
                bytes.extend(self.serialize_message()?);
                Ok(bytes)
            }
            Self::V1 { signatures, message } => {
                if signatures.len() != message.message.header.num_required_signatures as usize {
                    return Err(invalid_transaction("V1 signature count does not match the message header"));
                }
                let mut bytes = self.serialize_message()?;
                for signature in signatures {
                    bytes.extend_from_slice(signature.as_bytes());
                }
                if bytes.len() > MAX_V1_TRANSACTION_SIZE {
                    return Err(invalid_transaction("V1 transaction size exceeds 4096 bytes"));
                }
                Ok(bytes)
            }
        }
    }

    fn compute_budget_program_index(&self) -> Option<u8> {
        self.account_keys()
            .iter()
            .position(|key| *key == compute_budget_program())
            .and_then(|index| u8::try_from(index).ok())
    }

    fn compute_budget_data(&self) -> impl Iterator<Item = &[u8]> {
        let program_id_index = self.compute_budget_program_index();
        self.instructions()
            .iter()
            .filter(move |instruction| Some(instruction.program_id_index) == program_id_index)
            .map(|instruction| instruction.data.as_slice())
    }

    fn replace_compute_budget_data(&mut self, matches: impl Fn(&[u8]) -> bool, data: Vec<u8>) -> bool {
        let Some(program_id_index) = self.compute_budget_program_index() else {
            return false;
        };
        let instruction = self
            .message_mut()
            .instructions
            .iter_mut()
            .find(|instruction| instruction.program_id_index == program_id_index && matches(&instruction.data));
        match instruction {
            Some(instruction) => {
                instruction.data = data;
                true
            }
            None => false,
        }
    }
}

mod decoder;

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use super::*;
    use crate::testkit::{mock_transaction_with_accounts, mock_v1_transaction};
    use crate::types::message::MESSAGE_VERSION_PREFIX;
    use gem_encoding::decode_base64;
    use serde_json::Value;

    const LEGACY_TX: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAgWAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEbrtjJdvWJAv9GZTGL8LaZtMvDe4j2ery4z7rOkRbioxZflXLFqWqlAt1REFSiam0ljvfB1tbBruEpGRTcUQIyQ+ddH9NRneQZQXje5U/3c4cZ2f1JESi76CvBvRoQ6I1LeNzfZ4ZONkowCnqCyeo5+D6Q21gn3U7HVw/KD3HyUW5gVpu5F8ZojWkXLg/+3N6q3ojiaqYyBIbz7VP7jS5Yktrxv5b22C/EFSDs5jUPA7Gz3GLdBNs0iwBHlqUqNEeyNpDX0HWNHV2LiVDOx6m018ea6P+1xroNvWKhmDeTW7oqHXAEK1ih5IO68BBiiKqWNR5VZdBgBsnR+rZKfpfuyE3yQziYO+SoWzCXuvQLyVcRCNKJrACzaN8XXUR1z3rOt8T1lYUIIAQS7tqgcLRsn18N4vVQgXQyv3bQWjh3JtpQT3Bgy9N9myGC4PDjGuVnx2Y7mF4eqlysb0rgrdrB2+FMK6YBPXtlXF4QPTY6rEe+hxkBpCoGK7UJu5BHUK4gJhAewgMolkoyq6sTbFQFuR86447k9ky2veh5uGg40gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+FkDBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hBUpTWpkpIQZNJOhxYNo4fHw1td28kruB5B+oQEEFRI0Gm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAQbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpDgNoX46QkFPkWBIcZvWnau3HcGqhHIL4qpUqjyt4ealuCa42Moiy1mB8REcWJlkis4eCMyKfY2HMRfldn8r2XwcQAAUCoGgGABAACQNwEQEAAAAAAA8GAAYAEw4UAQAVERQUEgAHExEGCQoCBAULDAgBMSsE7QsayR5iC50OAAAAAAA8XqkAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEBAAAABgIUAwYAAAEJFAMKAwAJA8wSAAAAAAAADgIADQwCAAAAODEAAAAAAAA=";

    const MAYAN_V0_TX: &str = "ATzYOiofQZSWsNe3SxxEPip+Xp9A2Fji+h0xfs7FkmvQxNNgwjeEbTlMr7+e42q9vcvExw2CX4PgNBRuY77O+waAAQAEDPlBHYJN7SVAqQdmNtdFQsCIDVJuEnf59VTtTCOGI7yLh4jpmImexNtJSORTO+sbJ63Aysdx88si41jIW1Wf65qHxwlVbaZ8xI24o/VzmleK1NqPB2lMTcy78ZFbqJ6agIqQAqWC7XmuIVDA/VxhSMZPxFOazPZMJbWyD+TYtXxA3sS/qzC61MydFxPOY3xt62Ug5Tp3r/hC0NimkXNfrMH0UmoX+WTY7c2jVeACjg8EqVgtZZSXgaQRvotGaelPhCySBd5s0S8tvrZZSGGBUknE3Jjh4aGsgXpNY0QHkFnJayU0QDsmAQ7sF/E5yI6Oq1k8w8tnKB6wJR28JzZwp3KVGAf9PgfpG6VoBYOYtT4QWhLzz8wJo5Da/9f9tVVfo7Qj5Z1paZLqq3kUJ1PAm9bYE1qpQE9jUkcSHEnSn0OVAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAAAGTCSuZOXkbU4/LKndRkF4gm16E7to0DdpTPoefoS0rYF08m4FFLws+yIpIkWYIyALDIz0sekCn1BgZGSqLNo5CwsAF0FkUEJ2ZE5kVGxlWmNsc25JeDVkeUExCgAFAhxCBwAKAAkDBBcBAAAAAAAJBxUABgUWGRQACQcVAAEAGxkUAQEJAxkAAQwCAAAAC/UHPQYAAAAJAhQBAREJBxUAAwAWGRQBAQgoHAABAxsWFBQGHRwhACITAQMPERIUBBACBxweAA4gHw0MAQMbFhQUGjIBLQAAALtk+swxxK8UC/UHPQYAAAD8nvqKAAAAAGQAAAAAAAIAAAAaQAYAAl8A0CAAAgkEFAEAAAEJCQoYAAAFBgMWFxQZxgEgTCkMJ6KE2yRjS4oAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhOLnPgTBemY6Iqc117gEL72WnXMeAAAAAAAAAAAAAAAAAIM1ifzW7bbgj0x8MtT3G1S9oCkT/ySLiQAAAAAAAAAAAAAAAG4DAAAAAAAA8dcFAAAAAAA7a4ppAAAAAAAAAAAAAAAAAAAAAN3bmpXkQ6IE64ZQ1epXjtcH/iEjAAMC0SsrhG32PclzqA5blk8lqOrlLpR5OoOt60ksQpGLgw4D2cMN+5YRja4DNaX11bThHmwP8vCzdRYtSPXpFGWT2KsACgUGESAhKSowMTQme3jXiWKuyj4qkRn+CZK3WspZpXBM+tnHyaYm4WA/BAMICgsDBgkMIbxt+8RM8X78HZP9nB+0Ah2xfOX9io4UH0AdkLgPT00Fdnd6fH4CdHU=";

    fn decode_legacy_transaction() -> VersionedTransaction {
        let data = decode_base64(LEGACY_TX).unwrap();
        VersionedTransaction::deserialize_with_version(&data).unwrap()
    }

    fn decode_mayan_transaction() -> VersionedTransaction {
        let data = decode_base64(MAYAN_V0_TX).unwrap();
        VersionedTransaction::deserialize_with_version(&data).unwrap()
    }

    fn v1_mainnet_bytes() -> Vec<u8> {
        let fixture: Value = serde_json::from_str(include_str!("../../../testdata/transaction_v1_mainnet.json")).unwrap();
        decode_base64(fixture["transaction"][0].as_str().unwrap()).unwrap()
    }

    #[test]
    fn test_get_compute_unit_price() {
        assert_eq!(decode_legacy_transaction().get_compute_unit_price(), Some(70_000));
        assert_eq!(decode_mayan_transaction().get_compute_unit_price(), Some(71_428));
    }

    #[test]
    fn test_set_compute_unit_price() {
        let mut transaction = decode_legacy_transaction();
        assert!(transaction.set_compute_unit_price(999_999));
        assert_eq!(transaction.get_compute_unit_price(), Some(999_999));

        let mut without_budget = mock_transaction_with_accounts(vec![Pubkey::new([1; 32])], vec![]);
        assert!(!without_budget.set_compute_unit_price(1));
        assert_eq!(without_budget.get_compute_unit_price(), None);
    }

    #[test]
    fn test_get_compute_unit_limit() {
        assert_eq!(decode_legacy_transaction().get_compute_unit_limit(), Some(420_000));
        assert_eq!(decode_mayan_transaction().get_compute_unit_limit(), Some(475_676));
    }

    #[test]
    fn test_set_compute_unit_limit() {
        let mut transaction = decode_legacy_transaction();
        assert!(transaction.set_compute_unit_limit(500_000));
        assert_eq!(transaction.get_compute_unit_limit(), Some(500_000));
    }

    fn legacy_message_bytes(header: [u8; 3], num_accounts: u8) -> Vec<u8> {
        let mut bytes = header.to_vec();
        bytes.push(num_accounts);
        bytes.extend(repeat_n(0u8, 32 * num_accounts as usize));
        bytes.extend_from_slice(&[0u8; 32]);
        bytes
    }

    fn transaction_bytes(message: &[u8], signature_count: u8) -> Vec<u8> {
        let mut bytes = vec![signature_count];
        bytes.extend(repeat_n(0u8, 64 * signature_count as usize));
        bytes.extend_from_slice(message);
        bytes
    }

    #[test]
    fn test_deserialize_rejects_huge_instruction_count() {
        let mut message = legacy_message_bytes([1, 0, 0], 1);
        message.extend(encode_length_to_compact_u16_bytes(60_000).unwrap());

        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 1)).is_err());
    }

    #[test]
    fn test_deserialize_rejects_inconsistent_header() {
        for header in [[1, 0, 5], [1, 0, 2], [1, 2, 0], [1, 1, 0], [0, 0, 0], [3, 0, 0]] {
            let mut message = legacy_message_bytes(header, 2);
            message.push(0);
            let bytes = transaction_bytes(&message, header[0]);
            assert!(VersionedTransaction::deserialize_with_version(&bytes).is_err(), "header {header:?} must be rejected");
        }

        let mut message = legacy_message_bytes([1, 0, 2], 2);
        message.push(0);
        message.push(0);
        message[0] |= MESSAGE_VERSION_PREFIX;
        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 1)).is_err());
    }

    #[test]
    fn test_deserialize_rejects_signature_count_mismatch() {
        let mut message = legacy_message_bytes([2, 0, 0], 2);
        message.push(0);

        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 2)).is_ok());
        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 1)).is_err());
        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 3)).is_err());
    }

    #[test]
    fn test_deserialize_rejects_trailing_bytes_and_missing_lookup_count() {
        let mut message = legacy_message_bytes([1, 0, 0], 1);
        message.push(0);
        let mut legacy = transaction_bytes(&message, 1);
        assert!(VersionedTransaction::deserialize_with_version(&legacy).is_ok());
        legacy.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&legacy).is_err());

        let mut message = legacy_message_bytes([1, 0, 0], 1);
        message.insert(0, MESSAGE_VERSION_PREFIX);
        message.push(0);
        let mut v0 = transaction_bytes(&message, 1);
        assert!(VersionedTransaction::deserialize_with_version(&v0).is_err());
        v0.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&v0).is_ok());
    }

    #[test]
    fn test_deserialize_rejects_too_many_account_keys() {
        let mut message = legacy_message_bytes([1, 0, 0], 1);
        message.insert(0, MESSAGE_VERSION_PREFIX);
        message.push(0);
        message.push(1);
        message.extend_from_slice(&[7u8; 32]);
        message.extend(encode_length_to_compact_u16_bytes(255).unwrap());
        message.extend(0..=254u8);
        let mut full = message.clone();
        message.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 1)).is_ok());

        full.push(1);
        full.push(255);
        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&full, 1)).is_err());
    }

    #[test]
    fn test_deserialize_rejects_unsupported_version() {
        let mut message = legacy_message_bytes([1, 0, 0], 1);
        message.insert(0, MESSAGE_VERSION_PREFIX | 1);
        message.push(0);
        message.push(0);

        assert!(VersionedTransaction::deserialize_with_version(&transaction_bytes(&message, 1)).is_err());
    }

    #[test]
    fn test_serialize_roundtrip_legacy() {
        let data = decode_base64(LEGACY_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&data).unwrap();

        assert_eq!(transaction.serialize_message().unwrap()[0], 1);
        assert_eq!(transaction.signatures().len(), 1);
        assert_eq!(transaction.account_keys().len(), 22);
        assert_eq!(transaction.instructions().len(), 7);

        let reserialized = transaction.serialize().unwrap();
        assert_eq!(reserialized, data, "byte-exact roundtrip failed");

        let decoded = VersionedTransaction::deserialize_with_version(&reserialized).unwrap();
        assert_eq!(decoded, transaction);
    }

    #[test]
    fn test_serialize_roundtrip_v0() {
        let data = decode_base64(MAYAN_V0_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&data).unwrap();

        let reserialized = transaction.serialize().unwrap();
        assert_eq!(reserialized, data, "byte-exact roundtrip failed");

        let decoded = VersionedTransaction::deserialize_with_version(&reserialized).unwrap();
        assert_eq!(decoded, transaction);
    }

    #[test]
    fn test_serialize_roundtrip_v1_mainnet_transaction() {
        let data = v1_mainnet_bytes();
        let transaction = VersionedTransaction::deserialize_with_version(&data).unwrap();

        assert_eq!(data.len(), MAX_V1_TRANSACTION_SIZE);
        assert_eq!(transaction.signatures().len(), 1);
        assert_eq!(transaction.account_keys().len(), 2);
        assert_eq!(transaction.instructions().len(), 1);
        assert_eq!(transaction.instructions()[0].data.len(), 3905);
        assert_eq!(transaction.get_priority_fee(), Some(10_000));
        assert_eq!(transaction.get_compute_unit_limit(), Some(10_000));
        assert_eq!(transaction.transaction_config().unwrap().loaded_accounts_data_size_limit, Some(8192));
        assert_eq!(transaction.transaction_config().unwrap().heap_size, None);
        assert_eq!(transaction.serialize_message().unwrap().len(), 4032);
        assert_eq!(transaction.serialize().unwrap(), data);
    }

    #[test]
    fn test_v1_preserves_omitted_and_explicit_zero_config() {
        let mut omitted = mock_v1_transaction(1, 0);
        omitted.transaction_config_mut().unwrap().priority_fee = None;
        let omitted_bytes = omitted.serialize().unwrap();
        let decoded_omitted = VersionedTransaction::deserialize_with_version(&omitted_bytes).unwrap();

        let mut explicit_zero = omitted;
        explicit_zero.transaction_config_mut().unwrap().priority_fee = Some(0);
        let explicit_zero_bytes = explicit_zero.serialize().unwrap();
        let decoded_explicit_zero = VersionedTransaction::deserialize_with_version(&explicit_zero_bytes).unwrap();

        assert_eq!(decoded_omitted.get_priority_fee(), None);
        assert_eq!(decoded_explicit_zero.get_priority_fee(), Some(0));
        assert_eq!(explicit_zero_bytes.len(), omitted_bytes.len() + 8);
        assert_eq!(decoded_omitted.serialize().unwrap(), omitted_bytes);
        assert_eq!(decoded_explicit_zero.serialize().unwrap(), explicit_zero_bytes);
    }

    #[test]
    fn test_v1_rejects_malformed_wire_data() {
        let transaction = mock_v1_transaction(1, 0);
        let bytes = transaction.serialize().unwrap();

        let mut unknown_mask = bytes.clone();
        unknown_mask[4] |= 0b100000;
        assert!(VersionedTransaction::deserialize_with_version(&unknown_mask).is_err());

        let mut partial_priority_fee = bytes.clone();
        partial_priority_fee[4] &= !0b10;
        assert!(VersionedTransaction::deserialize_with_version(&partial_priority_fee).is_err());

        let mut duplicate_account = bytes.clone();
        duplicate_account.copy_within(42..74, 74);
        assert!(VersionedTransaction::deserialize_with_version(&duplicate_account).is_err());

        let mut invalid_program_index = bytes.clone();
        invalid_program_index[126] = 2;
        assert!(VersionedTransaction::deserialize_with_version(&invalid_program_index).is_err());

        let mut truncated = bytes.clone();
        truncated.pop();
        assert!(VersionedTransaction::deserialize_with_version(&truncated).is_err());

        let mut trailing = bytes;
        trailing.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&trailing).is_err());

        let mut oversized_wire = v1_mainnet_bytes();
        oversized_wire[124..126].copy_from_slice(&3906u16.to_le_bytes());
        oversized_wire.insert(4032, 0);
        assert_eq!(oversized_wire.len(), MAX_V1_TRANSACTION_SIZE + 1);
        assert!(VersionedTransaction::deserialize_with_version(&oversized_wire).is_err());
    }

    #[test]
    fn test_v1_serialize_rejects_invalid_transactions() {
        let mut missing_signature = mock_v1_transaction(1, 0);
        missing_signature.signatures_mut().clear();
        assert!(missing_signature.serialize().is_err());

        let mut oversized_transaction = VersionedTransaction::deserialize_with_version(&v1_mainnet_bytes()).unwrap();
        oversized_transaction.message_mut().instructions[0].data.push(0);
        assert!(oversized_transaction.serialize().is_err());

        assert!(mock_v1_transaction(13, 0).serialize().is_err());

        let mut too_many_accounts = mock_v1_transaction(1, 0);
        too_many_accounts.message_mut().account_keys.extend((2..=64).map(|value| Pubkey::new([value; 32])));
        assert_eq!(too_many_accounts.account_keys().len(), 65);
        assert!(too_many_accounts.serialize().is_err());

        let mut too_many_instructions = mock_v1_transaction(1, 0);
        let instruction = too_many_instructions.instructions()[0].clone();
        too_many_instructions.message_mut().instructions.resize(65, instruction);
        assert!(too_many_instructions.serialize().is_err());

        let mut invalid_heap_size = mock_v1_transaction(1, 0);
        invalid_heap_size.transaction_config_mut().unwrap().heap_size = Some(32 * 1024 + 1);
        assert!(invalid_heap_size.serialize().is_err());
    }
}
