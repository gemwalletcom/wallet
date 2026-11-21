use super::model::{BuildRequest, BuildResponse};
use super::swap_instructions::SwapInstructions;
use gem_solana::{TOKEN_PROGRAM, USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS, token_account::get_token_account};
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
        serde_json::from_str(include_str!("test/build_sol_usdc.json")).unwrap()
    }

    pub fn mock_usdc_sol() -> Self {
        serde_json::from_str(include_str!("test/build_usdc_sol.json")).unwrap()
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
