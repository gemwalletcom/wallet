use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, Base64InstructionData, COMPUTE_BUDGET_PROGRAM_ID, Instruction, Pubkey, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM, TOKEN_PROGRAM_2022,
    associated_token::{is_associated_token_address, is_create_account_data},
    compute_budget, instruction_from_primitive, instructions_from_primitives, system, token, wrapped_sol,
};
use primitives::SolanaInstruction;

use super::route::JupiterRoute;
use crate::SwapperError;

const MAX_SETUP_INSTRUCTIONS: usize = 6;
const MAX_COMPUTE_UNIT_PRICE: u64 = 100_000_000;

#[derive(Debug, Clone, Copy)]
pub(crate) enum RouteDestination {
    WalletAccount(Pubkey),
    Account(Pubkey),
}

#[derive(Debug, Clone)]
pub(crate) struct SwapInstructions {
    pub compute_budget: Vec<Instruction>,
    pub setup: Vec<Instruction>,
    pub swap: Instruction,
    pub cleanup: Option<Instruction>,
}

impl SwapInstructions {
    pub(crate) fn new(compute_budget: Vec<SolanaInstruction>, setup: Vec<SolanaInstruction>, swap: SolanaInstruction, cleanup: Option<SolanaInstruction>) -> Result<Self, SwapperError> {
        Ok(Self {
            compute_budget: instructions_from_primitives::<Base64InstructionData>(compute_budget).map_err(SwapperError::transaction_error)?,
            setup: instructions_from_primitives::<Base64InstructionData>(setup).map_err(SwapperError::transaction_error)?,
            swap: instruction_from_primitive::<Base64InstructionData>(swap).map_err(SwapperError::transaction_error)?,
            cleanup: cleanup.map(instruction_from_primitive::<Base64InstructionData>).transpose().map_err(SwapperError::transaction_error)?,
        })
    }

    pub(crate) fn verify(&self, wallet: &Pubkey, input_mint: &Pubkey, amount: u64, destination: RouteDestination) -> Result<(), SwapperError> {
        let route = JupiterRoute::decode(&self.swap).ok_or(SwapperError::InvalidRoute)?;
        let wrapped_sol = wrapped_sol::wrapped_sol_account(wallet).map_err(SwapperError::transaction_error)?;
        let pays_into = |account: &Pubkey| match destination {
            RouteDestination::WalletAccount(mint) => is_associated_token_address(account, wallet, &mint),
            RouteDestination::Account(destination) => *account == destination,
        };
        let is_bound = route.authority == *wallet
            && is_associated_token_address(&route.source, wallet, input_mint)
            && route.in_amount <= amount
            && pays_into(&route.destination)
            && self.compute_budget.iter().all(is_compute_budget)
            && self.setup.len() <= MAX_SETUP_INSTRUCTIONS
            && self.setup.iter().all(|instruction| is_setup(instruction, wallet, &wrapped_sol, pays_into))
            && self.cleanup.iter().all(|instruction| is_cleanup(instruction, wallet));
        if !is_bound {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(())
    }

    pub(crate) fn wraps_native_sol(&self, wallet: &Pubkey) -> Result<bool, SwapperError> {
        let wrapped_sol = wrapped_sol::wrapped_sol_account(wallet).map_err(SwapperError::transaction_error)?;
        Ok(self.setup.iter().any(|instruction| is_native_sol_wrap(instruction, &wrapped_sol)))
    }
}

fn account(instruction: &Instruction, index: usize) -> Option<&Pubkey> {
    instruction.accounts.get(index).map(|account| &account.pubkey)
}

fn is_compute_budget(instruction: &Instruction) -> bool {
    instruction.program_id.to_base58() == COMPUTE_BUDGET_PROGRAM_ID && compute_budget::parse_compute_unit_price_data(&instruction.data).is_none_or(|price| price <= MAX_COMPUTE_UNIT_PRICE)
}

fn is_setup(instruction: &Instruction, wallet: &Pubkey, wrapped_sol: &Pubkey, pays_into: impl Fn(&Pubkey) -> bool) -> bool {
    match instruction.program_id.to_base58().as_str() {
        ASSOCIATED_TOKEN_ACCOUNT_PROGRAM => is_create_account_data(&instruction.data) && (account(instruction, 2) == Some(wallet) || account(instruction, 1).is_some_and(pays_into)),
        SYSTEM_PROGRAM_ID => is_native_sol_wrap(instruction, wrapped_sol),
        TOKEN_PROGRAM | TOKEN_PROGRAM_2022 => instruction.data == [token::SYNC_NATIVE_DISCRIMINANT],
        _ => false,
    }
}

fn is_native_sol_wrap(instruction: &Instruction, wrapped_sol: &Pubkey) -> bool {
    instruction.program_id.to_base58() == SYSTEM_PROGRAM_ID && system::transfer_lamports(&instruction.data).is_some() && account(instruction, 1) == Some(wrapped_sol)
}

fn is_cleanup(instruction: &Instruction, wallet: &Pubkey) -> bool {
    match instruction.program_id.to_base58().as_str() {
        TOKEN_PROGRAM | TOKEN_PROGRAM_2022 => instruction.data == [token::CLOSE_ACCOUNT_DISCRIMINANT] && account(instruction, 1) == Some(wallet),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jupiter::testkit::TEST_PAYER;
    use gem_solana::{
        AccountMeta, USDC_TOKEN_MINT,
        associated_token::{create_associated_token_account_idempotent, get_associated_token_address_with_program_id},
        instructions::program_ids::token_program,
        wrapped_sol::wrapped_sol_mint,
    };
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

    const AMOUNT: u64 = 100_000_000;

    #[test]
    fn test_verify() {
        let wallet = Pubkey::from_base58(TEST_PAYER).unwrap();
        let stranger = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();
        let (sol, usdc) = (wrapped_sol_mint(), Pubkey::from_base58(USDC_TOKEN_MINT).unwrap());
        let wallet_usdc = get_associated_token_address_with_program_id(&wallet, &usdc, &token_program()).unwrap();
        let stranger_usdc = get_associated_token_address_with_program_id(&stranger, &usdc, &token_program()).unwrap();
        let stranger_sol = wrapped_sol::wrapped_sol_account(&stranger).unwrap();
        let instructions = SwapInstructions::mock();

        assert_eq!(instructions.verify(&wallet, &sol, AMOUNT, RouteDestination::WalletAccount(usdc)), Ok(()));
        assert_eq!(instructions.verify(&wallet, &sol, AMOUNT, RouteDestination::Account(wallet_usdc)), Ok(()));
        assert_eq!(instructions.verify(&wallet, &sol, AMOUNT + 1, RouteDestination::WalletAccount(usdc)), Ok(()), "a route may spend less than quoted");
        assert_eq!(
            SwapInstructions::mock_usdc_sol().verify(&wallet, &usdc, 20_000_000, RouteDestination::WalletAccount(sol)),
            Ok(()),
            "a route without its own destination pays into the wallet's account"
        );

        assert_eq!(
            instructions.verify(&wallet, &sol, AMOUNT - 1, RouteDestination::WalletAccount(usdc)),
            Err(SwapperError::InvalidRoute),
            "a route that spends more than quoted"
        );
        for (index, account, reason) in [
            (0, stranger, "a route another key authorizes"),
            (1, stranger_sol, "a route that spends from someone else's account"),
            (1, wallet_usdc, "a route that spends another token the wallet holds"),
            (2, stranger_usdc, "a route that pays someone else"),
            (7, stranger_usdc, "a route whose own destination is someone else's"),
        ] {
            let mut route = instructions.clone();
            route.swap.accounts[index].pubkey = account;
            assert_eq!(route.verify(&wallet, &sol, AMOUNT, RouteDestination::WalletAccount(usdc)), Err(SwapperError::InvalidRoute), "{reason}");
        }

        let set_authority = Instruction {
            program_id: token_program(),
            accounts: vec![AccountMeta::new_writable(wallet_usdc), AccountMeta::new_signer(wallet)],
            data: vec![token::SET_AUTHORITY_DISCRIMINANT, 2, 0],
        };
        let unknown_program = Instruction {
            program_id: stranger,
            accounts: vec![AccountMeta::new_signer(wallet)],
            data: vec![],
        };
        for (step, reason) in [
            (
                SwapInstructions {
                    setup: vec![token::transfer(&wallet_usdc, &stranger_usdc, &wallet, 1)],
                    ..instructions.clone()
                },
                "a setup that moves another token",
            ),
            (
                SwapInstructions {
                    setup: vec![system::transfer(&wallet, &stranger, 1)],
                    ..instructions.clone()
                },
                "a setup that sends coins away",
            ),
            (
                SwapInstructions {
                    setup: vec![set_authority],
                    ..instructions.clone()
                },
                "a setup that hands a token account over",
            ),
            (
                SwapInstructions {
                    setup: vec![unknown_program.clone()],
                    ..instructions.clone()
                },
                "a setup that runs another program",
            ),
            (
                SwapInstructions {
                    setup: vec![create_associated_token_account_idempotent(&wallet, &stranger, &usdc, &token_program()).unwrap()],
                    ..instructions.clone()
                },
                "a setup that pays rent for someone else's account",
            ),
            (
                SwapInstructions {
                    setup: vec![token::sync_native(&wallet_usdc); MAX_SETUP_INSTRUCTIONS + 1],
                    ..instructions.clone()
                },
                "more setup steps than a swap needs",
            ),
            (
                SwapInstructions {
                    cleanup: Some(token::close_account(&wallet_usdc, &stranger, &wallet)),
                    ..instructions.clone()
                },
                "a cleanup that pays a stranger",
            ),
            (
                SwapInstructions {
                    cleanup: Some(token::transfer(&wallet_usdc, &wallet_usdc, &wallet, 1)),
                    ..instructions.clone()
                },
                "a cleanup that is not a close",
            ),
            (
                SwapInstructions {
                    compute_budget: vec![compute_budget::set_compute_unit_price(MAX_COMPUTE_UNIT_PRICE + 1)],
                    ..instructions.clone()
                },
                "a priority fee above the limit",
            ),
            (
                SwapInstructions {
                    compute_budget: vec![system::transfer(&wallet, &stranger, 1)],
                    ..instructions.clone()
                },
                "a compute budget step that is something else",
            ),
            (SwapInstructions { swap: unknown_program, ..instructions }, "a swap through another program"),
        ] {
            assert_eq!(step.verify(&wallet, &sol, AMOUNT, RouteDestination::WalletAccount(usdc)), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_wraps_native_sol() {
        let wallet = Pubkey::from_base58(TEST_PAYER).unwrap();

        assert_eq!(SwapInstructions::mock().wraps_native_sol(&wallet), Ok(true));
        assert_eq!(SwapInstructions::mock_usdc_sol().wraps_native_sol(&wallet), Ok(false));
        assert_eq!(SwapInstructions::mock().wraps_native_sol(&Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap()), Ok(false));
    }
}
