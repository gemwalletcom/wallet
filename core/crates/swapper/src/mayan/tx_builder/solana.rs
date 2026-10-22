use std::fmt::Display;

use gem_encoding::decode_base64;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, AccountMeta, Base64InstructionData, Instruction, Pubkey, SYSTEM_PROGRAM_ID, SolanaAddress,
    associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id},
    compute_budget, instruction_from_primitive,
    instructions::program_ids,
    instructions_from_primitives, token, wrapped_sol,
};
use primitives::SolanaInstruction;

use crate::{
    SwapperError,
    mayan::{constants::MAYAN_CPI_PROXY_PROGRAM_ID, model::SolanaClientSwap},
};

pub(in crate::mayan::tx_builder) struct SolanaLedgerDeposit<'a> {
    pub user: &'a Pubkey,
    pub relayer: &'a Pubkey,
    pub ledger: &'a Pubkey,
    pub ledger_account: &'a Pubkey,
    pub mint: &'a Pubkey,
    pub amount: u64,
    pub suggested_priority_fee: Option<u64>,
}

pub(in crate::mayan::tx_builder) fn append_ledger_deposit_instructions(instructions: &mut Vec<Instruction>, deposit: SolanaLedgerDeposit<'_>) -> Result<(), SwapperError> {
    if let Some(priority_fee) = deposit.suggested_priority_fee.filter(|&fee| fee > 0) {
        instructions.push(compute_budget::set_compute_unit_price(priority_fee));
    }

    instructions.push(wrap_instruction_in_cpi_proxy(create_associated_token_account_idempotent_with_address(
        deposit.relayer,
        deposit.ledger_account,
        deposit.ledger,
        deposit.mint,
        &program_ids::token_program(),
    ))?);

    let source_account = get_associated_token_address_with_program_id(deposit.user, deposit.mint, &program_ids::token_program()).map_err(solana_error)?;
    instructions.push(wrap_instruction_in_cpi_proxy(token::transfer(&source_account, deposit.ledger_account, deposit.user, deposit.amount))?);
    Ok(())
}

pub(in crate::mayan::tx_builder) fn append_client_swap_instructions(
    instructions: &mut Vec<Instruction>,
    swap: SolanaClientSwap,
    user: &Pubkey,
    relayer: &Pubkey,
    from_token_contract: &str,
    amount_in64: &str,
) -> Result<Vec<String>, SwapperError> {
    let setup = swap.setup_instructions.unwrap_or_default();
    let compute_budget = swap.compute_budget_instructions.unwrap_or_default();
    instructions.extend(instructions_from_primitives::<Base64InstructionData>(compute_budget).map_err(solana_error)?);
    if from_token_contract == EVM_ZERO_ADDRESS && !setup_wraps_native_sol(&setup, user)? {
        instructions.extend(wrap_native_sol_instructions(user, amount_in64.parse::<u64>()?)?);
    }
    instructions.extend(setup_instructions(setup, relayer)?);
    instructions.push(instruction_from_primitive::<Base64InstructionData>(swap.swap_instruction).map_err(solana_error)?);
    if let Some(cleanup_instruction) = swap.cleanup_instruction {
        instructions.push(wrap_instruction_in_cpi_proxy(instruction_from_primitive::<Base64InstructionData>(cleanup_instruction).map_err(solana_error)?)?);
    }
    Ok(swap.address_lookup_table_addresses)
}

fn setup_instructions(instructions: Vec<SolanaInstruction>, payer: &Pubkey) -> Result<Vec<Instruction>, SwapperError> {
    instructions
        .into_iter()
        .map(|instruction| {
            override_setup_payer(instruction, payer)
                .and_then(|instruction| instruction_from_primitive::<Base64InstructionData>(instruction).map_err(solana_error))
                .and_then(wrap_instruction_in_cpi_proxy)
        })
        .collect()
}

fn setup_wraps_native_sol(instructions: &[SolanaInstruction], owner: &Pubkey) -> Result<bool, SwapperError> {
    let wrapped_account = wrapped_sol::wrapped_sol_account(owner).map_err(solana_error)?;
    Ok(instructions.iter().any(|instruction| {
        instruction.program_id == SYSTEM_PROGRAM_ID && instruction.accounts.get(1).is_some_and(|account| account.pubkey == wrapped_account.to_string()) && decode_base64(&instruction.data).is_ok_and(|data| data.starts_with(&[2, 0, 0, 0]))
    }))
}

fn wrap_native_sol_instructions(owner: &Pubkey, amount: u64) -> Result<Vec<Instruction>, SwapperError> {
    wrapped_sol::wrap_instructions(owner, amount).map_err(solana_error)?.into_iter().map(wrap_instruction_in_cpi_proxy).collect()
}

fn override_setup_payer(mut instruction: SolanaInstruction, payer: &Pubkey) -> Result<SolanaInstruction, SwapperError> {
    if instruction.accounts.is_empty() {
        return Ok(instruction);
    }
    let data = decode_base64(&instruction.data).map_err(solana_error)?;
    let should_override = match instruction.program_id.as_str() {
        SYSTEM_PROGRAM_ID => data.starts_with(&[0, 0, 0, 0]),
        ASSOCIATED_TOKEN_ACCOUNT_PROGRAM => data.is_empty() || data.as_slice() == [1],
        _ => false,
    };
    if should_override {
        instruction.accounts[0].pubkey = payer.to_string();
    }
    Ok(instruction)
}

pub(in crate::mayan::tx_builder) fn wrap_instruction_in_cpi_proxy(instruction: Instruction) -> Result<Instruction, SwapperError> {
    let mut accounts = Vec::with_capacity(instruction.accounts.len() + 1);
    accounts.push(AccountMeta::new_readonly(instruction.program_id));
    accounts.extend(instruction.accounts);
    Ok(Instruction {
        program_id: SolanaAddress::parse(MAYAN_CPI_PROXY_PROGRAM_ID).map_err(solana_error)?.into(),
        accounts,
        data: instruction.data,
    })
}

pub(in crate::mayan::tx_builder) fn solana_error(err: impl Display) -> SwapperError {
    SwapperError::transaction_error(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_instruction_in_cpi_proxy_uses_deployed_program_id() {
        let program_id = Pubkey::new([1; 32]);
        let account = Pubkey::new([2; 32]);
        let instruction = Instruction {
            program_id,
            accounts: vec![AccountMeta::new_readonly(account)],
            data: vec![1, 2, 3],
        };

        let wrapped = wrap_instruction_in_cpi_proxy(instruction).unwrap();

        assert_eq!(wrapped.program_id.to_string(), MAYAN_CPI_PROXY_PROGRAM_ID);
        assert_eq!(wrapped.accounts.len(), 2);
        assert_eq!(wrapped.accounts[0].pubkey, program_id);
        assert!(!wrapped.accounts[0].is_signer);
        assert!(!wrapped.accounts[0].is_writable);
        assert_eq!(wrapped.accounts[1].pubkey, account);
        assert!(!wrapped.accounts[1].is_signer);
        assert!(!wrapped.accounts[1].is_writable);
        assert_eq!(wrapped.data, vec![1, 2, 3]);
    }
}
