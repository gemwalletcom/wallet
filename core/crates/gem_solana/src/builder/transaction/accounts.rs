use std::collections::HashMap;

use crate::{AccountMeta, CompiledInstruction, Instruction, MessageHeader, Pubkey, Result, SolanaError, types::MAX_ACCOUNT_KEYS};

#[derive(Default)]
pub(crate) struct AccountBuckets {
    writable_signers: Vec<Pubkey>,
    readonly_signers: Vec<Pubkey>,
    writable_accounts: Vec<Pubkey>,
    readonly_accounts: Vec<Pubkey>,
}

impl AccountBuckets {
    pub(crate) fn from_accounts(accounts: Vec<AccountMeta>) -> Self {
        let mut buckets = Self::default();
        for account in accounts {
            buckets.push(account);
        }
        buckets
    }

    pub(super) fn push(&mut self, account: AccountMeta) {
        match (account.is_signer, account.is_writable) {
            (true, true) => self.writable_signers.push(account.pubkey),
            (true, false) => self.readonly_signers.push(account.pubkey),
            (false, true) => self.writable_accounts.push(account.pubkey),
            (false, false) => self.readonly_accounts.push(account.pubkey),
        }
    }

    pub(super) fn sort(&mut self) {
        self.writable_signers.sort();
        self.readonly_signers.sort();
        self.writable_accounts.sort();
        self.readonly_accounts.sort();
    }

    pub(super) fn header(&self) -> Result<MessageHeader> {
        let count = |len: usize| u8::try_from(len).map_err(|_| SolanaError::InvalidMessage);
        Ok(MessageHeader {
            num_required_signatures: count(self.writable_signers.len() + self.readonly_signers.len())?,
            num_readonly_signed_accounts: count(self.readonly_signers.len())?,
            num_readonly_unsigned_accounts: count(self.readonly_accounts.len())?,
        })
    }

    pub(super) fn account_keys(&self, fee_payer: Pubkey) -> Vec<Pubkey> {
        let mut account_keys = Vec::with_capacity(self.writable_signers.len() + self.readonly_signers.len() + self.writable_accounts.len() + self.readonly_accounts.len());
        account_keys.push(fee_payer);
        account_keys.extend(self.writable_signers.iter().copied().filter(|pubkey| *pubkey != fee_payer));
        account_keys.extend(self.readonly_signers.iter().copied());
        account_keys.extend(self.writable_accounts.iter().copied());
        account_keys.extend(self.readonly_accounts.iter().copied());
        account_keys
    }
}

pub(crate) fn collect_accounts(fee_payer: Pubkey, accounts: impl IntoIterator<Item = AccountMeta>) -> Vec<AccountMeta> {
    let mut merged = vec![AccountMeta::new_signer_writable(fee_payer)];
    let mut indexes = HashMap::from([(fee_payer, 0)]);

    for account in accounts {
        match indexes.get(&account.pubkey).copied() {
            Some(index) => {
                merged[index].is_signer |= account.is_signer;
                merged[index].is_writable |= account.is_writable;
            }
            None => {
                indexes.insert(account.pubkey, merged.len());
                merged.push(account);
            }
        }
    }
    merged
}

pub(super) fn index_accounts(account_keys: &[Pubkey]) -> Result<HashMap<Pubkey, u8>> {
    if account_keys.len() > MAX_ACCOUNT_KEYS {
        return Err(SolanaError::InvalidMessage);
    }
    Ok(account_keys.iter().enumerate().map(|(index, pubkey)| (*pubkey, index as u8)).collect())
}

pub(super) fn compile_instructions(instructions: &[Instruction], account_indexes: &HashMap<Pubkey, u8>) -> Result<Vec<CompiledInstruction>> {
    instructions
        .iter()
        .map(|instruction| {
            let program_id_index = account_index(account_indexes, instruction.program_id)?;
            let accounts = instruction.accounts.iter().map(|account| account_index(account_indexes, account.pubkey)).collect::<Result<Vec<_>>>()?;
            Ok(CompiledInstruction {
                program_id_index,
                accounts,
                data: instruction.data.clone(),
            })
        })
        .collect()
}

fn account_index(account_indexes: &HashMap<Pubkey, u8>, pubkey: Pubkey) -> Result<u8> {
    account_indexes.get(&pubkey).copied().ok_or(SolanaError::InvalidMessage)
}
