use std::collections::HashSet;

use crate::{
    Result, SolanaError, encode_length_to_compact_u16_bytes,
    types::{CompiledInstruction, MessageAddressTableLookup, Pubkey, invalid_transaction},
};

pub(crate) const MESSAGE_VERSION_PREFIX: u8 = 0x80;
pub(crate) const MESSAGE_V1_PREFIX: u8 = MESSAGE_VERSION_PREFIX | 1;
#[cfg(feature = "signer")]
pub(crate) const OFFCHAIN_MESSAGE_PREFIX: u8 = 0xFF;

const MAX_V1_SIGNATURES: usize = 12;
const MAX_V1_ADDRESSES: usize = 64;
const MAX_V1_INSTRUCTIONS: usize = 64;
const MIN_V1_HEAP_SIZE: u32 = 32 * 1024;
const MAX_V1_HEAP_SIZE: u32 = 256 * 1024;
const PRIORITY_FEE_MASK: u32 = 0b11;
const COMPUTE_UNIT_LIMIT_MASK: u32 = 0b100;
const LOADED_ACCOUNTS_DATA_SIZE_LIMIT_MASK: u32 = 0b1000;
const HEAP_SIZE_MASK: u32 = 0b10000;
pub(crate) const V1_CONFIG_MASK: u32 = PRIORITY_FEE_MASK | COMPUTE_UNIT_LIMIT_MASK | LOADED_ACCOUNTS_DATA_SIZE_LIMIT_MASK | HEAP_SIZE_MASK;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

impl MessageHeader {
    pub(crate) fn validate(&self, account_keys_len: usize) -> Result<()> {
        let num_required_signatures = self.num_required_signatures as usize;
        if num_required_signatures == 0 || num_required_signatures > account_keys_len {
            return Err(invalid_transaction("required-signature count must be between one and the account count"));
        }
        if self.num_readonly_signed_accounts as usize >= num_required_signatures {
            return Err(invalid_transaction("read-only signer count leaves no writable fee payer"));
        }
        if self.num_readonly_unsigned_accounts as usize > account_keys_len - num_required_signatures {
            return Err(invalid_transaction("read-only account count exceeds unsigned-account count"));
        }
        Ok(())
    }
}

pub(crate) fn validate_instructions(instructions: &[CompiledInstruction], num_static_keys: usize, num_account_keys: usize) -> Result<()> {
    for instruction in instructions {
        let program_id_index = instruction.program_id_index as usize;
        if program_id_index == 0 || program_id_index >= num_static_keys {
            return Err(invalid_transaction("instruction program index is invalid"));
        }
        if instruction.accounts.iter().any(|index| *index as usize >= num_account_keys) {
            return Err(invalid_transaction("instruction account index is invalid"));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub header: MessageHeader,
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: [u8; 32],
    pub instructions: Vec<CompiledInstruction>,
}

impl Message {
    pub fn serialize_for_signing(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![
            self.header.num_required_signatures,
            self.header.num_readonly_signed_accounts,
            self.header.num_readonly_unsigned_accounts,
        ];

        bytes.extend(encode_length_to_compact_u16_bytes(self.account_keys.len())?);
        for pubkey in &self.account_keys {
            bytes.extend_from_slice(pubkey.as_bytes());
        }

        bytes.extend_from_slice(&self.recent_blockhash);

        bytes.extend(encode_length_to_compact_u16_bytes(self.instructions.len())?);
        for instruction in &self.instructions {
            bytes.push(instruction.program_id_index);
            push_compact_slice(&mut bytes, &instruction.accounts)?;
            push_compact_slice(&mut bytes, &instruction.data)?;
        }

        Ok(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedMessageV0 {
    pub message: Message,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

impl VersionedMessageV0 {
    pub fn serialize_for_signing(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![MESSAGE_VERSION_PREFIX];
        bytes.extend(self.message.serialize_for_signing()?);

        bytes.extend(encode_length_to_compact_u16_bytes(self.address_table_lookups.len())?);
        for lookup in &self.address_table_lookups {
            bytes.extend_from_slice(lookup.account_key.as_bytes());
            push_compact_slice(&mut bytes, &lookup.writable_indexes)?;
            push_compact_slice(&mut bytes, &lookup.readonly_indexes)?;
        }

        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransactionConfig {
    pub priority_fee: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub loaded_accounts_data_size_limit: Option<u32>,
    pub heap_size: Option<u32>,
}

impl TransactionConfig {
    pub(crate) fn mask(&self) -> u32 {
        let mut mask = 0;
        if self.priority_fee.is_some() {
            mask |= PRIORITY_FEE_MASK;
        }
        if self.compute_unit_limit.is_some() {
            mask |= COMPUTE_UNIT_LIMIT_MASK;
        }
        if self.loaded_accounts_data_size_limit.is_some() {
            mask |= LOADED_ACCOUNTS_DATA_SIZE_LIMIT_MASK;
        }
        if self.heap_size.is_some() {
            mask |= HEAP_SIZE_MASK;
        }
        mask
    }

    pub(crate) fn has_priority_fee(mask: u32) -> bool {
        mask & PRIORITY_FEE_MASK == PRIORITY_FEE_MASK
    }

    pub(crate) fn has_invalid_priority_fee(mask: u32) -> bool {
        let priority_fee = mask & PRIORITY_FEE_MASK;
        priority_fee != 0 && priority_fee != PRIORITY_FEE_MASK
    }

    pub(crate) fn has_compute_unit_limit(mask: u32) -> bool {
        mask & COMPUTE_UNIT_LIMIT_MASK != 0
    }

    pub(crate) fn has_loaded_accounts_data_size_limit(mask: u32) -> bool {
        mask & LOADED_ACCOUNTS_DATA_SIZE_LIMIT_MASK != 0
    }

    pub(crate) fn has_heap_size(mask: u32) -> bool {
        mask & HEAP_SIZE_MASK != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedMessageV1 {
    pub message: Message,
    pub config: TransactionConfig,
}

impl VersionedMessageV1 {
    pub fn serialize_for_signing(&self) -> Result<Vec<u8>> {
        self.validate()?;

        let mut bytes = vec![
            MESSAGE_V1_PREFIX,
            self.message.header.num_required_signatures,
            self.message.header.num_readonly_signed_accounts,
            self.message.header.num_readonly_unsigned_accounts,
        ];
        bytes.extend_from_slice(&self.config.mask().to_le_bytes());
        bytes.extend_from_slice(&self.message.recent_blockhash);
        bytes.push(self.message.instructions.len() as u8);
        bytes.push(self.message.account_keys.len() as u8);
        for pubkey in &self.message.account_keys {
            bytes.extend_from_slice(pubkey.as_bytes());
        }
        if let Some(priority_fee) = self.config.priority_fee {
            bytes.extend_from_slice(&priority_fee.to_le_bytes());
        }
        if let Some(compute_unit_limit) = self.config.compute_unit_limit {
            bytes.extend_from_slice(&compute_unit_limit.to_le_bytes());
        }
        if let Some(loaded_accounts_data_size_limit) = self.config.loaded_accounts_data_size_limit {
            bytes.extend_from_slice(&loaded_accounts_data_size_limit.to_le_bytes());
        }
        if let Some(heap_size) = self.config.heap_size {
            bytes.extend_from_slice(&heap_size.to_le_bytes());
        }
        for instruction in &self.message.instructions {
            bytes.push(instruction.program_id_index);
            bytes.push(instruction.accounts.len() as u8);
            bytes.extend_from_slice(&(instruction.data.len() as u16).to_le_bytes());
        }
        for instruction in &self.message.instructions {
            bytes.extend_from_slice(&instruction.accounts);
            bytes.extend_from_slice(&instruction.data);
        }
        Ok(bytes)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let account_keys_len = self.message.account_keys.len();
        if self.message.header.num_required_signatures as usize > MAX_V1_SIGNATURES || account_keys_len > MAX_V1_ADDRESSES {
            return Err(SolanaError::TransactionTooLarge);
        }
        self.message.header.validate(account_keys_len)?;
        if self.message.account_keys.iter().collect::<HashSet<_>>().len() != account_keys_len {
            return Err(invalid_transaction("V1 contains duplicate accounts"));
        }
        if self.message.instructions.len() > MAX_V1_INSTRUCTIONS {
            return Err(SolanaError::TransactionTooLarge);
        }
        if let Some(heap_size) = self.config.heap_size
            && (!(MIN_V1_HEAP_SIZE..=MAX_V1_HEAP_SIZE).contains(&heap_size) || !heap_size.is_multiple_of(1024))
        {
            return Err(invalid_transaction("V1 heap size is invalid"));
        }
        for instruction in &self.message.instructions {
            if instruction.accounts.len() > u8::MAX as usize {
                return Err(invalid_transaction("V1 instruction account count exceeds 255"));
            }
            if instruction.data.len() > u16::MAX as usize {
                return Err(invalid_transaction("V1 instruction data length exceeds 65535"));
            }
        }
        validate_instructions(&self.message.instructions, account_keys_len, account_keys_len)
    }
}

fn push_compact_slice(bytes: &mut Vec<u8>, slice: &[u8]) -> Result<()> {
    bytes.extend(encode_length_to_compact_u16_bytes(slice.len())?);
    bytes.extend_from_slice(slice);
    Ok(())
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use super::*;
    use crate::testkit::TEST_BLOCKHASH;

    #[test]
    fn test_versioned_message() {
        let v0_message = VersionedMessageV0 {
            message: Message {
                header: MessageHeader::mock(1, 1),
                account_keys: vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])],
                recent_blockhash: [0u8; 32],
                instructions: vec![CompiledInstruction::mock(1, vec![0], vec![])],
            },
            address_table_lookups: vec![MessageAddressTableLookup::new(Pubkey::new([2; 32]), vec![0, 1], vec![2])],
        };

        let mut expected = hex!("8001000102").to_vec();
        expected.extend_from_slice(&[0; 32]);
        expected.extend_from_slice(&[1; 32]);
        expected.extend_from_slice(&[0; 32]);
        expected.extend_from_slice(&hex!("010101000001"));
        expected.extend_from_slice(&[2; 32]);
        expected.extend_from_slice(&hex!("0200010102"));

        assert_eq!(v0_message.serialize_for_signing().unwrap(), expected);
    }

    #[test]
    fn test_v1_serialize_for_signing() {
        let message = VersionedMessageV1::mock(
            1,
            vec![Pubkey::new([1; 32]), Pubkey::new([2; 32])],
            vec![CompiledInstruction::mock(1, vec![], vec![])],
            TransactionConfig {
                priority_fee: Some(0x0102030405060708),
                compute_unit_limit: Some(0x11223344),
                ..TransactionConfig::default()
            },
        );

        let mut expected = hex!("8101000107000000").to_vec();
        expected.extend_from_slice(&TEST_BLOCKHASH);
        expected.extend_from_slice(&hex!("0102"));
        expected.extend_from_slice(&[1; 32]);
        expected.extend_from_slice(&[2; 32]);
        expected.extend_from_slice(&hex!("08070605040302014433221101000000"));

        let grouped_message = VersionedMessageV1::mock(
            1,
            vec![Pubkey::new([1; 32]), Pubkey::new([2; 32]), Pubkey::new([3; 32])],
            vec![
                CompiledInstruction::mock(1, vec![0], vec![0xaa, 0xbb]),
                CompiledInstruction::mock(2, vec![0, 1], vec![0xcc]),
            ],
            TransactionConfig::default(),
        );

        let mut grouped_expected = hex!("8101000200000000").to_vec();
        grouped_expected.extend_from_slice(&TEST_BLOCKHASH);
        grouped_expected.extend_from_slice(&hex!("0203"));
        grouped_expected.extend_from_slice(&[1; 32]);
        grouped_expected.extend_from_slice(&[2; 32]);
        grouped_expected.extend_from_slice(&[3; 32]);
        grouped_expected.extend_from_slice(&hex!("010102000202010000aabb0001cc"));

        assert_eq!(message.serialize_for_signing().unwrap(), expected);
        assert_eq!(grouped_message.serialize_for_signing().unwrap(), grouped_expected);
    }
}
