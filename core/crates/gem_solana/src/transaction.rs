use gem_encoding::{decode_base64, encode_base64};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, SolanaInstruction, TransactionType};
use solana_primitives::{
    AccountMeta, AddressLookupTableAccount, Instruction, Pubkey, TransactionBuilder, VersionedTransaction,
    instructions::program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
};

pub trait VersionedTransactionExt {
    fn account_keys_mut(&mut self) -> &mut [Pubkey];

    fn recent_blockhash_mut(&mut self) -> &mut [u8; 32];

    fn memo(&self) -> Option<String>;

    fn simple_transfer(&self, signer: &Pubkey) -> Option<SolanaTransfer>;

    fn transaction_type(&self) -> TransactionType;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolanaTransfer {
    pub asset_id: AssetId,
    pub recipient: String,
    pub value: BigUint,
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

    fn simple_transfer(&self, signer: &Pubkey) -> Option<SolanaTransfer> {
        decode_transfer(self, Some(signer))
    }

    fn transaction_type(&self) -> TransactionType {
        match decode_transfer(self, None) {
            Some(_) => TransactionType::Transfer,
            None => TransactionType::SmartContractCall,
        }
    }
}

fn decode_transfer(transaction: &VersionedTransaction, signer: Option<&Pubkey>) -> Option<SolanaTransfer> {
    let account_keys = transaction.account_keys();
    let mut transfer = None;

    for instruction in transaction.instructions() {
        let program = account_keys.get(instruction.program_id_index as usize)?.to_base58();
        let decoded = match program.as_str() {
            SYSTEM_PROGRAM_ID => Some(system_transfer(instruction, account_keys, signer)?),
            TOKEN_PROGRAM_ID | TOKEN_2022_PROGRAM_ID => Some(token_transfer(transaction, instruction, account_keys, signer)?),
            ASSOCIATED_TOKEN_PROGRAM_ID | MEMO_PROGRAM_ID | COMPUTE_BUDGET_PROGRAM_ID => None,
            _ => return None,
        };
        if let Some(decoded) = decoded
            && transfer.replace(decoded).is_some()
        {
            return None;
        }
    }

    transfer
}

fn authorized(account: &Pubkey, signer: Option<&Pubkey>) -> bool {
    match signer {
        Some(signer) => account == signer,
        None => true,
    }
}

fn system_transfer(instruction: &solana_primitives::CompiledInstruction, account_keys: &[Pubkey], signer: Option<&Pubkey>) -> Option<SolanaTransfer> {
    let data: &[u8; 12] = instruction.data.as_slice().try_into().ok()?;
    (data[..4] == 2u32.to_le_bytes()).then_some(())?;
    authorized(instruction_account(instruction, account_keys, 0)?, signer).then_some(())?;
    let recipient = instruction_account(instruction, account_keys, 1)?;
    Some(SolanaTransfer {
        asset_id: AssetId::from_chain(Chain::Solana),
        recipient: recipient.to_base58(),
        value: u64::from_le_bytes(data[4..].try_into().ok()?).into(),
    })
}

fn token_transfer(transaction: &VersionedTransaction, instruction: &solana_primitives::CompiledInstruction, account_keys: &[Pubkey], signer: Option<&Pubkey>) -> Option<SolanaTransfer> {
    let data: &[u8; 10] = instruction.data.as_slice().try_into().ok()?;
    (data[0] == 12).then_some(())?;
    authorized(instruction_account(instruction, account_keys, 3)?, signer).then_some(())?;
    let mint = instruction_account(instruction, account_keys, 1)?;
    let token_account = instruction_account(instruction, account_keys, 2)?;
    let recipient = associated_token_owner(transaction, token_account).unwrap_or(token_account);
    Some(SolanaTransfer {
        asset_id: AssetId::from_token(Chain::Solana, &mint.to_base58()),
        recipient: recipient.to_base58(),
        value: u64::from_le_bytes(data[1..9].try_into().ok()?).into(),
    })
}

fn associated_token_owner<'a>(transaction: &'a VersionedTransaction, token_account: &Pubkey) -> Option<&'a Pubkey> {
    let account_keys = transaction.account_keys();
    transaction.instructions().iter().find_map(|instruction| {
        let program = account_keys.get(instruction.program_id_index as usize)?;
        let creates_account = instruction.data.is_empty() || instruction.data.as_slice() == [1];
        if program.to_base58() != ASSOCIATED_TOKEN_PROGRAM_ID || !creates_account || instruction_account(instruction, account_keys, 1)? != token_account {
            return None;
        }
        instruction_account(instruction, account_keys, 2)
    })
}

fn instruction_account<'a>(instruction: &solana_primitives::CompiledInstruction, account_keys: &'a [Pubkey], position: usize) -> Option<&'a Pubkey> {
    account_keys.get(*instruction.accounts.get(position)? as usize)
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
    use crate::testkit::{mock_transaction, mock_transaction_with_accounts};

    #[test]
    fn test_try_decode_blockhash() {
        assert!(try_decode_blockhash("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").is_some());
        assert!(try_decode_blockhash("invalid blockhash").is_none());
        assert!(try_decode_blockhash("1111111111111111111111111111111").is_none());
    }

    #[test]
    fn test_transaction_type() {
        let payer = Pubkey::new([1; 32]);
        let source = Pubkey::new([2; 32]);
        let mint = Pubkey::new([3; 32]);
        let destination = Pubkey::new([4; 32]);
        let account_keys = vec![
            payer,
            source,
            mint,
            destination,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
            Pubkey::from_base58(MEMO_PROGRAM_ID).unwrap(),
        ];
        let mut transfer_data = vec![12];
        transfer_data.extend_from_slice(&19_000_000u64.to_le_bytes());
        transfer_data.push(6);
        let mut legacy_transfer_data = vec![3];
        legacy_transfer_data.extend_from_slice(&19_000_000u64.to_le_bytes());
        let transfer_instruction = solana_primitives::CompiledInstruction {
            program_id_index: 4,
            accounts: vec![1, 2, 3, 0],
            data: transfer_data,
        };
        let memo_instruction = solana_primitives::CompiledInstruction {
            program_id_index: 5,
            accounts: vec![],
            data: b"payment memo".to_vec(),
        };
        let transfer = mock_transaction_with_accounts(account_keys.clone(), vec![transfer_instruction.clone(), memo_instruction]);
        let truncated_transfer = mock_transaction(&[(TOKEN_PROGRAM_ID, vec![12])]);
        let legacy_transfer = mock_transaction_with_accounts(
            account_keys.clone(),
            vec![solana_primitives::CompiledInstruction {
                program_id_index: 4,
                accounts: vec![1, 3, 0],
                data: legacy_transfer_data,
            }],
        );
        let ambiguous = mock_transaction_with_accounts(account_keys, vec![transfer_instruction.clone(), transfer_instruction]);
        let contract_call = mock_transaction(&[("BPFLoaderUpgradeab1e11111111111111111111111", vec![1])]);

        assert_eq!(transfer.transaction_type(), TransactionType::Transfer);
        assert_eq!(transfer.memo().as_deref(), Some("payment memo"));
        assert_eq!(truncated_transfer.transaction_type(), TransactionType::SmartContractCall);
        assert_eq!(legacy_transfer.transaction_type(), TransactionType::SmartContractCall);
        assert_eq!(ambiguous.transaction_type(), TransactionType::SmartContractCall);
        assert_eq!(contract_call.transaction_type(), TransactionType::SmartContractCall);
    }

    #[test]
    fn test_simple_token_transfer() {
        let payer = Pubkey::new([1; 32]);
        let source = Pubkey::new([2; 32]);
        let mint = Pubkey::new([3; 32]);
        let token_account = Pubkey::new([4; 32]);
        let recipient = Pubkey::new([5; 32]);
        let account_keys = vec![
            payer,
            source,
            mint,
            token_account,
            recipient,
            Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap(),
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        ];
        let mut transfer_data = vec![12];
        transfer_data.extend_from_slice(&19_000_000u64.to_le_bytes());
        transfer_data.push(6);
        let instructions = vec![
            solana_primitives::CompiledInstruction {
                program_id_index: 5,
                accounts: vec![0, 3, 4, 2],
                data: vec![1],
            },
            solana_primitives::CompiledInstruction {
                program_id_index: 6,
                accounts: vec![1, 2, 3, 0],
                data: transfer_data,
            },
        ];
        let transaction = mock_transaction_with_accounts(account_keys.clone(), instructions.clone());
        let ambiguous = mock_transaction_with_accounts(account_keys, vec![instructions[1].clone(), instructions[1].clone()]);

        let transfer = transaction.simple_transfer(&payer).unwrap();
        assert_eq!(transfer.asset_id, AssetId::from_token(Chain::Solana, &mint.to_base58()));
        assert_eq!(transfer.recipient, recipient.to_base58());
        assert_eq!(transfer.value, BigUint::from(19_000_000u64));
        assert_eq!(ambiguous.simple_transfer(&payer), None);
        assert_eq!(transaction.simple_transfer(&Pubkey::new([9; 32])), None);
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
