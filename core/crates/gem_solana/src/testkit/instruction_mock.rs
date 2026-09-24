use crate::{AccountMeta, CompiledInstruction, Instruction};

impl Instruction {
    pub(crate) fn mock(program_id_index: usize, account_indexes: &[u8], data_base58: &str, combined_accounts: &[AccountMeta]) -> Self {
        Self {
            program_id: combined_accounts[program_id_index].pubkey,
            accounts: account_indexes.iter().map(|index| combined_accounts[*index as usize].clone()).collect(),
            data: bs58::decode(data_base58).into_vec().unwrap(),
        }
    }
}

impl CompiledInstruction {
    pub(crate) fn mock(program_id_index: u8, accounts: Vec<u8>, data: Vec<u8>) -> Self {
        Self { program_id_index, accounts, data }
    }
}
