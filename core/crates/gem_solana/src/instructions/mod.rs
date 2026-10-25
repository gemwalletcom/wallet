pub mod anchor;
pub mod associated_token;
pub mod compute_budget;
pub mod memo;
pub mod program_ids;
pub mod system;
pub mod token;
pub mod wrapped_sol;

use program_ids::{SOLANA_SYSTEM_PROGRAM_ID, SOLANA_TOKEN_2022_PROGRAM_ID, SOLANA_TOKEN_PROGRAM_ID};

pub fn grants_authority(program_id: &str, data: &[u8]) -> bool {
    match program_id {
        SOLANA_TOKEN_PROGRAM_ID | SOLANA_TOKEN_2022_PROGRAM_ID => token::grants_authority_data(data),
        SOLANA_SYSTEM_PROGRAM_ID => system::grants_authority_data(data),
        _ => false,
    }
}
