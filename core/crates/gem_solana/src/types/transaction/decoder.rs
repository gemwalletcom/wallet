use std::fmt::Display;

use super::VersionedTransaction;
use crate::{
    Result, SolanaError, decode_compact_u16_len,
    types::{CompiledInstruction, Message, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes, VersionedMessageV0, message::MESSAGE_VERSION_PREFIX},
};

const PUBKEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

impl VersionedTransaction {
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        let mut decoder = Decoder::new(bytes);
        let signatures = decode_signatures(&mut decoder)?;
        let transaction = decode_message(&mut decoder, signatures)?;
        if !decoder.is_empty() {
            return Err(invalid_transaction("trailing bytes after the message"));
        }
        Ok(transaction)
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn peek_u8(&self, missing: &'static str) -> Result<u8> {
        self.bytes.get(self.offset).copied().ok_or_else(|| invalid_transaction(missing))
    }

    fn read_u8(&mut self, missing: &'static str) -> Result<u8> {
        let byte = self.peek_u8(missing)?;
        self.offset += 1;
        Ok(byte)
    }

    fn read_bytes(&mut self, len: usize, missing: &'static str) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(len).filter(|end| *end <= self.bytes.len()).ok_or_else(|| invalid_transaction(missing))?;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn read_array<const N: usize>(&mut self, missing: &'static str) -> Result<[u8; N]> {
        self.read_bytes(N, missing)?.try_into().map_err(|_| invalid_transaction(missing))
    }

    fn read_arrays<const N: usize>(&mut self, count: usize, missing: &'static str) -> Result<Vec<[u8; N]>> {
        self.read_bytes(count.saturating_mul(N), missing)?
            .chunks_exact(N)
            .map(|chunk| chunk.try_into().map_err(|_| invalid_transaction(missing)))
            .collect()
    }

    fn read_length(&mut self) -> Result<usize> {
        let (len, consumed) = decode_compact_u16_len(&self.bytes[self.offset..]).map_err(invalid_transaction)?;
        self.offset += consumed;
        Ok(len)
    }
}

fn invalid_transaction(reason: impl Display) -> SolanaError {
    SolanaError::invalid_input(format!("Invalid Solana transaction: {reason}"))
}

fn decode_signatures(decoder: &mut Decoder<'_>) -> Result<Vec<SignatureBytes>> {
    let signature_count = decoder.read_length()?;
    let signatures = decoder.read_arrays::<SIGNATURE_LENGTH>(signature_count, "not enough bytes for signatures")?;
    Ok(signatures.into_iter().map(SignatureBytes::new).collect())
}

fn decode_message(decoder: &mut Decoder<'_>, signatures: Vec<SignatureBytes>) -> Result<VersionedTransaction> {
    let version = decoder.peek_u8("message is empty")?;
    if version & MESSAGE_VERSION_PREFIX == 0 {
        let message = decode_message_body(decoder)?;
        return Ok(VersionedTransaction::Legacy { signatures, message });
    }

    decoder.read_u8("message is empty")?;
    match version & !MESSAGE_VERSION_PREFIX {
        0 => {
            let message = decode_message_body(decoder)?;
            let address_table_lookups = decode_address_table_lookups(decoder)?;
            Ok(VersionedTransaction::V0 {
                signatures,
                message: VersionedMessageV0 { message, address_table_lookups },
            })
        }
        version => Err(invalid_transaction(format!("unsupported message version: {version}"))),
    }
}

fn decode_message_body(decoder: &mut Decoder<'_>) -> Result<Message> {
    let header = MessageHeader {
        num_required_signatures: decoder.read_u8("missing required-signature count")?,
        num_readonly_signed_accounts: decoder.read_u8("missing read-only signer count")?,
        num_readonly_unsigned_accounts: decoder.read_u8("missing read-only account count")?,
    };

    let account_count = decoder.read_length()?;
    let account_keys = decoder.read_arrays::<PUBKEY_LENGTH>(account_count, "not enough bytes for accounts")?;
    let account_keys = account_keys.into_iter().map(Pubkey::new).collect::<Vec<_>>();
    validate_header_counts(&header, account_keys.len())?;

    let recent_blockhash = decoder.read_array("missing recent blockhash")?;
    let instruction_count = decoder.read_length()?;
    if instruction_count.saturating_mul(3) > decoder.remaining() {
        return Err(invalid_transaction("instruction count exceeds remaining bytes"));
    }
    let instructions = (0..instruction_count).map(|_| decode_instruction(decoder)).collect::<Result<Vec<_>>>()?;

    Ok(Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    })
}

fn validate_header_counts(header: &MessageHeader, account_keys_len: usize) -> Result<()> {
    let num_required_signatures = header.num_required_signatures as usize;
    if num_required_signatures == 0 || num_required_signatures > account_keys_len {
        return Err(invalid_transaction("required-signature count must be between one and the account count"));
    }
    if header.num_readonly_signed_accounts as usize >= num_required_signatures {
        return Err(invalid_transaction("read-only signer count leaves no writable fee payer"));
    }
    if header.num_readonly_unsigned_accounts as usize > account_keys_len - num_required_signatures {
        return Err(invalid_transaction("read-only account count exceeds unsigned-account count"));
    }
    Ok(())
}

fn decode_instruction(decoder: &mut Decoder<'_>) -> Result<CompiledInstruction> {
    let program_id_index = decoder.read_u8("missing instruction program ID index")?;
    let account_count = decoder.read_length()?;
    let accounts = decoder.read_bytes(account_count, "not enough instruction account indexes")?.to_vec();
    let data_length = decoder.read_length()?;
    let data = decoder.read_bytes(data_length, "not enough instruction data")?.to_vec();
    Ok(CompiledInstruction { program_id_index, accounts, data })
}

fn decode_address_table_lookups(decoder: &mut Decoder<'_>) -> Result<Vec<MessageAddressTableLookup>> {
    let lookup_count = decoder.read_length()?;
    (0..lookup_count)
        .map(|_| {
            let account_key = Pubkey::new(decoder.read_array("incomplete address lookup table")?);
            let writable_count = decoder.read_length()?;
            let writable_indexes = decoder.read_bytes(writable_count, "not enough writable lookup indexes")?.to_vec();
            let readonly_count = decoder.read_length()?;
            let readonly_indexes = decoder.read_bytes(readonly_count, "not enough read-only lookup indexes")?.to_vec();
            Ok(MessageAddressTableLookup {
                account_key,
                writable_indexes,
                readonly_indexes,
            })
        })
        .collect()
}
