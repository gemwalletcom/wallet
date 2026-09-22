use crate::{
    Result, encode_length_to_compact_u16_bytes,
    instructions::{
        compute_budget::{find_unique_compute_unit_limit, find_unique_compute_unit_price, parse_compute_unit_limit_data, parse_compute_unit_price_data, set_compute_unit_limit, set_compute_unit_price},
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
            Self::Legacy { .. } | Self::V0 { .. } => self.replace_compute_budget_data(|data| parse_compute_unit_price_data(data).is_some(), set_compute_unit_price(micro_lamports).data),
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
        self.account_keys().iter().position(|key| *key == compute_budget_program()).and_then(|index| u8::try_from(index).ok())
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
        let instruction = self.message_mut().instructions.iter_mut().find(|instruction| instruction.program_id_index == program_id_index && matches(&instruction.data));
        match instruction {
            Some(instruction) => {
                instruction.data = data;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_LEGACY_TX, TEST_MAYAN_V0_TX, mock_transaction_with_accounts, mock_v1_mainnet_transaction_bytes, mock_v1_transaction};
    use gem_encoding::decode_base64;

    fn decode(transaction_base64: &str) -> VersionedTransaction {
        VersionedTransaction::deserialize_with_version(&decode_base64(transaction_base64).unwrap()).unwrap()
    }

    #[test]
    fn test_get_compute_unit_price() {
        assert_eq!(decode(TEST_LEGACY_TX).get_compute_unit_price(), Some(70_000));
        assert_eq!(decode(TEST_MAYAN_V0_TX).get_compute_unit_price(), Some(71_428));
    }

    #[test]
    fn test_set_compute_unit_price() {
        let mut transaction = decode(TEST_LEGACY_TX);
        assert!(transaction.set_compute_unit_price(999_999));
        assert_eq!(transaction.get_compute_unit_price(), Some(999_999));

        let mut without_budget = mock_transaction_with_accounts(vec![Pubkey::new([1; 32])], vec![]);
        assert!(!without_budget.set_compute_unit_price(1));
        assert_eq!(without_budget.get_compute_unit_price(), None);
    }

    #[test]
    fn test_get_compute_unit_limit() {
        assert_eq!(decode(TEST_LEGACY_TX).get_compute_unit_limit(), Some(420_000));
        assert_eq!(decode(TEST_MAYAN_V0_TX).get_compute_unit_limit(), Some(475_676));
    }

    #[test]
    fn test_set_compute_unit_limit() {
        let mut transaction = decode(TEST_LEGACY_TX);
        assert!(transaction.set_compute_unit_limit(500_000));
        assert_eq!(transaction.get_compute_unit_limit(), Some(500_000));
    }

    #[test]
    fn test_serialize_roundtrip_legacy() {
        let data = decode_base64(TEST_LEGACY_TX).unwrap();
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
        let data = decode_base64(TEST_MAYAN_V0_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&data).unwrap();

        let reserialized = transaction.serialize().unwrap();
        assert_eq!(reserialized, data, "byte-exact roundtrip failed");

        let decoded = VersionedTransaction::deserialize_with_version(&reserialized).unwrap();
        assert_eq!(decoded, transaction);
    }

    #[test]
    fn test_serialize_roundtrip_v1_mainnet_transaction() {
        let data = mock_v1_mainnet_transaction_bytes();
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
    fn test_v1_serialize_rejects_invalid_transactions() {
        let mut missing_signature = mock_v1_transaction(1, 0);
        missing_signature.signatures_mut().clear();
        assert!(missing_signature.serialize().is_err());

        let mut oversized_transaction = VersionedTransaction::deserialize_with_version(&mock_v1_mainnet_transaction_bytes()).unwrap();
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
