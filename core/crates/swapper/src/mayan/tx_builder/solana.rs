use std::fmt::Display;

use gem_encoding::decode_base64;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, AccountMeta, Base64InstructionData, COMPUTE_BUDGET_PROGRAM_ID, Instruction, JUPITER_PROGRAM_ID, Pubkey, SYSTEM_PROGRAM_ID, SolanaAddress, TOKEN_PROGRAM, TOKEN_PROGRAM_2022, anchor,
    associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id, is_create_account_data},
    compute_budget, instruction_from_primitive,
    instructions::program_ids,
    instructions_from_primitives, system, token, wrapped_sol,
};
use primitives::SolanaInstruction;

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

fn verify_client_swap(swap: &SolanaClientSwap, user: &Pubkey, destination_account: &Pubkey) -> Result<(), SwapperError> {
    let wrapped_account = wrapped_sol::wrapped_sol_account(user).map_err(solana_error)?.to_string();
    let setup = swap.setup_instructions.as_deref().unwrap_or_default();
    let is_expected = swap.compute_budget_instructions.iter().flatten().all(is_compute_budget)
        && setup.len() <= MAX_SETUP_INSTRUCTIONS
        && setup.iter().all(|instruction| is_setup(instruction, &wrapped_account))
        && pays_into(&swap.swap_instruction, &destination_account.to_string())
        && swap.cleanup_instruction.as_ref().is_none_or(|instruction| is_cleanup(instruction, &user.to_string()));
    match is_expected {
        true => Ok(()),
        false => Err(SwapperError::InvalidRoute),
    }
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
        AssetId, Chain, SolanaAccountMeta,
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
    const SOL_LEDGER_ACCOUNT: &str = "9HrGRWMADkN3Sdh3pUpcPuRbX9cyJztokCaS9PfmcnsX";

    fn pubkey(address: &str) -> Pubkey {
        SolanaAddress::parse(address).unwrap().into()
    }

    fn usdt_swap() -> SolanaClientSwap {
        serde_json::from_str(include_str!("../test/solana_client_swap_usdt_jupiter.json")).unwrap()
    }

    fn sol_swap() -> SolanaClientSwap {
        serde_json::from_str(include_str!("../test/solana_client_swap_sol_jupiter.json")).unwrap()
    }

    fn primitive(instruction: Instruction) -> SolanaInstruction {
        SolanaInstruction {
            program_id: instruction.program_id.to_string(),
            accounts: instruction
                .accounts
                .into_iter()
                .map(|account| SolanaAccountMeta {
                    pubkey: account.pubkey.to_string(),
                    is_signer: account.is_signer,
                    is_writable: account.is_writable,
                })
                .collect(),
            data: gem_encoding::encode_base64(&instruction.data),
        }
    }

    fn build(swap: SolanaClientSwap, user: &str, ledger_account: &str, from_token_contract: &str, amount: &str) -> Result<Vec<Instruction>, SwapperError> {
        let mut instructions = Vec::new();
        append_client_swap_instructions(&mut instructions, swap, &pubkey(user), &pubkey(user), &pubkey(ledger_account), from_token_contract, amount)?;
        Ok(instructions)
    }

    fn build_sol(swap: SolanaClientSwap) -> Result<Vec<Instruction>, SwapperError> {
        build(swap, TEST_SOLANA_SENDER, SOL_LEDGER_ACCOUNT, EVM_ZERO_ADDRESS, "2000000000")
    }

    #[test]
    fn test_client_swap_through_jupiter_passes_the_contract_check() {
        let mayan = Mayan::with_clients(MayanClient::new(MockClient::new()), MayanClient::new(MockClient::new()), Arc::new(ProviderMock::new("{}".to_string())));
        let targets = mayan.contracts(&AssetId::from_chain(Chain::Solana)).targets;

        let token = build(usdt_swap(), USDT_USER, USDT_LEDGER_ACCOUNT, SOLANA_USDT_TOKEN_ID, "1000000").unwrap();
        verify_solana_programs(&SolanaTransaction::new(token, vec![]).encode(USDT_USER, &[]).unwrap().data, &targets).unwrap();

        assert_eq!(build_sol(sol_swap()).unwrap().len(), 8, "the reply wraps the coin itself, so no second wrap is added");
    }

    #[test]
    fn test_client_swap_refuses_what_the_swap_does_not_need() {
        let user = pubkey(TEST_SOLANA_SENDER);
        let stranger = pubkey(TEST_PRIVATE_KEY_SOLANA_ADDRESS);
        let token_account = get_associated_token_address_with_program_id(&user, &pubkey(SOLANA_USDT_TOKEN_ID), &program_ids::token_program()).unwrap();
        let setup = |instruction: Instruction| {
            let mut swap = sol_swap();
            swap.setup_instructions.get_or_insert_default().push(primitive(instruction));
            swap
        };
        let cleanup = |instruction: Instruction| SolanaClientSwap {
            cleanup_instruction: Some(primitive(instruction)),
            ..sol_swap()
        };
        let swap_account = |index: usize, address: &Pubkey| {
            let mut swap = sol_swap();
            swap.swap_instruction.accounts[index].pubkey = address.to_string();
            swap
        };
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

        for (swap, reason) in [
            (setup(token::transfer(&token_account, &stranger, &user, 1)), "a setup that moves another token"),
            (setup(system::transfer(&user, &stranger, 1)), "a setup that sends coins away"),
            (setup(set_authority), "a setup that hands a token account over"),
            (setup(unknown_program.clone()), "a setup that runs another program"),
            (
                SolanaClientSwap {
                    setup_instructions: Some(vec![primitive(token::sync_native(&token_account)); MAX_SETUP_INSTRUCTIONS + 1]),
                    ..sol_swap()
                },
                "more setup steps than a swap needs",
            ),
            (swap_account(5, &stranger), "a swap that pays into another account"),
            (
                SolanaClientSwap {
                    swap_instruction: primitive(unknown_program),
                    ..sol_swap()
                },
                "a swap through another program",
            ),
            (cleanup(token::close_account(&token_account, &stranger, &user)), "a cleanup that pays a stranger"),
            (cleanup(token::transfer(&token_account, &user, &user, 1)), "a cleanup that is not a close"),
            (
                SolanaClientSwap {
                    compute_budget_instructions: Some(vec![primitive(compute_budget::set_compute_unit_price(MAX_COMPUTE_UNIT_PRICE + 1))]),
                    ..sol_swap()
                },
                "a priority fee above the limit",
            ),
            (
                SolanaClientSwap {
                    compute_budget_instructions: Some(vec![primitive(system::transfer(&user, &stranger, 1))]),
                    ..sol_swap()
                },
                "a compute budget step that is something else",
            ),
        ] {
            assert_eq!(build_sol(swap).err(), Some(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_pays_into() {
        let destination = pubkey(SOL_LEDGER_ACCOUNT);
        let jupiter = pubkey(JUPITER_PROGRAM_ID);
        let swap = |name: &str, destination_index: usize| {
            let mut accounts = vec![AccountMeta::new_readonly(pubkey(TEST_SOLANA_SENDER)); 8];
            accounts[destination_index] = AccountMeta::new_writable(destination);
            primitive(Instruction {
                program_id: jupiter,
                accounts,
                data: anchor::global_discriminator(name).to_vec(),
            })
        };

        for (name, index) in JUPITER_DESTINATION_ACCOUNTS {
            assert!(pays_into(&swap(name, index), SOL_LEDGER_ACCOUNT), "{name}");
            assert!(!pays_into(&swap(name, index - 1), SOL_LEDGER_ACCOUNT), "{name}");
        }
        assert!(!pays_into(&swap("exact_out_route", 4), SOL_LEDGER_ACCOUNT));
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
