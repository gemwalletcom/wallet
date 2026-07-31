use std::collections::BTreeMap;

use primitives::SolanaInstruction;
use serde::{Deserialize, Serialize};
use solana_primitives::Pubkey;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub taker: String,
    pub slippage_bps: u32,
    pub platform_fee_bps: u32,
    pub fee_account: String,
    pub max_accounts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u32,
    pub compute_budget_instructions: Vec<SolanaInstruction>,
    pub setup_instructions: Vec<SolanaInstruction>,
    pub swap_instruction: SolanaInstruction,
    pub cleanup_instruction: Option<SolanaInstruction>,
    pub other_instructions: Vec<SolanaInstruction>,
    pub tip_instruction: Option<SolanaInstruction>,
    pub addresses_by_lookup_table_address: Option<BTreeMap<Pubkey, Vec<Pubkey>>>,
    pub blockhash_with_metadata: BlockhashWithMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockhashWithMetadata {
    pub blockhash: [u8; 32],
}
