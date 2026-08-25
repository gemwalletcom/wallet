use gem_encoding::{decode_base64, encode_base64};
use primitives::{SolanaInstruction, TransactionType};
use solana_primitives::{
    AccountMeta, AddressLookupTableAccount, Instruction, Pubkey, TransactionBuilder, VersionedTransaction,
    instructions::program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
};

pub trait VersionedTransactionExt {
    fn account_keys_mut(&mut self) -> &mut [Pubkey];

    fn recent_blockhash_mut(&mut self) -> &mut [u8; 32];

    fn memo(&self) -> Option<String>;

    fn transaction_type(&self) -> TransactionType;
}

impl VersionedTransactionExt for VersionedTransaction {
    fn account_keys_mut(&mut self) -> &mut [Pubkey] {
        match self {
            Self::Legacy { message, .. } => &mut message.account_keys,
            Self::V0 { message, .. } => &mut message.account_keys,
        }
    }

    fn recent_blockhash_mut(&mut self) -> &mut [u8; 32] {
        match self {
            Self::Legacy { message, .. } => &mut message.recent_blockhash,
            Self::V0 { message, .. } => &mut message.recent_blockhash,
        }
    }

    fn memo(&self) -> Option<String> {
        let account_keys = self.account_keys();
        self.instructions().iter().find_map(|instruction| {
            let program = account_keys.get(instruction.program_id_index as usize)?;
            if program.to_base58() != MEMO_PROGRAM_ID {
                return None;
            }
            String::from_utf8(instruction.data.clone()).ok().filter(|memo| !memo.is_empty())
        })
    }

    fn transaction_type(&self) -> TransactionType {
        let account_keys = self.account_keys();
        let mut has_transfer = false;

        for instruction in self.instructions() {
            let Some(program) = account_keys.get(instruction.program_id_index as usize).map(Pubkey::to_base58) else {
                return TransactionType::SmartContractCall;
            };
            match program.as_str() {
                SYSTEM_PROGRAM_ID => match instruction.data.get(..4) {
                    Some(data) if data == 2u32.to_le_bytes() => has_transfer = true,
                    _ => return TransactionType::SmartContractCall,
                },
                TOKEN_PROGRAM_ID | TOKEN_2022_PROGRAM_ID => match instruction.data.first() {
                    Some(3 | 12) => has_transfer = true,
                    _ => return TransactionType::SmartContractCall,
                },
                ASSOCIATED_TOKEN_PROGRAM_ID | MEMO_PROGRAM_ID | COMPUTE_BUDGET_PROGRAM_ID => {}
                _ => return TransactionType::SmartContractCall,
            }
        }

        if has_transfer { TransactionType::Transfer } else { TransactionType::SmartContractCall }
    }
}

pub fn try_decode_transaction(transaction_base64: &str) -> Option<VersionedTransaction> {
    let data = decode_base64(transaction_base64).ok()?;
    try_decode_transaction_bytes(&data)
}

pub(crate) fn try_decode_transaction_bytes(transaction: &[u8]) -> Option<VersionedTransaction> {
    let decoded = VersionedTransaction::deserialize_with_version(transaction).ok()?;
    (decoded.serialize().ok()? == transaction).then_some(decoded)
}

#[cfg(feature = "signer")]
pub(crate) fn is_transaction_bytes(transaction: &[u8]) -> bool {
    try_decode_transaction_bytes(transaction).is_some() || try_decode_transaction_message(transaction).is_some()
}

#[cfg(feature = "signer")]
fn try_decode_transaction_message(message: &[u8]) -> Option<VersionedTransaction> {
    let mut transaction = Vec::with_capacity(message.len() + 1);
    transaction.push(0);
    transaction.extend_from_slice(message);

    let decoded = VersionedTransaction::deserialize_with_version(&transaction).ok()?;
    (decoded.serialize_message().ok()? == message).then_some(decoded)
}

pub fn decode_transaction(transaction_base64: &str) -> Result<VersionedTransaction, String> {
    try_decode_transaction(transaction_base64).ok_or_else(|| "failed to decode transaction".to_string())
}

pub fn try_decode_blockhash(blockhash: &str) -> Option<[u8; 32]> {
    bs58::decode(blockhash).into_vec().ok()?.try_into().ok()
}

pub fn encode_v0_transaction(payer: Pubkey, recent_blockhash: &str, instructions: &[Instruction], lookup_tables: &[AddressLookupTableAccount]) -> Result<String, String> {
    let recent_blockhash = try_decode_blockhash(recent_blockhash).ok_or_else(|| "Invalid Solana blockhash".to_string())?;
    let transaction = TransactionBuilder::build_v0_transaction(payer, recent_blockhash, instructions, lookup_tables).map_err(|err| format!("Solana transaction error: {err}"))?;
    let bytes = transaction.serialize().map_err(|err| format!("Solana transaction error: {err}"))?;
    Ok(encode_base64(&bytes))
}

pub fn instruction_from_primitive(instruction: SolanaInstruction) -> Result<Instruction, String> {
    let program_id = Pubkey::from_base58(&instruction.program_id).map_err(|err| format!("Invalid Solana address {}: {err}", instruction.program_id))?;
    let accounts = instruction
        .accounts
        .into_iter()
        .map(|account| {
            Ok(AccountMeta {
                pubkey: Pubkey::from_base58(&account.pubkey).map_err(|err| format!("Invalid Solana address {}: {err}", account.pubkey))?,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Instruction {
        program_id,
        accounts,
        data: decode_base64(&instruction.data).map_err(|err| err.to_string())?,
    })
}

pub fn instructions_from_primitives(instructions: Vec<SolanaInstruction>) -> Result<Vec<Instruction>, String> {
    instructions.into_iter().map(instruction_from_primitive).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "signer")]
    use crate::signer::testkit::{SINGLE_SIG_TX, mock_legacy_transaction};
    use crate::testkit::mock_transaction;

    #[test]
    fn test_try_decode_blockhash() {
        assert!(try_decode_blockhash("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").is_some());
        assert!(try_decode_blockhash("invalid blockhash").is_none());
        assert!(try_decode_blockhash("1111111111111111111111111111111").is_none());
    }

    #[test]
    fn test_transaction_type() {
        let transfer = mock_transaction(&[
            (ASSOCIATED_TOKEN_PROGRAM_ID, vec![]),
            (TOKEN_PROGRAM_ID, vec![12]),
            (MEMO_PROGRAM_ID, b"payment memo".to_vec()),
        ]);
        let contract_call = mock_transaction(&[("BPFLoaderUpgradeab1e11111111111111111111111", vec![1])]);

        assert_eq!(transfer.transaction_type(), TransactionType::Transfer);
        assert_eq!(transfer.memo().as_deref(), Some("payment memo"));
        assert_eq!(contract_call.transaction_type(), TransactionType::SmartContractCall);
    }

    #[cfg(feature = "signer")]
    #[test]
    fn test_is_transaction_bytes() {
        let full_transaction = gem_encoding::decode_base64(SINGLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&full_transaction).unwrap();
        let mut v0_message = transaction.serialize_message().unwrap();
        let mut transaction_with_trailing_byte = full_transaction.clone();

        assert!(is_transaction_bytes(&full_transaction));
        assert!(is_transaction_bytes(&v0_message));
        assert!(is_transaction_bytes(&mock_legacy_transaction().serialize_message().unwrap()));

        transaction_with_trailing_byte.push(0);
        v0_message.push(0);
        assert!(!is_transaction_bytes(&transaction_with_trailing_byte));
        assert!(!is_transaction_bytes(&v0_message));
        assert!(!is_transaction_bytes(b"hello"));
    }
}
