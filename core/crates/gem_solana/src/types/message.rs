use crate::{
    Result, encode_length_to_compact_u16_bytes,
    types::{CompiledInstruction, MessageAddressTableLookup, Pubkey},
};

pub(crate) const MESSAGE_VERSION_PREFIX: u8 = 0x80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
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
        let mut bytes = vec![self.header.num_required_signatures, self.header.num_readonly_signed_accounts, self.header.num_readonly_unsigned_accounts];

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

fn push_compact_slice(bytes: &mut Vec<u8>, slice: &[u8]) -> Result<()> {
    bytes.extend(encode_length_to_compact_u16_bytes(slice.len())?);
    bytes.extend_from_slice(slice);
    Ok(())
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use super::*;

    #[test]
    fn test_versioned_message() {
        let v0_message = VersionedMessageV0 {
            message: Message {
                header: MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 1,
                },
                account_keys: vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])],
                recent_blockhash: [0u8; 32],
                instructions: vec![CompiledInstruction {
                    program_id_index: 1,
                    accounts: vec![0],
                    data: vec![],
                }],
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
}
