use crate::{
    Result,
    instructions::program_ids::{associated_token_program, rent_sysvar, system_program},
    types::{AccountMeta, Instruction, Pubkey, find_program_address},
};

const CREATE_IDEMPOTENT_DISCRIMINANT: u8 = 1;

pub fn create_associated_token_account_idempotent(payer: &Pubkey, wallet: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Result<Instruction> {
    let associated_token_address = get_associated_token_address_with_program_id(wallet, mint, token_program)?;
    Ok(create_associated_token_account_idempotent_with_address(
        payer,
        &associated_token_address,
        wallet,
        mint,
        token_program,
    ))
}

pub fn create_associated_token_account_idempotent_with_address(
    payer: &Pubkey,
    associated_token_address: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: associated_token_program(),
        accounts: vec![
            AccountMeta::new_signer_writable(*payer),
            AccountMeta::new_writable(*associated_token_address),
            AccountMeta::new_readonly(*wallet),
            AccountMeta::new_readonly(*mint),
            AccountMeta::new_readonly(system_program()),
            AccountMeta::new_readonly(*token_program),
            AccountMeta::new_readonly(rent_sysvar()),
        ],
        data: vec![CREATE_IDEMPOTENT_DISCRIMINANT],
    }
}

pub fn get_associated_token_address_with_program_id(wallet: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Result<Pubkey> {
    let seeds = [wallet.as_bytes().as_slice(), token_program.as_bytes().as_slice(), mint.as_bytes().as_slice()];
    find_program_address(&associated_token_program(), &seeds).map(|(address, _)| address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::{token_2022_program, token_program};

    #[test]
    fn test_associated_token_instruction_and_address() {
        let payer = Pubkey::from_base58("3ECJhLBQ9DAuKBKNjQGLEk3YqoFcF1YvhdayQ2C96eXF").unwrap();
        let wallet = Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap();
        let mint = Pubkey::from_base58("7o36UsWR1JQLpZ9PE2gn9L4SQ69CNNiWAXd4Jt7rqz9Z").unwrap();

        let token_address = get_associated_token_address_with_program_id(&wallet, &mint, &token_program()).unwrap();
        let token_2022_address = get_associated_token_address_with_program_id(&wallet, &mint, &token_2022_program()).unwrap();
        assert_ne!(token_address, token_2022_address);

        let instruction = create_associated_token_account_idempotent(&payer, &wallet, &mint, &token_program()).unwrap();
        assert_eq!(instruction.program_id, associated_token_program());
        assert_eq!(instruction.accounts[1].pubkey, token_address);
        assert_eq!(instruction.accounts[5].pubkey, token_program());
        assert_eq!(instruction.data, [CREATE_IDEMPOTENT_DISCRIMINANT]);
    }
}
