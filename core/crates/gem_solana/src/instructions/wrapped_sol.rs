use primitives::decode_hex_literal;

use crate::{
    Result,
    instructions::{associated_token, program_ids::token_program, system, token},
    types::{Instruction, Pubkey},
};

pub const fn wrapped_sol_mint() -> Pubkey {
    Pubkey::new(const { decode_hex_literal("069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f00000000001") })
}

pub fn wrapped_sol_account(owner: &Pubkey) -> Result<Pubkey> {
    associated_token::get_associated_token_address_with_program_id(owner, &wrapped_sol_mint(), &token_program())
}

pub fn wrap_instructions(owner: &Pubkey, lamports: u64) -> Result<Vec<Instruction>> {
    let account = wrapped_sol_account(owner)?;
    Ok(vec![
        associated_token::create_associated_token_account_idempotent_with_address(owner, &account, owner, &wrapped_sol_mint(), &token_program()),
        system::transfer(owner, &account, lamports),
        token::sync_native(&account),
    ])
}

pub fn unwrap_instruction(owner: &Pubkey) -> Result<Instruction> {
    Ok(token::close_account(&wrapped_sol_account(owner)?, owner, owner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::{associated_token_program, system_program};

    #[test]
    fn test_wrap_instructions() {
        let owner = Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap();
        let account = wrapped_sol_account(&owner).unwrap();

        assert_eq!(
            wrap_instructions(&owner, 1_000).unwrap().iter().map(|instruction| instruction.program_id).collect::<Vec<_>>(),
            vec![associated_token_program(), system_program(), token_program()]
        );
        assert_eq!(wrap_instructions(&owner, 1_000).unwrap()[1], system::transfer(&owner, &account, 1_000));
        assert_eq!(unwrap_instruction(&owner).unwrap(), token::close_account(&account, &owner, &owner));
    }
}
