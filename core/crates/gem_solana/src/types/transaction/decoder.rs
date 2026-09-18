use super::VersionedTransaction;
use crate::{
    Result, decode_compact_u16_len,
    types::{
        CompiledInstruction, MAX_ACCOUNT_KEYS, MAX_TRANSACTION_SIZE, MAX_V1_TRANSACTION_SIZE, Message, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes,
        TransactionConfig, VersionedMessageV0, VersionedMessageV1, invalid_transaction,
        message::{MESSAGE_V1_PREFIX, MESSAGE_VERSION_PREFIX, V1_CONFIG_MASK, validate_instructions},
    },
};

const PUBKEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

impl VersionedTransaction {
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        let mut decoder = Decoder::new(bytes);
        let transaction = if decoder.peek_u8("transaction is empty")? == MESSAGE_V1_PREFIX {
            if bytes.len() > MAX_V1_TRANSACTION_SIZE {
                return Err(invalid_transaction("V1 transaction size exceeds 4096 bytes"));
            }
            decode_v1_transaction(&mut decoder)?
        } else {
            if bytes.len() > MAX_TRANSACTION_SIZE {
                return Err(invalid_transaction("transaction size exceeds 1232 bytes"));
            }
            let signatures = decode_signatures(&mut decoder)?;
            decode_message(&mut decoder, signatures)?
        };
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
        let end = self
            .offset
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid_transaction(missing))?;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn read_array<const N: usize>(&mut self, missing: &'static str) -> Result<[u8; N]> {
        self.read_bytes(N, missing)?.try_into().map_err(|_| invalid_transaction(missing))
    }

    fn read_u16(&mut self, missing: &'static str) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_array(missing)?))
    }

    fn read_u32(&mut self, missing: &'static str) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_array(missing)?))
    }

    fn read_u64(&mut self, missing: &'static str) -> Result<u64> {
        Ok(u64::from_le_bytes(self.read_array(missing)?))
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

fn decode_signatures(decoder: &mut Decoder<'_>) -> Result<Vec<SignatureBytes>> {
    let signature_count = decoder.read_length()?;
    let signatures = decoder.read_arrays::<SIGNATURE_LENGTH>(signature_count, "not enough bytes for signatures")?;
    Ok(signatures.into_iter().map(SignatureBytes::new).collect())
}

fn decode_message(decoder: &mut Decoder<'_>, signatures: Vec<SignatureBytes>) -> Result<VersionedTransaction> {
    let version = decoder.peek_u8("message is empty")?;
    if version & MESSAGE_VERSION_PREFIX == 0 {
        let message = decode_message_body(decoder)?;
        validate_message(&message, &[], signatures.len())?;
        return Ok(VersionedTransaction::Legacy { signatures, message });
    }

    decoder.read_u8("message is empty")?;
    match version & !MESSAGE_VERSION_PREFIX {
        0 => {
            let message = decode_message_body(decoder)?;
            let address_table_lookups = decode_address_table_lookups(decoder)?;
            validate_message(&message, &address_table_lookups, signatures.len())?;
            Ok(VersionedTransaction::V0 {
                signatures,
                message: VersionedMessageV0 { message, address_table_lookups },
            })
        }
        version => Err(invalid_transaction(format!("unsupported message version: {version}"))),
    }
}

fn validate_message(message: &Message, address_table_lookups: &[MessageAddressTableLookup], signature_count: usize) -> Result<()> {
    if signature_count != message.header.num_required_signatures as usize {
        return Err(invalid_transaction("signature count does not match the required-signature count"));
    }
    if address_table_lookups
        .iter()
        .any(|lookup| lookup.writable_indexes.is_empty() && lookup.readonly_indexes.is_empty())
    {
        return Err(invalid_transaction("address lookup table loads no account"));
    }
    let loaded_key_count: usize = address_table_lookups
        .iter()
        .map(|lookup| lookup.writable_indexes.len().saturating_add(lookup.readonly_indexes.len()))
        .sum();
    let total_key_count = message.account_keys.len().saturating_add(loaded_key_count);
    if total_key_count > MAX_ACCOUNT_KEYS {
        return Err(invalid_transaction("account count exceeds 256"));
    }
    validate_instructions(&message.instructions, message.account_keys.len(), total_key_count)
}

fn decode_v1_transaction(decoder: &mut Decoder<'_>) -> Result<VersionedTransaction> {
    let message = decode_v1_message(decoder)?;
    let signature_count = message.message.header.num_required_signatures as usize;
    let signatures = decoder.read_arrays::<SIGNATURE_LENGTH>(signature_count, "not enough bytes for V1 signatures")?;
    Ok(VersionedTransaction::V1 {
        signatures: signatures.into_iter().map(SignatureBytes::new).collect(),
        message,
    })
}

fn decode_v1_message(decoder: &mut Decoder<'_>) -> Result<VersionedMessageV1> {
    let prefix = decoder.read_u8("message is empty")?;
    if prefix != MESSAGE_V1_PREFIX {
        return Err(invalid_transaction("invalid V1 message prefix"));
    }
    let header = MessageHeader {
        num_required_signatures: decoder.read_u8("missing V1 required-signature count")?,
        num_readonly_signed_accounts: decoder.read_u8("missing V1 read-only signer count")?,
        num_readonly_unsigned_accounts: decoder.read_u8("missing V1 read-only account count")?,
    };
    let mask = decoder.read_u32("missing V1 transaction config mask")?;
    if mask & !V1_CONFIG_MASK != 0 || TransactionConfig::has_invalid_priority_fee(mask) {
        return Err(invalid_transaction("invalid V1 transaction config mask"));
    }
    let recent_blockhash = decoder.read_array("missing V1 lifetime specifier")?;
    let instruction_count = decoder.read_u8("missing V1 instruction count")? as usize;
    let account_count = decoder.read_u8("missing V1 account count")? as usize;
    let account_keys = decoder.read_arrays::<PUBKEY_LENGTH>(account_count, "not enough bytes for V1 accounts")?;
    let account_keys = account_keys.into_iter().map(Pubkey::new).collect::<Vec<_>>();
    let config = TransactionConfig {
        priority_fee: TransactionConfig::has_priority_fee(mask).then(|| decoder.read_u64("missing V1 priority fee")).transpose()?,
        compute_unit_limit: TransactionConfig::has_compute_unit_limit(mask)
            .then(|| decoder.read_u32("missing V1 compute unit limit"))
            .transpose()?,
        loaded_accounts_data_size_limit: TransactionConfig::has_loaded_accounts_data_size_limit(mask)
            .then(|| decoder.read_u32("missing V1 loaded-accounts data-size limit"))
            .transpose()?,
        heap_size: TransactionConfig::has_heap_size(mask).then(|| decoder.read_u32("missing V1 heap size")).transpose()?,
    };
    if instruction_count.saturating_mul(4) > decoder.remaining() {
        return Err(invalid_transaction("V1 instruction count exceeds remaining bytes"));
    }
    let instruction_headers = (0..instruction_count)
        .map(|_| {
            Ok((
                decoder.read_u8("missing V1 instruction program ID index")?,
                decoder.read_u8("missing V1 instruction account count")? as usize,
                decoder.read_u16("missing V1 instruction data length")? as usize,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let instructions = instruction_headers
        .into_iter()
        .map(|(program_id_index, account_count, data_length)| {
            let accounts = decoder.read_bytes(account_count, "not enough V1 instruction account indexes")?.to_vec();
            let data = decoder.read_bytes(data_length, "not enough V1 instruction data")?.to_vec();
            Ok(CompiledInstruction { program_id_index, accounts, data })
        })
        .collect::<Result<Vec<_>>>()?;
    let message = VersionedMessageV1 {
        message: Message {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        },
        config,
    };
    message.validate()?;
    Ok(message)
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
    header.validate(account_keys.len())?;

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
