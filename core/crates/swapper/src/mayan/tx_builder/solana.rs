use std::fmt::Display;

use gem_encoding::decode_base64;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::models::SolanaInstruction;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, AccountMeta, Base64InstructionData, COMPUTE_BUDGET_PROGRAM_ID, Instruction, JUPITER_PROGRAM_ID, Pubkey, SYSTEM_PROGRAM_ID, SolanaAddress, TOKEN_PROGRAM, TOKEN_PROGRAM_2022, anchor,
    associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id, is_create_account_data},
    compute_budget, decode_instruction, decode_instructions,
    instructions::program_ids,
    system, token, wrapped_sol,
};

use crate::{
    SwapperError,
    mayan::{constants::MAYAN_CPI_PROXY_PROGRAM_ID, model::SolanaClientSwap},
};

const MAX_SETUP_INSTRUCTIONS: usize = 6;
const MAX_COMPUTE_UNIT_PRICE: u64 = 100_000_000;
const JUPITER_DESTINATION_ACCOUNTS: [(&str, usize); 4] = [("route", 4), ("route_v2", 7), ("shared_accounts_route", 6), ("shared_accounts_route_v2", 5)];

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
    verify_client_swap(&swap, user, destination_account)?;
    let setup = swap.setup_instructions.unwrap_or_default();
    let compute_budget = swap.compute_budget_instructions.unwrap_or_default();
    instructions.extend(decode_instructions::<Base64InstructionData>(compute_budget).map_err(solana_error)?);
    if from_token_contract == EVM_ZERO_ADDRESS && !setup_wraps_native_sol(&setup, user)? {
        instructions.extend(wrap_native_sol_instructions(user, amount_in64.parse::<u64>()?)?);
    }
    instructions.extend(setup_instructions(setup, relayer)?);
    instructions.push(decode_instruction::<Base64InstructionData>(swap.swap_instruction).map_err(solana_error)?);
    if let Some(cleanup_instruction) = swap.cleanup_instruction {
        instructions.push(wrap_instruction_in_cpi_proxy(decode_instruction::<Base64InstructionData>(cleanup_instruction).map_err(solana_error)?)?);
    }
    Ok(swap.address_lookup_table_addresses)
}

fn setup_instructions(instructions: Vec<SolanaInstruction>, payer: &Pubkey) -> Result<Vec<Instruction>, SwapperError> {
    instructions
        .into_iter()
        .map(|instruction| {
            override_setup_payer(instruction, payer)
                .and_then(|instruction| decode_instruction::<Base64InstructionData>(instruction).map_err(solana_error))
                .and_then(wrap_instruction_in_cpi_proxy)
        })
        .collect()
}

fn verify_client_swap(swap: &SolanaClientSwap, user: &Pubkey, destination_account: &Pubkey) -> Result<(), SwapperError> {
    let wrapped_account = wrapped_sol::wrapped_sol_account(user).map_err(solana_error)?.to_string();
    let setup = swap.setup_instructions.as_deref().unwrap_or_default();
    let is_expected = swap.compute_budget_instructions.iter().flatten().all(is_compute_budget)
        && setup.len() <= MAX_SETUP_INSTRUCTIONS
        && setup.iter().all(|instruction| is_setup(instruction, &wrapped_account))
        && pays_into(&swap.swap_instruction, &destination_account.to_string())
        && swap.cleanup_instruction.as_ref().is_none_or(|instruction| is_cleanup(instruction, &user.to_string()));
    is_expected.then_some(()).ok_or(SwapperError::InvalidRoute)
}

fn is_compute_budget(instruction: &SolanaInstruction) -> bool {
    instruction.program_id == COMPUTE_BUDGET_PROGRAM_ID && decode_base64(&instruction.data).is_ok_and(|data| compute_budget::parse_compute_unit_price_data(&data).is_none_or(|price| price <= MAX_COMPUTE_UNIT_PRICE))
}

fn is_setup(instruction: &SolanaInstruction, wrapped_account: &str) -> bool {
    match instruction.program_id.as_str() {
        ASSOCIATED_TOKEN_ACCOUNT_PROGRAM => decode_base64(&instruction.data).is_ok_and(|data| is_create_account_data(&data)),
        SYSTEM_PROGRAM_ID => wraps_native_sol(instruction, wrapped_account),
        TOKEN_PROGRAM | TOKEN_PROGRAM_2022 => decode_base64(&instruction.data).is_ok_and(|data| data == [token::SYNC_NATIVE_DISCRIMINANT]),
        _ => false,
    }
}

fn pays_into(instruction: &SolanaInstruction, destination_account: &str) -> bool {
    let Ok(data) = decode_base64(&instruction.data) else {
        return false;
    };
    let destination = JUPITER_DESTINATION_ACCOUNTS
        .iter()
        .find(|(name, _)| data.starts_with(&anchor::global_discriminator(name)))
        .and_then(|(_, index)| instruction.accounts.get(*index));
    instruction.program_id == JUPITER_PROGRAM_ID && destination.is_some_and(|account| account.pubkey == destination_account)
}

fn is_cleanup(instruction: &SolanaInstruction, receiver: &str) -> bool {
    matches!(instruction.program_id.as_str(), TOKEN_PROGRAM | TOKEN_PROGRAM_2022)
        && decode_base64(&instruction.data).is_ok_and(|data| data == [token::CLOSE_ACCOUNT_DISCRIMINANT])
        && instruction.accounts.get(1).is_some_and(|account| account.pubkey == receiver)
}

fn wraps_native_sol(instruction: &SolanaInstruction, wrapped_account: &str) -> bool {
    instruction.program_id == SYSTEM_PROGRAM_ID && instruction.accounts.get(1).is_some_and(|account| account.pubkey == wrapped_account) && decode_base64(&instruction.data).is_ok_and(|data| system::transfer_lamports(&data).is_some())
}

fn setup_wraps_native_sol(instructions: &[SolanaInstruction], owner: &Pubkey) -> Result<bool, SwapperError> {
    let wrapped_account = wrapped_sol::wrapped_sol_account(owner).map_err(solana_error)?.to_string();
    Ok(instructions.iter().any(|instruction| wraps_native_sol(instruction, &wrapped_account)))
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
        mayan::{client::MayanClient, provider::Mayan, testkit::mock_solana_instruction},
        solana_dex::SolanaTransaction,
    };

    const USDT_USER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
    const USDT_LEDGER_ACCOUNT: &str = "5SGghFsD6otbtctbotPveZvUWBJGDz8CGFTCUjp5dNcB";
    const SOL_LEDGER_ACCOUNT: &str = "9HrGRWMADkN3Sdh3pUpcPuRbX9cyJztokCaS9PfmcnsX";
    const SOL_AMOUNT: &str = "2000000000";

    #[test]
    fn test_client_swap_through_jupiter_passes_the_contract_check() {
        let mayan = Mayan::with_clients(MayanClient::new(MockClient::new()), MayanClient::new(MockClient::new()), Arc::new(ProviderMock::new("{}".to_string())));
        let targets = mayan.contracts(&AssetId::from_chain(Chain::Solana)).targets;

        let user = Pubkey::from_base58(USDT_USER).unwrap();
        let ledger_account = Pubkey::from_base58(USDT_LEDGER_ACCOUNT).unwrap();
        let mut token = Vec::new();
        append_client_swap_instructions(&mut token, SolanaClientSwap::mock_usdt(), &user, &user, &ledger_account, SOLANA_USDT_TOKEN_ID, "1000000").unwrap();
        verify_solana_programs(&SolanaTransaction::new(token, vec![]).encode(USDT_USER, &[]).unwrap().data, &targets).unwrap();

        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let ledger_account = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let mut coin = Vec::new();
        append_client_swap_instructions(&mut coin, SolanaClientSwap::mock_sol(), &user, &user, &ledger_account, EVM_ZERO_ADDRESS, SOL_AMOUNT).unwrap();
        assert_eq!(coin.len(), 8, "the reply wraps the coin itself, so no second wrap is added");
    }

    #[test]
    fn test_client_swap_refuses_what_the_swap_does_not_need() {
        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let stranger = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();
        let ledger_account = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let token_account = get_associated_token_address_with_program_id(&user, &Pubkey::from_base58(SOLANA_USDT_TOKEN_ID).unwrap(), &program_ids::token_program()).unwrap();
        let swap = SolanaClientSwap::mock_sol();
        let set_authority = Instruction {
            program_id: program_ids::token_program(),
            accounts: vec![],
            data: vec![token::SET_AUTHORITY_DISCRIMINANT, 2, 0],
        };
        let unknown_program = Instruction {
            program_id: stranger,
            accounts: vec![AccountMeta::new_signer(user)],
            data: vec![],
        };
        let mut other_destination = swap.swap_instruction.clone();
        other_destination.accounts[5].pubkey = stranger.to_string();

        for (setup, reason) in [
            (vec![token::transfer(&token_account, &stranger, &user, 1)], "a setup that moves another token"),
            (vec![system::transfer(&user, &stranger, 1)], "a setup that sends coins away"),
            (vec![set_authority], "a setup that hands a token account over"),
            (vec![unknown_program.clone()], "a setup that runs another program"),
            (vec![token::sync_native(&token_account); MAX_SETUP_INSTRUCTIONS + 1], "more setup steps than a swap needs"),
        ] {
            let swap = SolanaClientSwap {
                setup_instructions: Some(setup.into_iter().map(mock_solana_instruction).collect()),
                ..swap.clone()
            };
            assert_eq!(
                append_client_swap_instructions(&mut Vec::new(), swap, &user, &user, &ledger_account, EVM_ZERO_ADDRESS, SOL_AMOUNT).err(),
                Some(SwapperError::InvalidRoute),
                "{reason}"
            );
        }

        for (swap, reason) in [
            (
                SolanaClientSwap {
                    swap_instruction: other_destination,
                    ..swap.clone()
                },
                "a swap that pays into another account",
            ),
            (
                SolanaClientSwap {
                    swap_instruction: mock_solana_instruction(unknown_program),
                    ..swap.clone()
                },
                "a swap through another program",
            ),
            (
                SolanaClientSwap {
                    cleanup_instruction: Some(mock_solana_instruction(token::close_account(&token_account, &stranger, &user))),
                    ..swap.clone()
                },
                "a cleanup that pays a stranger",
            ),
            (
                SolanaClientSwap {
                    cleanup_instruction: Some(mock_solana_instruction(token::transfer(&token_account, &user, &user, 1))),
                    ..swap.clone()
                },
                "a cleanup that is not a close",
            ),
            (
                SolanaClientSwap {
                    compute_budget_instructions: Some(vec![mock_solana_instruction(compute_budget::set_compute_unit_price(MAX_COMPUTE_UNIT_PRICE + 1))]),
                    ..swap.clone()
                },
                "a priority fee above the limit",
            ),
            (
                SolanaClientSwap {
                    compute_budget_instructions: Some(vec![mock_solana_instruction(system::transfer(&user, &stranger, 1))]),
                    ..swap.clone()
                },
                "a compute budget step that is something else",
            ),
        ] {
            assert_eq!(
                append_client_swap_instructions(&mut Vec::new(), swap, &user, &user, &ledger_account, EVM_ZERO_ADDRESS, SOL_AMOUNT).err(),
                Some(SwapperError::InvalidRoute),
                "{reason}"
            );
        }
    }

    #[test]
    fn test_pays_into() {
        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let destination = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let jupiter = Pubkey::from_base58(JUPITER_PROGRAM_ID).unwrap();

        for (name, destination_index, expected) in [
            ("route", 4, true),
            ("route", 3, false),
            ("route_v2", 7, true),
            ("route_v2", 6, false),
            ("shared_accounts_route", 6, true),
            ("shared_accounts_route", 5, false),
            ("shared_accounts_route_v2", 5, true),
            ("shared_accounts_route_v2", 4, false),
            ("exact_out_route", 4, false),
        ] {
            let mut accounts = vec![AccountMeta::new_readonly(user); 8];
            accounts[destination_index] = AccountMeta::new_writable(destination);
            let instruction = mock_solana_instruction(Instruction {
                program_id: jupiter,
                accounts,
                data: anchor::global_discriminator(name).to_vec(),
            });
            assert_eq!(pays_into(&instruction, SOL_LEDGER_ACCOUNT), expected, "{name} with the destination at account {destination_index}");
        }
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
