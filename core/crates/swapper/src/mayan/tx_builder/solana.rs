use std::fmt::Display;

use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{
    AccountMeta, Instruction, Pubkey, SolanaAddress,
    associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id},
    compute_budget,
    instructions::program_ids,
    token, wrapped_sol,
};

use crate::{
    SwapperError,
    jupiter::{QuotedRoute, RouteDestination, SwapInstructions},
    mayan::{
        constants::MAYAN_CPI_PROXY_PROGRAM_ID,
        model::{MayanQuoteCommon, SolanaClientSwap},
    },
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

pub(in crate::mayan::tx_builder) fn client_swap_route(user: &Pubkey, destination_account: &Pubkey, route: &MayanQuoteCommon) -> Result<QuotedRoute, SwapperError> {
    Ok(QuotedRoute {
        wallet: *user,
        input_mint: match route.from_token.contract.as_str() {
            EVM_ZERO_ADDRESS => wrapped_sol::wrapped_sol_mint(),
            contract => SolanaAddress::parse(contract).map_err(solana_error)?.into(),
        },
        amount: route.effective_amount_in64.parse::<u64>()?,
        minimum_out: 0,
        platform_fee_bps: 0,
        destination: RouteDestination::Account(*destination_account),
    })
}

pub(in crate::mayan::tx_builder) fn append_client_swap_instructions(instructions: &mut Vec<Instruction>, swap: SolanaClientSwap, relayer: &Pubkey, quoted: &QuotedRoute) -> Result<Vec<String>, SwapperError> {
    let SolanaClientSwap {
        compute_budget_instructions,
        setup_instructions,
        swap_instruction,
        cleanup_instruction,
        address_lookup_table_addresses,
    } = swap;
    let client_swap = SwapInstructions::new(compute_budget_instructions.unwrap_or_default(), setup_instructions.unwrap_or_default(), swap_instruction, cleanup_instruction)?;
    client_swap.verify(quoted)?;
    let needs_native_sol_wrap = quoted.input_mint == wrapped_sol::wrapped_sol_mint() && !client_swap.wraps_native_sol(&quoted.wallet)?;

    let SwapInstructions { compute_budget, setup, swap, cleanup } = client_swap;
    instructions.extend(compute_budget);
    if needs_native_sol_wrap {
        instructions.extend(wrap_native_sol_instructions(&quoted.wallet, quoted.amount)?);
    }
    for instruction in setup {
        instructions.push(wrap_instruction_in_cpi_proxy(with_setup_payer(instruction, relayer))?);
    }
    instructions.push(swap);
    if let Some(cleanup) = cleanup {
        instructions.push(wrap_instruction_in_cpi_proxy(cleanup)?);
    }
    Ok(address_lookup_table_addresses)
}

fn wrap_native_sol_instructions(owner: &Pubkey, amount: u64) -> Result<Vec<Instruction>, SwapperError> {
    wrapped_sol::wrap_instructions(owner, amount).map_err(solana_error)?.into_iter().map(wrap_instruction_in_cpi_proxy).collect()
}

fn with_setup_payer(mut instruction: Instruction, payer: &Pubkey) -> Instruction {
    if instruction.program_id == program_ids::associated_token_program()
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
        Chain,
        asset_constants::{SOLANA_USDC_TOKEN_ID, SOLANA_USDT_TOKEN_ID},
        testkit::signer_mock::{TEST_PRIVATE_KEY_SOLANA_ADDRESS, TEST_SOLANA_SENDER},
    };

    use super::*;
    use crate::{
        Quote, QuoteRequest, Swapper,
        alien::mock::ProviderMock,
        mayan::{
            client::MayanClient,
            model::{MayanMctpQuote, MayanToken},
            provider::Mayan,
        },
        solana_dex::SolanaTransaction,
    };

    const USDT_USER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
    const USDT_LEDGER_ACCOUNT: &str = "5SGghFsD6otbtctbotPveZvUWBJGDz8CGFTCUjp5dNcB";
    const USDT_AMOUNT: u64 = 5_000_000;
    const SOL_LEDGER_ACCOUNT: &str = "9HrGRWMADkN3Sdh3pUpcPuRbX9cyJztokCaS9PfmcnsX";
    const SOL_AMOUNT: u64 = 2_000_000_000;

    #[test]
    fn test_client_swap_through_jupiter_passes_the_contract_check() {
        let mayan = Mayan::with_clients(MayanClient::new(MockClient::new()), MayanClient::new(MockClient::new()), Arc::new(ProviderMock::new("{}".to_string())));
        let token = QuotedRoute {
            wallet: Pubkey::from_base58(USDT_USER).unwrap(),
            input_mint: Pubkey::from_base58(SOLANA_USDT_TOKEN_ID).unwrap(),
            amount: USDT_AMOUNT,
            minimum_out: 0,
            platform_fee_bps: 0,
            destination: RouteDestination::Account(Pubkey::from_base58(USDT_LEDGER_ACCOUNT).unwrap()),
        };
        let coin = QuotedRoute {
            wallet: Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap(),
            input_mint: wrapped_sol::wrapped_sol_mint(),
            amount: SOL_AMOUNT,
            destination: RouteDestination::Account(Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap()),
            ..token
        };

        let mut token_instructions = Vec::new();
        append_client_swap_instructions(&mut token_instructions, SolanaClientSwap::mock_usdt(), &token.wallet, &token).unwrap();
        let quote_data = SolanaTransaction::new(token_instructions, vec![]).encode(USDT_USER, &[]).unwrap();
        let quote = Quote::mock_with_request(&QuoteRequest {
            wallet_address: USDT_USER.to_string(),
            ..QuoteRequest::mock(Chain::Solana, Some(SOLANA_USDT_TOKEN_ID))
        });
        (&mayan as &dyn Swapper).verify_binding(&quote, &quote_data).unwrap();

        let mut coin_instructions = Vec::new();
        append_client_swap_instructions(&mut coin_instructions, SolanaClientSwap::mock_sol(), &coin.wallet, &coin).unwrap();
        assert_eq!(coin_instructions.len(), 8, "the reply wraps the coin itself, so no second wrap is added");

        let other_account = QuotedRoute {
            destination: RouteDestination::Account(Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap()),
            ..coin
        };
        assert_eq!(
            append_client_swap_instructions(&mut Vec::new(), SolanaClientSwap::mock_sol(), &other_account.wallet, &other_account).err(),
            Some(SwapperError::InvalidRoute),
            "a swap that pays into another account than the bridge's"
        );

        let relayer = Pubkey::new([7; 32]);
        let mut paid_by_relayer = Vec::new();
        append_client_swap_instructions(&mut paid_by_relayer, SolanaClientSwap::mock_sol(), &relayer, &coin).unwrap();
        let payers: Vec<Pubkey> = paid_by_relayer
            .iter()
            .filter(|instruction| instruction.account(0) == Some(&program_ids::associated_token_program()))
            .filter_map(|instruction| instruction.account(1).copied())
            .collect();
        assert_eq!(payers, vec![relayer, relayer], "the relayer pays for every token account the setup creates");
    }

    #[test]
    fn test_client_swap_route() {
        let user = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let ledger = Pubkey::from_base58(SOL_LEDGER_ACCOUNT).unwrap();
        let token = MayanMctpQuote::mock_solana_to_sui().common;
        let coin = MayanQuoteCommon {
            from_token: MayanToken {
                contract: EVM_ZERO_ADDRESS.to_string(),
                ..token.from_token.clone()
            },
            ..token.clone()
        };

        let route = client_swap_route(&user, &ledger, &token).unwrap();
        assert_eq!(route.input_mint, Pubkey::from_base58(SOLANA_USDC_TOKEN_ID).unwrap());
        assert_eq!(
            (route.wallet, route.amount, route.minimum_out, route.platform_fee_bps),
            (user, 1_000_000, 0, 0),
            "the bridge's own instruction carries the minimum it needs"
        );
        assert_eq!(client_swap_route(&user, &ledger, &coin).unwrap().input_mint, wrapped_sol::wrapped_sol_mint());
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
