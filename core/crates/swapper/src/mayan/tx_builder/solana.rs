use std::fmt::Display;

use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, AccountMeta, Instruction, Pubkey, SolanaAddress,
    associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id, is_create_account_data},
    compute_budget,
    instructions::program_ids,
    token, wrapped_sol,
};

use crate::{
    SwapperError,
    jupiter::{RouteDestination, SwapInstructions},
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
    destination_account: &Pubkey,
    from_token_contract: &str,
    amount_in64: &str,
) -> Result<Vec<String>, SwapperError> {
    let SolanaClientSwap {
        compute_budget_instructions,
        setup_instructions,
        swap_instruction,
        cleanup_instruction,
        address_lookup_table_addresses,
    } = swap;
    let amount = amount_in64.parse::<u64>()?;
    let is_native = from_token_contract == EVM_ZERO_ADDRESS;
    let input_mint = if is_native {
        wrapped_sol::wrapped_sol_mint()
    } else {
        SolanaAddress::parse(from_token_contract).map_err(solana_error)?.into()
    };
    let client_swap = SwapInstructions::new(compute_budget_instructions.unwrap_or_default(), setup_instructions.unwrap_or_default(), swap_instruction, cleanup_instruction)?;
    client_swap.verify(user, &input_mint, amount, RouteDestination::Account(*destination_account))?;

    let wraps_native_sol = client_swap.wraps_native_sol(user)?;
    instructions.extend(client_swap.compute_budget);
    if is_native && !wraps_native_sol {
        instructions.extend(wrap_native_sol_instructions(user, amount)?);
    }
    for instruction in client_swap.setup {
        instructions.push(wrap_instruction_in_cpi_proxy(with_setup_payer(instruction, relayer))?);
    }
    instructions.push(client_swap.swap);
    if let Some(cleanup) = client_swap.cleanup {
        instructions.push(wrap_instruction_in_cpi_proxy(cleanup)?);
    }
    Ok(address_lookup_table_addresses)
}

fn wrap_native_sol_instructions(owner: &Pubkey, amount: u64) -> Result<Vec<Instruction>, SwapperError> {
    wrapped_sol::wrap_instructions(owner, amount).map_err(solana_error)?.into_iter().map(wrap_instruction_in_cpi_proxy).collect()
}

fn with_setup_payer(mut instruction: Instruction, payer: &Pubkey) -> Instruction {
    if instruction.program_id.to_base58() == ASSOCIATED_TOKEN_ACCOUNT_PROGRAM
        && is_create_account_data(&instruction.data)
        && let Some(funding) = instruction.accounts.first_mut()
    {
        funding.pubkey = *payer;
    }
    instruction
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
    use std::sync::Arc;

    use gem_client::testkit::MockClient;
    use primitives::{
        AssetId, Chain,
        asset_constants::SOLANA_USDT_TOKEN_ID,
        testkit::signer_mock::{TEST_PRIVATE_KEY_SOLANA_ADDRESS, TEST_SOLANA_SENDER},
    };

    use super::*;
    use crate::{
        Swapper,
        alien::mock::ProviderMock,
        contract_check::verify_solana_programs,
        mayan::{client::MayanClient, provider::Mayan},
        solana_dex::SolanaTransaction,
    };

    const USDT_USER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
    const USDT_LEDGER_ACCOUNT: &str = "5SGghFsD6otbtctbotPveZvUWBJGDz8CGFTCUjp5dNcB";
    const USDT_AMOUNT: &str = "5000000";
    const SOL_LEDGER_ACCOUNT: &str = "9HrGRWMADkN3Sdh3pUpcPuRbX9cyJztokCaS9PfmcnsX";
    const SOL_AMOUNT: &str = "2000000000";

    #[test]
    fn test_client_swap_through_jupiter_passes_the_contract_check() {
        let mayan = Mayan::with_clients(MayanClient::new(MockClient::new()), MayanClient::new(MockClient::new()), Arc::new(ProviderMock::new("{}".to_string())));
        let targets = mayan.contracts(&AssetId::from_chain(Chain::Solana)).targets;

        let user = Pubkey::from_base58(USDT_USER).unwrap();
        let ledger_account = Pubkey::from_base58(USDT_LEDGER_ACCOUNT).unwrap();
        let mut token = Vec::new();
        append_client_swap_instructions(&mut token, SolanaClientSwap::mock_usdt(), &user, &user, &ledger_account, SOLANA_USDT_TOKEN_ID, USDT_AMOUNT).unwrap();
        verify_solana_programs(&SolanaTransaction::new(token, vec![]).encode(USDT_USER, &[]).unwrap().data, &targets).unwrap();

        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let ledger_account = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let mut coin = Vec::new();
        append_client_swap_instructions(&mut coin, SolanaClientSwap::mock_sol(), &user, &user, &ledger_account, EVM_ZERO_ADDRESS, SOL_AMOUNT).unwrap();
        assert_eq!(coin.len(), 8, "the reply wraps the coin itself, so no second wrap is added");
    }

    #[test]
    fn test_client_swap_pays_into_the_bridge() {
        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let ledger_account = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let stranger = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();

        for (destination, from_token, amount, reason) in [
            (stranger, EVM_ZERO_ADDRESS, SOL_AMOUNT, "a swap that pays into another account than the bridge's"),
            (ledger_account, SOLANA_USDT_TOKEN_ID, SOL_AMOUNT, "a swap that spends another token than quoted"),
            (ledger_account, EVM_ZERO_ADDRESS, "1999999999", "a swap that spends more than quoted"),
        ] {
            assert_eq!(
                append_client_swap_instructions(&mut Vec::new(), SolanaClientSwap::mock_sol(), &user, &user, &destination, from_token, amount).err(),
                Some(SwapperError::InvalidRoute),
                "{reason}"
            );
        }
    }

    #[test]
    fn test_client_swap_setup_is_paid_by_the_relayer() {
        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let relayer = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();
        let mut instructions = Vec::new();
        append_client_swap_instructions(
            &mut instructions,
            SolanaClientSwap::mock_sol(),
            &user,
            &relayer,
            &Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap(),
            EVM_ZERO_ADDRESS,
            SOL_AMOUNT,
        )
        .unwrap();

        let payers: Vec<Pubkey> = instructions
            .iter()
            .filter(|instruction| instruction.accounts.first().is_some_and(|program| program.pubkey.to_base58() == ASSOCIATED_TOKEN_ACCOUNT_PROGRAM))
            .map(|instruction| instruction.accounts[1].pubkey)
            .collect();
        assert_eq!(payers, vec![relayer, relayer]);
    }

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
