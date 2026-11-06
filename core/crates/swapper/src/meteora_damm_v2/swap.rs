use gem_solana::{AccountMeta, Instruction, Pubkey, anchor::instruction_data, instructions::program_ids::meteora_damm_v2_program};
use primitives::decode_hex_literal;

use super::pool::Pool;
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap},
};

const SWAP: &str = "swap";
const POOL_AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("f2ccd535aca5f1736ac822dd0773e4d92fbd8a59b29403500295a9011c73a9e5"));
const EVENT_AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("2a76e7b344640a1cfc594c8bcad0a091011cac7dd156bf83a833fb2208eb77ad"));

pub(super) fn swap_instruction(address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let a_to_b = pool.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let (token_a, token_b) = if a_to_b { (swap.input, swap.output) } else { (swap.output, swap.input) };
    let accounts = vec![
        AccountMeta::new_readonly(POOL_AUTHORITY),
        AccountMeta::new_writable(*address),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(swap.output.account(&swap.authority)?),
        AccountMeta::new_writable(pool.token_a_vault),
        AccountMeta::new_writable(pool.token_b_vault),
        AccountMeta::new_readonly(pool.token_a_mint),
        AccountMeta::new_readonly(pool.token_b_mint),
        AccountMeta::new_signer(swap.authority),
        AccountMeta::new_readonly(token_a.token_program),
        AccountMeta::new_readonly(token_b.token_program),
        AccountMeta::new_readonly(meteora_damm_v2_program()),
        AccountMeta::new_readonly(EVENT_AUTHORITY),
        AccountMeta::new_readonly(meteora_damm_v2_program()),
    ];
    Ok(Instruction {
        program_id: meteora_damm_v2_program(),
        accounts,
        data: instruction_data(SWAP, &swap.amounts()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        meteora_damm_v2::testkit::{TEST_BOTH_TOKEN_POOL, mock_pool_snapshot},
        solana_dex::SwapToken,
    };
    use gem_solana::{find_program_address, instructions::program_ids::token_program, wrapped_sol::wrapped_sol_mint};
    use primitives::hex;

    #[test]
    fn test_program_addresses() {
        let address = |seed: &[u8]| find_program_address(&meteora_damm_v2_program(), &[seed]).unwrap().0;

        assert_eq!(address(b"pool_authority"), POOL_AUTHORITY);
        assert_eq!(address(b"__event_authority"), EVENT_AUTHORITY);
    }

    #[test]
    fn test_swap_instruction() {
        let snapshot = mock_pool_snapshot(TEST_BOTH_TOKEN_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let sol = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let tos = SwapToken {
            mint: pool.token_a_mint,
            ..SwapToken::mock()
        };
        let sell = PoolSwap {
            authority,
            input: &tos,
            intermediate: None,
            output: &sol,
            amount: 326_754_966,
            minimum_out: 9_836_774,
        };
        let buy = PoolSwap { input: &sol, output: &tos, ..sell };
        let instruction = swap_instruction(&address, &pool, &sell).unwrap();
        let swapped_user_accounts = swap_instruction(&address, &pool, &buy).unwrap().accounts;

        assert_eq!(instruction.program_id, meteora_damm_v2_program());
        assert_eq!(
            instruction.accounts,
            vec![
                AccountMeta::new_readonly(POOL_AUTHORITY),
                AccountMeta::new_writable(address),
                AccountMeta::new_writable(tos.account(&authority).unwrap()),
                AccountMeta::new_writable(sol.account(&authority).unwrap()),
                AccountMeta::new_writable(pool.token_a_vault),
                AccountMeta::new_writable(pool.token_b_vault),
                AccountMeta::new_readonly(tos.mint),
                AccountMeta::new_readonly(sol.mint),
                AccountMeta::new_signer(authority),
                AccountMeta::new_readonly(token_program()),
                AccountMeta::new_readonly(token_program()),
                AccountMeta::new_readonly(meteora_damm_v2_program()),
                AccountMeta::new_readonly(EVENT_AUTHORITY),
                AccountMeta::new_readonly(meteora_damm_v2_program()),
            ]
        );
        assert_eq!(instruction.data, hex::decode("f8c69e91e17587c896e2791300000000e618960000000000").unwrap());
        assert_eq!(swapped_user_accounts[2..4], [instruction.accounts[3].clone(), instruction.accounts[2].clone()]);
        assert_eq!(swap_instruction(&address, &pool, &PoolSwap { output: &tos, ..sell }), Err(SwapperError::InvalidRoute));
    }
}
