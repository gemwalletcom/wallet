use gem_solana::{
    MAX_COMPUTE_UNIT_LIMIT, Pubkey, StandardInstruction,
    associated_token::is_associated_token_address,
    decode_transaction,
    instructions::program_ids::{token_2022_program, token_program},
    wrapped_sol,
};
use primitives::{Chain, swap::SOLANA_REFERRAL_ADDRESS};

use crate::{
    Deployment, Quote, SwapperError,
    fees::{FeeSide, on_chain_referral_fees},
};

const MAX_PRIORITY_FEE_LAMPORTS: u64 = 10_000_000;
const MICRO_LAMPORTS_PER_LAMPORT: u64 = 1_000_000;
const MAX_COMPUTE_UNIT_PRICE: u64 = MAX_PRIORITY_FEE_LAMPORTS * MICRO_LAMPORTS_PER_LAMPORT / MAX_COMPUTE_UNIT_LIMIT as u64;
const TOKEN_ACCOUNT_RENT_LAMPORTS: u64 = 2_039_280;
const MAX_UNBOUND_LAMPORTS: u64 = 2 * TOKEN_ACCOUNT_RENT_LAMPORTS;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct StepLimits {
    pub fees: Vec<(Pubkey, u64)>,
    pub wrapped_sol: u64,
}

impl StepLimits {
    pub(crate) fn spending(mint: &Pubkey, amount: u64) -> Self {
        Self {
            fees: vec![],
            wrapped_sol: if *mint == wrapped_sol::wrapped_sol_mint() { amount } else { 0 },
        }
    }

    pub(super) fn from_quote(quote: &Quote) -> Result<Self, SwapperError> {
        let fee = on_chain_referral_fees(&quote.request).solana;
        let from_asset = quote.request.from_asset.asset_id();
        let fees = [(from_asset.clone(), FeeSide::Input, &quote.from_value), (quote.request.to_asset.asset_id(), FeeSide::Output, &quote.to_value)]
            .into_iter()
            .filter(|(asset, _, _)| asset.chain == Chain::Solana)
            .map(|(asset, side, value)| {
                let mint = match asset.token_id {
                    Some(token_id) => Pubkey::from_base58(&token_id).map_err(SwapperError::transaction_error)?,
                    None => wrapped_sol::wrapped_sol_mint(),
                };
                Ok((mint, u64::try_from(fee.quoted_fee(side, value.clone())).unwrap_or(u64::MAX)))
            })
            .collect::<Result<_, SwapperError>>()?;
        let wrapped_sol = match from_asset.chain == Chain::Solana && from_asset.is_native() {
            true => u64::try_from(&quote.from_value).unwrap_or(u64::MAX),
            false => 0,
        };
        Ok(Self { fees, wrapped_sol })
    }
}

struct StepPolicy<'a> {
    wallet: &'a Pubkey,
    wrapped_sol: Pubkey,
    referral: Pubkey,
    contracts: &'a [&'a str],
    fee_limits: &'a [(Pubkey, u64)],
}

pub(crate) fn verify_solana(transaction: &str, wallet: &str, contracts: &[&str], limits: &StepLimits) -> Result<(), SwapperError> {
    let transaction = decode_transaction(transaction).map_err(SwapperError::transaction_error)?;
    let keys = transaction.account_keys();
    let instructions = transaction
        .instructions()
        .iter()
        .map(|instruction| StandardInstruction::compiled(instruction, keys))
        .collect::<Option<Vec<_>>>()
        .ok_or(SwapperError::InvalidRoute)?;
    verify_steps(&instructions, &Pubkey::from_base58(wallet).map_err(SwapperError::transaction_error)?, contracts, limits)
}

pub(crate) fn verify_steps(instructions: &[StandardInstruction], wallet: &Pubkey, contracts: &[&str], limits: &StepLimits) -> Result<(), SwapperError> {
    let policy = StepPolicy {
        wallet,
        wrapped_sol: wrapped_sol::wrapped_sol_account(wallet).map_err(SwapperError::transaction_error)?,
        referral: Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).map_err(SwapperError::transaction_error)?,
        contracts,
        fee_limits: &limits.fees,
    };
    let unbound_lamports = instructions.iter().enumerate().try_fold(0u64, |total, (position, instruction)| {
        policy.unbound_lamports(instruction, &instructions[position + 1..]).map(|lamports| total.saturating_add(lamports))
    })?;
    let wrapped_lamports = instructions
        .iter()
        .enumerate()
        .fold(0u64, |total, (position, instruction)| total.saturating_add(policy.wrapped_lamports(instruction, &instructions[position + 1..])));
    let fees: Vec<(u64, u64)> = limits
        .fees
        .iter()
        .map(|(mint, limit)| (instructions.iter().fold(0u64, |total, instruction| total.saturating_add(policy.gem_fee(instruction, mint))), *limit))
        .filter(|(fee, _)| *fee > 0)
        .collect();
    let fee_within_limit = match fees.as_slice() {
        [] => true,
        [(fee, limit)] => fee <= limit,
        _ => false,
    };
    if unbound_lamports > MAX_UNBOUND_LAMPORTS || wrapped_lamports > limits.wrapped_sol || !fee_within_limit {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

impl StepPolicy<'_> {
    fn unbound_lamports(&self, instruction: &StandardInstruction, later: &[StandardInstruction]) -> Result<u64, SwapperError> {
        match *instruction {
            StandardInstruction::ComputeBudget { unit_price: Some(price) } if price > MAX_COMPUTE_UNIT_PRICE => Err(SwapperError::InvalidRoute),
            StandardInstruction::ComputeBudget { .. } | StandardInstruction::Memo | StandardInstruction::SyncNative => Ok(0),
            StandardInstruction::Transfer { destination: Some(destination), .. } if self.returns_to_wallet(destination, later) || (*destination == self.referral && self.takes_fee(&wrapped_sol::wrapped_sol_mint())) => Ok(0),
            StandardInstruction::Transfer { lamports, .. } => Ok(lamports),
            StandardInstruction::CreateTokenAccount { token_program: Some(program), .. } if ![token_program(), token_2022_program()].contains(program) => Err(SwapperError::InvalidRoute),
            StandardInstruction::CreateTokenAccount { owner, .. } if owner == Some(self.wallet) => Ok(0),
            StandardInstruction::CreateTokenAccount { .. } => Ok(TOKEN_ACCOUNT_RENT_LAMPORTS),
            StandardInstruction::CloseAccount { destination, .. } if destination == Some(self.wallet) => Ok(0),
            StandardInstruction::TransferChecked {
                mint: Some(mint),
                destination: Some(destination),
                ..
            } if is_associated_token_address(destination, &self.referral, mint) && self.takes_fee(mint) => Ok(0),
            StandardInstruction::Program(program) if Deployment::contains(Chain::Solana, self.contracts, &program.to_base58()) => Ok(0),
            StandardInstruction::CloseAccount { .. } | StandardInstruction::TransferChecked { .. } | StandardInstruction::Program(_) | StandardInstruction::Unsupported => Err(SwapperError::InvalidRoute),
        }
    }

    fn returns_to_wallet(&self, destination: &Pubkey, later: &[StandardInstruction]) -> bool {
        *destination == self.wrapped_sol || later.iter().any(|instruction| instruction.closes_to(destination, self.wallet))
    }

    fn wrapped_lamports(&self, instruction: &StandardInstruction, later: &[StandardInstruction]) -> u64 {
        match *instruction {
            StandardInstruction::Transfer {
                destination: Some(destination), lamports, ..
            } if *destination == self.wrapped_sol && !later.iter().any(|instruction| instruction.closes_to(destination, self.wallet)) => lamports,
            StandardInstruction::ComputeBudget { .. }
            | StandardInstruction::Memo
            | StandardInstruction::Transfer { .. }
            | StandardInstruction::CreateTokenAccount { .. }
            | StandardInstruction::SyncNative
            | StandardInstruction::CloseAccount { .. }
            | StandardInstruction::TransferChecked { .. }
            | StandardInstruction::Program(_)
            | StandardInstruction::Unsupported => 0,
        }
    }

    fn takes_fee(&self, mint: &Pubkey) -> bool {
        self.fee_limits.iter().any(|(limited, _)| limited == mint)
    }

    fn gem_fee(&self, instruction: &StandardInstruction, mint: &Pubkey) -> u64 {
        match *instruction {
            StandardInstruction::Transfer {
                destination: Some(destination), lamports, ..
            } if *destination == self.referral && *mint == wrapped_sol::wrapped_sol_mint() => lamports,
            StandardInstruction::TransferChecked {
                mint: Some(paid),
                destination: Some(destination),
                amount,
                ..
            } if paid == mint && is_associated_token_address(destination, &self.referral, mint) => amount,
            StandardInstruction::ComputeBudget { .. }
            | StandardInstruction::Memo
            | StandardInstruction::Transfer { .. }
            | StandardInstruction::CreateTokenAccount { .. }
            | StandardInstruction::SyncNative
            | StandardInstruction::CloseAccount { .. }
            | StandardInstruction::TransferChecked { .. }
            | StandardInstruction::Program(_)
            | StandardInstruction::Unsupported => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::SolanaTransaction;
    use crate::{QuoteRequest, SwapperQuoteAsset};
    use gem_encoding::{decode_base58, encode_base64};
    use gem_solana::{
        AccountMeta, Instruction, USDC_TOKEN_MINT,
        associated_token::{create_associated_token_account_idempotent, get_associated_token_address_with_program_id},
        compute_budget,
        instructions::program_ids::system_program,
        system, token,
    };
    use num_bigint::BigUint;
    use primitives::AssetId;
    use primitives::testkit::signer_mock::{TEST_PRIVATE_KEY_SOLANA_ADDRESS, TEST_SOLANA_SENDER};

    #[test]
    fn test_fee_limits() {
        let usdc = Pubkey::from_base58(USDC_TOKEN_MINT).unwrap();
        let quote = Quote {
            request: QuoteRequest {
                to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Solana, USDC_TOKEN_MINT)),
                ..QuoteRequest::mock(Chain::Solana, None)
            },
            from_value: BigUint::from(1_000_000_000u64),
            to_value: BigUint::from(995_000u64),
            ..Quote::mock(Chain::Solana, None)
        };

        assert_eq!(
            StepLimits::from_quote(&quote),
            Ok(StepLimits {
                fees: vec![(wrapped_sol::wrapped_sol_mint(), 5_000_000), (usdc, 5_000)],
                wrapped_sol: 1_000_000_000,
            }),
            "Gem's fee on the amount it is taken from, and no more SOL wrapped than quoted"
        );
        assert_eq!(StepLimits::from_quote(&Quote::mock(Chain::Ethereum, None)), Ok(StepLimits::default()));
    }

    #[test]
    fn test_verify_solana() {
        let okx: serde_json::Value = serde_json::from_str(include_str!("../okx/testdata/swap_sol_to_usdc.json")).unwrap();
        let okx_transaction = encode_base64(&decode_base58(okx["data"][0]["tx"]["data"].as_str().unwrap()).unwrap());
        let okx_wallet = decode_transaction(&okx_transaction).unwrap().account_keys()[0].to_base58();
        assert_eq!(
            verify_solana(&okx_transaction, &okx_wallet, &["proVF4pMXVaYqmy4NjniPh4pqKNfMmsihgd4wdkCX3u"], &StepLimits::default()),
            Ok(()),
            "OKX's program plus the standard steps"
        );
        assert_eq!(
            verify_solana(&okx_transaction, &okx_wallet, &["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"], &StepLimits::default()),
            Err(SwapperError::InvalidRoute),
            "a transaction calling another program is refused"
        );

        let wallet = Pubkey::from_base58(TEST_SOLANA_SENDER).unwrap();
        let stranger = Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap();
        let referral = Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap();
        let usdc = Pubkey::from_base58(USDC_TOKEN_MINT).unwrap();
        let wrapped_sol = wrapped_sol::wrapped_sol_account(&wallet).unwrap();
        let temporary = Pubkey::new([5; 32]);
        let wallet_usdc = get_associated_token_address_with_program_id(&wallet, &usdc, &token_program()).unwrap();
        let referral_usdc = get_associated_token_address_with_program_id(&referral, &usdc, &token_program()).unwrap();
        let stranger_usdc = get_associated_token_address_with_program_id(&stranger, &usdc, &token_program()).unwrap();
        let limits = StepLimits {
            fees: vec![(wrapped_sol::wrapped_sol_mint(), 5_000), (usdc, 1)],
            wrapped_sol: 1_000_000_000,
        };

        for (instructions, reason) in [
            (
                vec![
                    create_associated_token_account_idempotent(&wallet, &wallet, &wrapped_sol::wrapped_sol_mint(), &token_program()).unwrap(),
                    system::transfer(&wallet, &wrapped_sol, 1_000_000_000),
                    token::sync_native(&wrapped_sol),
                    token::close_account(&wrapped_sol, &wallet, &wallet),
                ],
                "wrapping SOL into the wallet's own account and closing it back",
            ),
            (
                vec![system::transfer(&wallet, &wrapped_sol, 1_000_000_000), token::sync_native(&wrapped_sol)],
                "SOL wrapped into the wallet's own account and kept there",
            ),
            (vec![system::transfer(&wallet, &referral, 5_000)], "Gem's fee in SOL"),
            (vec![token::transfer_checked_with_program_id(&wallet_usdc, &usdc, &referral_usdc, &wallet, 1, 6, &token_program())], "Gem's fee in a token"),
            (
                vec![system::transfer(&wallet, &temporary, 500_000_000), token::close_account(&temporary, &wallet, &wallet)],
                "SOL sent to an account the swap closes back to the wallet",
            ),
            (
                vec![
                    system::transfer(&wallet, &stranger, TOKEN_ACCOUNT_RENT_LAMPORTS),
                    create_associated_token_account_idempotent(&wallet, &stranger, &usdc, &token_program()).unwrap(),
                ],
                "rent for two token accounts the check cannot tie to the wallet",
            ),
        ] {
            let transaction = SolanaTransaction::new(instructions, vec![]).encode(TEST_SOLANA_SENDER, &[]).unwrap();
            assert_eq!(verify_solana(&transaction.data, TEST_SOLANA_SENDER, &[], &limits), Ok(()), "{reason}");
        }

        let assign = Instruction {
            program_id: system_program(),
            accounts: vec![AccountMeta::new_signer_writable(wallet)],
            data: [[1, 0, 0, 0].as_slice(), stranger.as_bytes()].concat(),
        };
        let set_authority = Instruction {
            program_id: token_program(),
            accounts: vec![AccountMeta::new_writable(wallet_usdc), AccountMeta::new_signer(wallet)],
            data: [[6, 2, 1].as_slice(), stranger.as_bytes()].concat(),
        };
        for (instructions, reason) in [
            (vec![system::transfer(&wallet, &stranger, MAX_UNBOUND_LAMPORTS + 1)], "SOL sent to someone else"),
            (
                vec![system::transfer(&wallet, &wrapped_sol, 1_000_000_001), token::sync_native(&wrapped_sol)],
                "more SOL wrapped into the wallet's own account than quoted, and kept there",
            ),
            (
                vec![
                    create_associated_token_account_idempotent(&wallet, &stranger, &usdc, &token_program()).unwrap(),
                    create_associated_token_account_idempotent(&wallet, &stranger, &wrapped_sol::wrapped_sol_mint(), &token_program()).unwrap(),
                    system::transfer(&wallet, &stranger, 1),
                ],
                "more rent than two accounts need, spent outside the wallet",
            ),
            (
                vec![token::close_account(&temporary, &wallet, &wallet), system::transfer(&wallet, &temporary, 500_000_000)],
                "SOL sent to an account after it is closed",
            ),
            (vec![token::close_account(&wallet_usdc, &stranger, &wallet)], "an account closed to someone else"),
            (vec![token::transfer(&wallet_usdc, &stranger_usdc, &wallet, 1)], "a token moved out of the wallet"),
            (
                vec![token::transfer_checked_with_program_id(&wallet_usdc, &usdc, &stranger_usdc, &wallet, 1, 6, &token_program())],
                "a token fee paid to someone other than Gem",
            ),
            (vec![set_authority], "a token account handed to another key"),
            (
                vec![create_associated_token_account_idempotent(&wallet, &wallet, &usdc, &stranger).unwrap()],
                "an account created under another token program, which the network would reject",
            ),
            (vec![assign], "the wallet reassigned to another program"),
            (vec![compute_budget::set_compute_unit_price(MAX_COMPUTE_UNIT_PRICE + 1)], "a priority fee above the limit"),
            (vec![system::transfer(&wallet, &referral, MAX_UNBOUND_LAMPORTS + 1)], "a fee to Gem above the amount quoted"),
            (
                vec![system::transfer(&wallet, &referral, 5_000), system::transfer(&wallet, &referral, 1)],
                "fees to Gem that add up to more than the amount quoted",
            ),
            (
                vec![token::transfer_checked_with_program_id(&wallet_usdc, &usdc, &referral_usdc, &wallet, 2, 6, &token_program())],
                "a token fee to Gem above the amount quoted",
            ),
            (
                vec![
                    system::transfer(&wallet, &referral, 5_000),
                    token::transfer_checked_with_program_id(&wallet_usdc, &usdc, &referral_usdc, &wallet, 1, 6, &token_program()),
                ],
                "Gem's fee taken in both tokens, which no quote charges",
            ),
        ] {
            let transaction = SolanaTransaction::new(instructions, vec![]).encode(TEST_SOLANA_SENDER, &[]).unwrap();
            assert_eq!(verify_solana(&transaction.data, TEST_SOLANA_SENDER, &[], &limits), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }
}
