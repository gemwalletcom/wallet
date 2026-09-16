use crate::{
    Result, encode_length_to_compact_u16_bytes,
    instructions::{
        compute_budget::{parse_compute_unit_limit_data, parse_compute_unit_price_data, set_compute_unit_limit, set_compute_unit_price},
        program_ids::compute_budget_program,
    },
    types::{CompiledInstruction, Message, Pubkey, SignatureBytes, VersionedMessageV0},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedTransaction {
    Legacy { signatures: Vec<SignatureBytes>, message: Message },
    V0 { signatures: Vec<SignatureBytes>, message: VersionedMessageV0 },
}

impl VersionedTransaction {
    pub fn message(&self) -> &Message {
        match self {
            Self::Legacy { message, .. } => message,
            Self::V0 { message, .. } => &message.message,
        }
    }

    pub fn message_mut(&mut self) -> &mut Message {
        match self {
            Self::Legacy { message, .. } => message,
            Self::V0 { message, .. } => &mut message.message,
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
        }
    }

    pub fn signatures_mut(&mut self) -> &mut Vec<SignatureBytes> {
        match self {
            Self::Legacy { signatures, .. } => signatures,
            Self::V0 { signatures, .. } => signatures,
        }
    }

    pub fn add_signature(&mut self, signature: SignatureBytes) {
        self.signatures_mut().push(signature);
    }

    pub fn get_compute_unit_price(&self) -> Option<u64> {
        self.compute_budget_data().find_map(parse_compute_unit_price_data)
    }

    pub fn set_compute_unit_price(&mut self, micro_lamports: u64) -> bool {
        self.replace_compute_budget_data(|data| parse_compute_unit_price_data(data).is_some(), set_compute_unit_price(micro_lamports).data)
    }

    pub fn get_compute_unit_limit(&self) -> Option<u32> {
        self.compute_budget_data().find_map(parse_compute_unit_limit_data)
    }

    pub fn set_compute_unit_limit(&mut self, units: u32) -> bool {
        self.replace_compute_budget_data(|data| parse_compute_unit_limit_data(data).is_some(), set_compute_unit_limit(units).data)
    }

    pub fn serialize_message(&self) -> Result<Vec<u8>> {
        match self {
            Self::Legacy { message, .. } => message.serialize_for_signing(),
            Self::V0 { message, .. } => message.serialize_for_signing(),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = encode_length_to_compact_u16_bytes(self.signatures().len())?;
        for signature in self.signatures() {
            bytes.extend_from_slice(signature.as_bytes());
        }
        bytes.extend(self.serialize_message()?);
        Ok(bytes)
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
    use crate::{testkit::mock_transaction_with_accounts, types::message::MESSAGE_VERSION_PREFIX};
    use gem_encoding::decode_base64;

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

    #[test]
    fn test_decode_legacy() {
        let transaction = decode_legacy_transaction();
        assert_eq!(transaction.serialize_message().unwrap()[0], 1);
        assert_eq!(transaction.signatures().len(), 1);
        assert_eq!(transaction.account_keys().len(), 22);
        assert_eq!(transaction.instructions().len(), 7);
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

    fn legacy_transaction_bytes(header: [u8; 3], num_accounts: u8) -> Vec<u8> {
        let mut bytes = vec![0];
        bytes.extend_from_slice(&header);
        bytes.push(num_accounts);
        bytes.extend(repeat_n(0u8, 32 * num_accounts as usize));
        bytes.extend_from_slice(&[0u8; 32]);
        bytes
    }

    #[test]
    fn test_deserialize_rejects_huge_instruction_count() {
        let mut bytes = legacy_transaction_bytes([1, 0, 0], 1);
        bytes.extend(encode_length_to_compact_u16_bytes(60_000).unwrap());

        assert!(VersionedTransaction::deserialize_with_version(&bytes).is_err());
    }

    #[test]
    fn test_deserialize_rejects_inconsistent_header() {
        for header in [[1, 0, 5], [1, 0, 2], [1, 2, 0], [1, 1, 0], [0, 0, 0], [3, 0, 0]] {
            let mut bytes = legacy_transaction_bytes(header, 2);
            bytes.push(0);
            assert!(VersionedTransaction::deserialize_with_version(&bytes).is_err(), "header {header:?} must be rejected");
        }

        let mut bytes = legacy_transaction_bytes([1, 0, 2], 2);
        bytes.push(0);
        bytes.push(0);
        bytes[1] |= MESSAGE_VERSION_PREFIX;
        assert!(VersionedTransaction::deserialize_with_version(&bytes).is_err());
    }

    #[test]
    fn test_deserialize_rejects_trailing_bytes_and_missing_lookup_count() {
        let mut legacy = legacy_transaction_bytes([1, 0, 0], 1);
        legacy.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&legacy).is_ok());
        legacy.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&legacy).is_err());

        let mut v0 = legacy_transaction_bytes([1, 0, 0], 1);
        v0.insert(1, MESSAGE_VERSION_PREFIX);
        v0.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&v0).is_err());
        v0.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&v0).is_ok());
    }

    #[test]
    fn test_deserialize_rejects_unsupported_version() {
        let mut bytes = legacy_transaction_bytes([1, 0, 0], 1);
        bytes.insert(1, MESSAGE_VERSION_PREFIX | 1);
        bytes.push(0);
        bytes.push(0);
        assert!(VersionedTransaction::deserialize_with_version(&bytes).is_err());
    }

    #[test]
    fn test_serialize_roundtrip_legacy() {
        let data = decode_base64(LEGACY_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&data).unwrap();

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
}
