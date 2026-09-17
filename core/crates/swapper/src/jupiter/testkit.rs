use super::model::{BlockhashWithMetadata, BuildResponse};
use gem_solana::JUPITER_PROGRAM_ID;
use primitives::{SolanaAccountMeta, SolanaInstruction};
use std::collections::BTreeMap;

pub const TEST_PAYER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
pub const TEST_FEE_ACCOUNT: &str = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";

impl BuildResponse {
    pub fn mock() -> Self {
        Self {
            out_amount: "125000000".to_string(),
            slippage_bps: 100,
            compute_budget_instructions: Vec::new(),
            setup_instructions: Vec::new(),
            swap_instruction: SolanaInstruction {
                program_id: JUPITER_PROGRAM_ID.to_string(),
                accounts: vec![
                    SolanaAccountMeta {
                        pubkey: TEST_PAYER.to_string(),
                        is_signer: true,
                        is_writable: true,
                    },
                    SolanaAccountMeta {
                        pubkey: TEST_FEE_ACCOUNT.to_string(),
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: String::new(),
            },
            cleanup_instruction: None,
            other_instructions: Vec::new(),
            tip_instruction: None,
            addresses_by_lookup_table_address: Some(BTreeMap::new()),
            blockhash_with_metadata: BlockhashWithMetadata { blockhash: [0; 32] },
        }
    }
}
