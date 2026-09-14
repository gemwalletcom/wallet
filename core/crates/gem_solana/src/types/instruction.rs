use super::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    pub fn new(pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        Self { pubkey, is_signer, is_writable }
    }

    pub fn new_readonly(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, false)
    }

    pub fn new_signer(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, false)
    }

    pub fn new_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, true)
    }

    pub fn new_signer_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}
