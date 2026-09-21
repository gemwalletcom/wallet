use crate::{
    instructions::program_ids::token_program,
    types::{AccountMeta, Instruction, Pubkey},
};

const TRANSFER_DISCRIMINANT: u8 = 3;
const TRANSFER_CHECKED_DISCRIMINANT: u8 = 12;
const SYNC_NATIVE_DISCRIMINANT: u8 = 17;

pub fn transfer(source: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64) -> Instruction {
    transfer_with_program_id(source, destination, owner, amount, &token_program())
}

fn transfer_with_program_id(source: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64, token_program_id: &Pubkey) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(TRANSFER_DISCRIMINANT);
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *token_program_id,
        accounts: vec![AccountMeta::new_writable(*source), AccountMeta::new_writable(*destination), AccountMeta::new_signer(*owner)],
        data,
    }
}

pub fn transfer_checked(source: &Pubkey, mint: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64, decimals: u8) -> Instruction {
    transfer_checked_with_program_id(source, mint, destination, owner, amount, decimals, &token_program())
}

pub fn transfer_checked_with_program_id(source: &Pubkey, mint: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64, decimals: u8, token_program_id: &Pubkey) -> Instruction {
    let mut data = Vec::with_capacity(10);
    data.push(TRANSFER_CHECKED_DISCRIMINANT);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);

    Instruction {
        program_id: *token_program_id,
        accounts: vec![AccountMeta::new_writable(*source), AccountMeta::new_readonly(*mint), AccountMeta::new_writable(*destination), AccountMeta::new_signer(*owner)],
        data,
    }
}

pub fn sync_native(account: &Pubkey) -> Instruction {
    Instruction {
        program_id: token_program(),
        accounts: vec![AccountMeta::new_writable(*account)],
        data: vec![SYNC_NATIVE_DISCRIMINANT],
    }
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use super::*;
    use crate::instructions::program_ids::token_2022_program;

    #[test]
    fn test_token_instruction_wire_formats() {
        let source = Pubkey::new([1; 32]);
        let mint = Pubkey::new([2; 32]);
        let destination = Pubkey::new([3; 32]);
        let owner = Pubkey::new([4; 32]);

        let instruction = transfer(&source, &destination, &owner, 1_000);
        assert_eq!(instruction.program_id, token_program());
        assert_eq!(instruction.data, hex!("03e803000000000000"));

        let instruction = transfer_checked_with_program_id(&source, &mint, &destination, &owner, 1_000, 6, &token_2022_program());
        assert_eq!(instruction.program_id, token_2022_program());
        assert_eq!(instruction.data, hex!("0ce80300000000000006"));

        let instruction = sync_native(&source);
        assert_eq!(instruction.accounts, vec![AccountMeta::new_writable(source)]);
        assert_eq!(instruction.data, [17]);
    }
}
