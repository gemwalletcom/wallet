use gem_solana::models::SolanaInstruction;
use gem_solana::{Base64InstructionData, Instruction, Pubkey, StandardInstruction, associated_token::is_associated_token_address, decode_instruction, decode_instructions, instructions::program_ids::compute_budget_program};

use super::JupiterRoute;
use crate::{
    SwapperError,
    binding_check::solana::{StepLimits, verify_steps},
    fees::subtract_bps,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum RouteDestination {
    WalletAccount(Pubkey),
    Account(Pubkey),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlatformFee {
    pub bps: u16,
    pub account: Pubkey,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QuotedRoute {
    pub wallet: Pubkey,
    pub input_mint: Pubkey,
    pub amount: u64,
    pub minimum_out: u64,
    pub platform_fee: Option<PlatformFee>,
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
            compute_budget: decode_instructions::<Base64InstructionData>(compute_budget).map_err(SwapperError::transaction_error)?,
            setup: decode_instructions::<Base64InstructionData>(setup).map_err(SwapperError::transaction_error)?,
            swap: decode_instruction::<Base64InstructionData>(swap).map_err(SwapperError::transaction_error)?,
            cleanup: cleanup.map(decode_instruction::<Base64InstructionData>).transpose().map_err(SwapperError::transaction_error)?,
        })
    }

    pub(crate) fn verify(&self, quoted: &QuotedRoute) -> Result<(), SwapperError> {
        let route = JupiterRoute::decode(&self.swap).ok_or(SwapperError::InvalidRoute)?;
        let pays_destination = match quoted.destination {
            RouteDestination::WalletAccount(mint) => is_associated_token_address(&route.destination, &quoted.wallet, &mint),
            RouteDestination::Account(destination) => route.destination == destination,
        };
        let takes_quoted_fee = match quoted.platform_fee {
            Some(fee) => route.arguments.platform_fee_bps == fee.bps && route.fee_account == Some(fee.account),
            None => route.arguments.platform_fee_bps == 0,
        };
        let is_quoted = route.spends_from(&quoted.wallet, &quoted.input_mint)
            && route.arguments.in_amount <= quoted.amount
            && subtract_bps(&route.arguments.quoted_out_amount, u32::from(route.arguments.slippage_bps)) >= quoted.minimum_out
            && route.arguments.positive_slippage_bps == 0
            && takes_quoted_fee
            && pays_destination;
        if !is_quoted || self.compute_budget.iter().any(|instruction| instruction.program_id != compute_budget_program()) {
            return Err(SwapperError::InvalidRoute);
        }
        let steps: Vec<StandardInstruction> = self.compute_budget.iter().chain(&self.setup).chain(&self.cleanup).map(StandardInstruction::new).collect();
        verify_steps(&steps, &quoted.wallet, &[], &StepLimits::spending(&quoted.input_mint, quoted.amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, associated_token::get_associated_token_address_with_program_id, instructions::program_ids::token_program, system, token, wrapped_sol};
    use primitives::{swap::SOLANA_REFERRAL_ADDRESS, testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS};

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
                    platform_fee: quoted.platform_fee.map(|fee| PlatformFee { bps: fee.bps - 1, ..fee }),
                    ..quoted
                },
                "a route that takes a higher fee than quoted",
            ),
            (
                QuotedRoute {
                    platform_fee: quoted.platform_fee.map(|fee| PlatformFee { bps: fee.bps + 1, ..fee }),
                    ..quoted
                },
                "a route that leaves out part of Gem's fee",
            ),
            (QuotedRoute { platform_fee: None, ..quoted }, "a fee where the quote takes none"),
            (
                QuotedRoute {
                    platform_fee: quoted.platform_fee.map(|fee| PlatformFee { account: stranger, ..fee }),
                    ..quoted
                },
                "a fee paid into an account other than Gem's",
            ),
        ] {
            assert_eq!(instructions.verify(&refused), Err(SwapperError::InvalidRoute), "{reason}");
        }

        let mut positive_slippage = instructions.clone();
        positive_slippage.swap.data[28] = 1;
        assert_eq!(positive_slippage.verify(&quoted), Err(SwapperError::InvalidRoute), "a route that keeps part of a better price than quoted");

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
            ..instructions.clone()
        };
        assert_eq!(sends_coins_away.verify(&quoted), Err(SwapperError::InvalidRoute), "the steps around the route follow the Solana step rules");
        let wraps_in_compute_budget = SwapInstructions {
            compute_budget: vec![system::transfer(&wallet, &wrapped_sol::wrapped_sol_account(&wallet).unwrap(), 1)],
            ..instructions
        };
        assert_eq!(wraps_in_compute_budget.verify(&quoted), Err(SwapperError::InvalidRoute), "the compute budget steps hold only compute budget instructions");
        let usdc_route = QuotedRoute::mock_usdc_sol();
        let pays_gem_apart = SwapInstructions {
            setup: vec![token::transfer_checked_with_program_id(
                &get_associated_token_address_with_program_id(&usdc_route.wallet, &usdc, &token_program()).unwrap(),
                &usdc,
                &get_associated_token_address_with_program_id(&Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap(), &usdc, &token_program()).unwrap(),
                &usdc_route.wallet,
                1,
                6,
                &token_program(),
            )],
            ..SwapInstructions::mock_usdc_sol()
        };
        assert_eq!(pays_gem_apart.verify(&usdc_route), Err(SwapperError::InvalidRoute), "Gem's fee is taken inside the route, never as a step of its own");
        let second_route = SwapInstructions {
            setup: vec![route_to_stranger],
            ..SwapInstructions::mock()
        };
        assert_eq!(second_route.verify(&quoted), Err(SwapperError::InvalidRoute), "a second Jupiter route among the steps around the swap");
    }
}
