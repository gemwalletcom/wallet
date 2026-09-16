use crate::{
    instructions::program_ids::memo_program,
    types::{AccountMeta, Instruction, Pubkey},
};

pub fn memo(memo_text: &str, signers: &[&Pubkey]) -> Instruction {
    let account_metas = signers
        .iter()
        .map(|signer| AccountMeta {
            pubkey: *(*signer),
            is_signer: true,
            is_writable: false,
        })
        .collect::<Vec<AccountMeta>>();

    Instruction {
        program_id: memo_program(),
        accounts: account_metas,
        data: memo_text.as_bytes().to_vec(),
    }
}
