use crate::{AccountMeta, Instruction, Pubkey};

#[derive(Debug)]
pub struct InstructionBuilder {
    program_id: Pubkey,
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
}

impl InstructionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            accounts: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn account(mut self, pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        self.accounts.push(AccountMeta { pubkey, is_signer, is_writable });
        self
    }

    pub fn accounts(mut self, accounts: Vec<AccountMeta>) -> Self {
        self.accounts.extend(accounts);
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: self.accounts,
            data: self.data,
        }
    }
}
