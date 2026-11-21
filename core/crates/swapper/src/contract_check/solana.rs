use gem_solana::{
    CompiledInstruction, Instruction, Pubkey,
    associated_token::{is_associated_token_address, is_create_account_data},
    compute_budget, decode_transaction, system, token, wrapped_sol,
};
use primitives::{
    Chain,
    contract_constants::{SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SOLANA_COMPUTE_BUDGET_PROGRAM_ID, SOLANA_MEMO_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID, SOLANA_TOKEN_2022_PROGRAM_ID, SOLANA_TOKEN_PROGRAM_ID},
    swap::SOLANA_REFERRAL_ADDRESS,
};

use crate::{Deployment, SwapperError};

const MAX_COMPUTE_UNIT_PRICE: u64 = 100_000_000;
const TOKEN_ACCOUNT_RENT_LAMPORTS: u64 = 2_039_280;
const MAX_UNBOUND_LAMPORTS: u64 = 2 * TOKEN_ACCOUNT_RENT_LAMPORTS;

pub(crate) struct SolanaStep<'a> {
    program: &'a Pubkey,
    accounts: Vec<Option<&'a Pubkey>>,
    data: &'a [u8],
}

pub(crate) enum StandardStep<'a> {
    ComputeBudget { unit_price: Option<u64> },
    Memo,
    Transfer { destination: Option<&'a Pubkey>, lamports: u64 },
    CreateTokenAccount { owner: Option<&'a Pubkey> },
    SyncNative,
    CloseAccount { account: Option<&'a Pubkey>, destination: Option<&'a Pubkey> },
    TransferChecked { mint: Option<&'a Pubkey>, destination: Option<&'a Pubkey> },
    Program(&'a Pubkey),
    Unsupported,
}

struct StepPolicy<'a> {
    wallet: &'a Pubkey,
    wrapped_sol: Pubkey,
    referral: Pubkey,
    contracts: &'a [&'a str],
}

pub(crate) fn verify_solana(transaction: &str, wallet: &str, contracts: &[&str]) -> Result<(), SwapperError> {
    let transaction = decode_transaction(transaction).map_err(SwapperError::transaction_error)?;
    let keys = transaction.account_keys();
    let steps = transaction
        .instructions()
        .iter()
        .map(|instruction| SolanaStep::compiled(instruction, keys))
        .collect::<Option<Vec<_>>>()
        .ok_or(SwapperError::InvalidRoute)?;
    verify_steps(&steps, &Pubkey::from_base58(wallet).map_err(SwapperError::transaction_error)?, contracts)
}

pub(crate) fn verify_steps(steps: &[SolanaStep], wallet: &Pubkey, contracts: &[&str]) -> Result<(), SwapperError> {
    let policy = StepPolicy {
        wallet,
        wrapped_sol: wrapped_sol::wrapped_sol_account(wallet).map_err(SwapperError::transaction_error)?,
        referral: Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).map_err(SwapperError::transaction_error)?,
        contracts,
    };
    let decoded: Vec<StandardStep> = steps.iter().map(SolanaStep::decode).collect();
    let unbound_lamports = decoded
        .iter()
        .enumerate()
        .try_fold(0u64, |total, (position, step)| policy.unbound_lamports(step, &decoded[position + 1..]).map(|lamports| total.saturating_add(lamports)))?;
    if unbound_lamports > MAX_UNBOUND_LAMPORTS {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

impl<'a> SolanaStep<'a> {
    pub(crate) fn new(instruction: &'a Instruction) -> Self {
        Self {
            program: &instruction.program_id,
            accounts: instruction.accounts.iter().map(|account| Some(&account.pubkey)).collect(),
            data: &instruction.data,
        }
    }

    fn compiled(instruction: &'a CompiledInstruction, keys: &'a [Pubkey]) -> Option<Self> {
        Some(Self {
            program: keys.get(usize::from(instruction.program_id_index))?,
            accounts: instruction.accounts.iter().map(|index| keys.get(usize::from(*index))).collect(),
            data: &instruction.data,
        })
    }

    fn account(&self, index: usize) -> Option<&'a Pubkey> {
        self.accounts.get(index).copied().flatten()
    }

    pub(crate) fn decode(&self) -> StandardStep<'a> {
        match self.program.to_base58().as_str() {
            SOLANA_COMPUTE_BUDGET_PROGRAM_ID => StandardStep::ComputeBudget {
                unit_price: compute_budget::parse_compute_unit_price_data(self.data),
            },
            SOLANA_MEMO_PROGRAM_ID => StandardStep::Memo,
            SOLANA_SYSTEM_PROGRAM_ID => match system::transfer_lamports(self.data) {
                Some(lamports) => StandardStep::Transfer { destination: self.account(1), lamports },
                None => StandardStep::Unsupported,
            },
            SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID => match (is_create_account_data(self.data), self.account(5).map(Pubkey::to_base58).as_deref()) {
                (true, Some(SOLANA_TOKEN_PROGRAM_ID | SOLANA_TOKEN_2022_PROGRAM_ID) | None) => StandardStep::CreateTokenAccount { owner: self.account(2) },
                _ => StandardStep::Unsupported,
            },
            SOLANA_TOKEN_PROGRAM_ID | SOLANA_TOKEN_2022_PROGRAM_ID => match self.data {
                [token::SYNC_NATIVE_DISCRIMINANT] => StandardStep::SyncNative,
                [token::CLOSE_ACCOUNT_DISCRIMINANT] => StandardStep::CloseAccount {
                    account: self.account(0),
                    destination: self.account(1),
                },
                [token::TRANSFER_CHECKED_DISCRIMINANT, ..] => StandardStep::TransferChecked {
                    mint: self.account(1),
                    destination: self.account(2),
                },
                _ => StandardStep::Unsupported,
            },
            _ => StandardStep::Program(self.program),
        }
    }
}

impl StandardStep<'_> {
    pub(crate) fn transfers_to(&self, account: &Pubkey) -> bool {
        match self {
            Self::Transfer { destination, .. } => *destination == Some(account),
            Self::ComputeBudget { .. } | Self::Memo | Self::CreateTokenAccount { .. } | Self::SyncNative | Self::CloseAccount { .. } | Self::TransferChecked { .. } | Self::Program(_) | Self::Unsupported => false,
        }
    }

    fn closes_to(&self, account: &Pubkey, destination: &Pubkey) -> bool {
        match self {
            Self::CloseAccount { account: closed, destination: paid } => *closed == Some(account) && *paid == Some(destination),
            Self::ComputeBudget { .. } | Self::Memo | Self::Transfer { .. } | Self::CreateTokenAccount { .. } | Self::SyncNative | Self::TransferChecked { .. } | Self::Program(_) | Self::Unsupported => false,
        }
    }
}

impl StepPolicy<'_> {
    fn unbound_lamports(&self, step: &StandardStep, later: &[StandardStep]) -> Result<u64, SwapperError> {
        match step {
            StandardStep::ComputeBudget { unit_price: Some(price) } if *price > MAX_COMPUTE_UNIT_PRICE => Err(SwapperError::InvalidRoute),
            StandardStep::ComputeBudget { .. } | StandardStep::Memo | StandardStep::SyncNative => Ok(0),
            StandardStep::Transfer { destination: Some(destination), .. } if self.returns_to_wallet_or_gem(destination, later) => Ok(0),
            StandardStep::Transfer { lamports, .. } => Ok(*lamports),
            StandardStep::CreateTokenAccount { owner } if *owner == Some(self.wallet) => Ok(0),
            StandardStep::CreateTokenAccount { .. } => Ok(TOKEN_ACCOUNT_RENT_LAMPORTS),
            StandardStep::CloseAccount { destination, .. } if *destination == Some(self.wallet) => Ok(0),
            StandardStep::TransferChecked {
                mint: Some(mint),
                destination: Some(destination),
            } if is_associated_token_address(destination, &self.referral, mint) => Ok(0),
            StandardStep::Program(program) if Deployment::contains(Chain::Solana, self.contracts, &program.to_base58()) => Ok(0),
            StandardStep::CloseAccount { .. } | StandardStep::TransferChecked { .. } | StandardStep::Program(_) | StandardStep::Unsupported => Err(SwapperError::InvalidRoute),
        }
    }

    fn returns_to_wallet_or_gem(&self, destination: &Pubkey, later: &[StandardStep]) -> bool {
        *destination == self.wrapped_sol || *destination == self.referral || later.iter().any(|step| step.closes_to(destination, self.wallet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::SolanaTransaction;
    use gem_encoding::{decode_base58, encode_base64};
    use gem_solana::{
        AccountMeta, Instruction, USDC_TOKEN_MINT,
        associated_token::{create_associated_token_account_idempotent, get_associated_token_address_with_program_id},
        instructions::program_ids::{system_program, token_program},
    };
    use primitives::testkit::signer_mock::{TEST_PRIVATE_KEY_SOLANA_ADDRESS, TEST_SOLANA_SENDER};

    #[test]
    fn test_verify_solana() {
        let okx: serde_json::Value = serde_json::from_str(include_str!("../okx/testdata/swap_sol_to_usdc.json")).unwrap();
        let okx_transaction = encode_base64(&decode_base58(okx["data"][0]["tx"]["data"].as_str().unwrap()).unwrap());
        let okx_wallet = decode_transaction(&okx_transaction).unwrap().account_keys()[0].to_base58();
        assert_eq!(verify_solana(&okx_transaction, &okx_wallet, &["proVF4pMXVaYqmy4NjniPh4pqKNfMmsihgd4wdkCX3u"]), Ok(()), "OKX's program plus the standard steps");
        assert_eq!(
            verify_solana(&okx_transaction, &okx_wallet, &["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"]),
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
            assert_eq!(verify_solana(&transaction.data, TEST_SOLANA_SENDER, &[]), Ok(()), "{reason}");
        }

        let assign = Instruction {
            program_id: system_program(),
            accounts: vec![AccountMeta::new_signer_writable(wallet)],
            data: [[1, 0, 0, 0].as_slice(), stranger.as_bytes()].concat(),
        };
        let set_authority = Instruction {
            program_id: token_program(),
            accounts: vec![AccountMeta::new_writable(wallet_usdc), AccountMeta::new_signer(wallet)],
            data: [vec![token::SET_AUTHORITY_DISCRIMINANT, 2, 1], stranger.as_bytes().to_vec()].concat(),
        };
        for (instructions, reason) in [
            (vec![system::transfer(&wallet, &stranger, MAX_UNBOUND_LAMPORTS + 1)], "SOL sent to someone else"),
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
                "an account created under another token program, which sets its size and owns its rent",
            ),
            (vec![assign], "the wallet reassigned to another program"),
            (vec![compute_budget::set_compute_unit_price(MAX_COMPUTE_UNIT_PRICE + 1)], "a priority fee above the limit"),
        ] {
            let transaction = SolanaTransaction::new(instructions, vec![]).encode(TEST_SOLANA_SENDER, &[]).unwrap();
            assert_eq!(verify_solana(&transaction.data, TEST_SOLANA_SENDER, &[]), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }
}
