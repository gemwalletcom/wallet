use gem_solana::{Base64InstructionData, Instruction, Pubkey, StandardInstruction, associated_token::is_associated_token_address, instruction_from_primitive, instructions_from_primitives, wrapped_sol};
use primitives::SolanaInstruction;

use super::route::JupiterRoute;
use crate::{SwapperError, contract_check::solana::verify_steps};

#[derive(Debug, Clone, Copy)]
pub(crate) enum RouteDestination {
    WalletAccount(Pubkey),
    Account(Pubkey),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QuotedRoute {
    pub wallet: Pubkey,
    pub input_mint: Pubkey,
    pub amount: u64,
    pub minimum_out: u64,
    pub platform_fee_bps: u16,
    pub destination: RouteDestination,
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

    pub(crate) fn verify(&self, quoted: &QuotedRoute) -> Result<(), SwapperError> {
        let route = JupiterRoute::decode(&self.swap).ok_or(SwapperError::InvalidRoute)?;
        let pays_destination = match quoted.destination {
            RouteDestination::WalletAccount(mint) => is_associated_token_address(&route.destination, &quoted.wallet, &mint),
            RouteDestination::Account(destination) => route.destination == destination,
        };
        let is_quoted = route.authority == quoted.wallet
            && is_associated_token_address(&route.source, &quoted.wallet, &quoted.input_mint)
            && route.arguments.in_amount <= quoted.amount
            && route.arguments.minimum_out() >= quoted.minimum_out
            && route.arguments.platform_fee_bps <= quoted.platform_fee_bps
            && pays_destination;
        if !is_quoted {
            return Err(SwapperError::InvalidRoute);
        }
        let steps: Vec<StandardInstruction> = self.compute_budget.iter().chain(&self.setup).chain(&self.cleanup).map(StandardInstruction::new).collect();
        verify_steps(&steps, &quoted.wallet, &[])
    }

    pub(crate) fn wraps_native_sol(&self, wallet: &Pubkey) -> Result<bool, SwapperError> {
        let wrapped_sol = wrapped_sol::wrapped_sol_account(wallet).map_err(SwapperError::transaction_error)?;
        Ok(self.setup.iter().any(|instruction| StandardInstruction::new(instruction).transfers_to(&wrapped_sol)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, associated_token::get_associated_token_address_with_program_id, instructions::program_ids::token_program, system};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

    #[test]
    fn test_verify() {
        let quoted = QuotedRoute::mock();
        let wallet = quoted.wallet;
        let stranger = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();
        let usdc = Pubkey::from_base58(USDC_TOKEN_MINT).unwrap();
        let wallet_usdc = get_associated_token_address_with_program_id(&wallet, &usdc, &token_program()).unwrap();
        let stranger_usdc = get_associated_token_address_with_program_id(&stranger, &usdc, &token_program()).unwrap();
        let instructions = SwapInstructions::mock();

        assert_eq!(instructions.verify(&quoted), Ok(()));
        assert_eq!(SwapInstructions::mock_usdc_sol().verify(&QuotedRoute::mock_usdc_sol()), Ok(()));
        assert_eq!(
            instructions.verify(&QuotedRoute {
                destination: RouteDestination::Account(wallet_usdc),
                ..quoted
            }),
            Ok(())
        );
        assert_eq!(instructions.verify(&QuotedRoute { amount: quoted.amount + 1, ..quoted }), Ok(()), "a route may spend less than quoted");

        for (refused, reason) in [
            (QuotedRoute { amount: quoted.amount - 1, ..quoted }, "a route that spends more than quoted"),
            (
                QuotedRoute {
                    minimum_out: quoted.minimum_out + 1,
                    ..quoted
                },
                "a route that accepts less than Confirm shows",
            ),
            (
                QuotedRoute {
                    platform_fee_bps: quoted.platform_fee_bps - 1,
                    ..quoted
                },
                "a route that takes a higher fee than quoted",
            ),
        ] {
            assert_eq!(instructions.verify(&refused), Err(SwapperError::InvalidRoute), "{reason}");
        }

        let mut route_to_stranger = instructions.swap.clone();
        route_to_stranger.accounts[2].pubkey = stranger_usdc;
        for (index, account, reason) in [
            (0, stranger, "a route another key authorizes"),
            (1, wrapped_sol::wrapped_sol_account(&stranger).unwrap(), "a route that spends from someone else's account"),
            (1, wallet_usdc, "a route that spends another token the wallet holds"),
            (2, stranger_usdc, "a route that pays someone else"),
        ] {
            let mut route = instructions.clone();
            route.swap.accounts[index].pubkey = account;
            assert_eq!(route.verify(&quoted), Err(SwapperError::InvalidRoute), "{reason}");
        }

        let sends_coins_away = SwapInstructions {
            setup: vec![system::transfer(&wallet, &stranger, 1_000_000_000)],
            ..instructions
        };
        assert_eq!(sends_coins_away.verify(&quoted), Err(SwapperError::InvalidRoute), "the steps around the route follow the Solana step rules");
        let second_route = SwapInstructions {
            setup: vec![route_to_stranger],
            ..SwapInstructions::mock()
        };
        assert_eq!(second_route.verify(&quoted), Err(SwapperError::InvalidRoute), "a second Jupiter route among the steps around the swap");
    }

    #[test]
    fn test_wraps_native_sol() {
        let wallet = QuotedRoute::mock().wallet;

        assert_eq!(SwapInstructions::mock().wraps_native_sol(&wallet), Ok(true));
        assert_eq!(SwapInstructions::mock_usdc_sol().wraps_native_sol(&wallet), Ok(false));
        assert_eq!(SwapInstructions::mock().wraps_native_sol(&Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap()), Ok(false));
    }
}
