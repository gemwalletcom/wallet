use hex_lit::hex;

use crate::{
    instructions::program_ids::system_program,
    types::{AccountMeta, Instruction, Pubkey},
};

const TRANSFER_DISCRIMINANT: [u8; 4] = hex!("02000000");

pub fn transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&TRANSFER_DISCRIMINANT);
    data.extend_from_slice(&lamports.to_le_bytes());

    Instruction {
        program_id: system_program(),
        accounts: vec![AccountMeta::new_signer_writable(*from), AccountMeta::new_writable(*to)],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_wire_format() {
        let from = Pubkey::new([1; 32]);
        let to = Pubkey::new([2; 32]);
        let instruction = transfer(&from, &to, 1_000_000);

        assert_eq!(instruction.program_id, system_program());
        assert_eq!(instruction.accounts, vec![AccountMeta::new_signer_writable(from), AccountMeta::new_writable(to)]);
        assert_eq!(instruction.data, hex!("0200000040420f0000000000"));
    }
}
