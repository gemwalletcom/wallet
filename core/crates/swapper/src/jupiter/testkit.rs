use super::model::{BuildRequest, BuildResponse};
use super::route::minimum_out;
use super::swap_instructions::{QuotedRoute, RouteDestination, SwapInstructions};
use gem_solana::{AddressLookupTableAccount, Pubkey, TOKEN_PROGRAM, USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS, token_account::get_token_account, wrapped_sol::wrapped_sol_mint};
use primitives::swap::SOLANA_REFERRAL_ADDRESS;

pub const TEST_PAYER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";

impl BuildRequest {
    pub fn mock() -> Self {
        Self {
            input_mint: WSOL_TOKEN_ADDRESS.to_string(),
            output_mint: USDC_TOKEN_MINT.to_string(),
            amount: "100000000".to_string(),
            taker: TEST_PAYER.to_string(),
            slippage_bps: 100,
            platform_fee_bps: 50,
            fee_account: get_token_account(SOLANA_REFERRAL_ADDRESS, USDC_TOKEN_MINT, TOKEN_PROGRAM).unwrap(),
            max_accounts: 64,
        }
    }

    pub fn mock_usdc_sol() -> Self {
        Self {
            input_mint: USDC_TOKEN_MINT.to_string(),
            output_mint: WSOL_TOKEN_ADDRESS.to_string(),
            amount: "20000000".to_string(),
            ..Self::mock()
        }
    }
}

impl BuildResponse {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("testdata/build_sol_usdc.json")).unwrap()
    }

    pub fn mock_usdc_sol() -> Self {
        serde_json::from_str(include_str!("testdata/build_usdc_sol.json")).unwrap()
    }

    pub fn mock_lookup_tables(&self) -> Vec<AddressLookupTableAccount> {
        self.addresses_by_lookup_table_address
            .iter()
            .flatten()
            .map(|(key, addresses)| AddressLookupTableAccount::new(*key, addresses.clone()))
            .collect()
    }
}

impl SwapInstructions {
    pub fn mock() -> Self {
        Self::from_build(BuildResponse::mock())
    }

    pub fn mock_usdc_sol() -> Self {
        Self::from_build(BuildResponse::mock_usdc_sol())
    }

    fn from_build(build: BuildResponse) -> Self {
        Self::new(build.compute_budget_instructions, build.setup_instructions, build.swap_instruction, build.cleanup_instruction).unwrap()
    }
}

impl QuotedRoute {
    pub fn mock() -> Self {
        Self {
            wallet: Pubkey::from_base58(TEST_PAYER).unwrap(),
            input_mint: wrapped_sol_mint(),
            amount: 100_000_000,
            minimum_out: minimum_out(6_829_563, 100),
            platform_fee_bps: 50,
            destination: RouteDestination::WalletAccount(Pubkey::from_base58(USDC_TOKEN_MINT).unwrap()),
        }
    }

    pub fn mock_usdc_sol() -> Self {
        Self {
            input_mint: Pubkey::from_base58(USDC_TOKEN_MINT).unwrap(),
            amount: 20_000_000,
            minimum_out: minimum_out(289_924_882, 100),
            destination: RouteDestination::WalletAccount(wrapped_sol_mint()),
            ..Self::mock()
        }
    }
}
