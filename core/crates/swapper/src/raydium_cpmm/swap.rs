use gem_solana::{AccountMeta, Instruction, Pubkey, anchor::instruction_data, instructions::program_ids::raydium_cpmm_program};
use primitives::decode_hex_literal;

use super::pool::Pool;
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap},
};

const SWAP_BASE_INPUT: &str = "swap_base_input";
const AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("eb00d9f5b292b4214ac7d037b4d6f06450b964600df373052bb5e84f2f8e9a67"));

pub(super) fn swap_base_input_instruction(address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let (input_vault, output_vault) = pool.vaults(pool.swap_direction(&swap.input.mint, &swap.output.mint)?);
    let accounts = vec![
        AccountMeta::new_signer(swap.authority),
        AccountMeta::new_readonly(AUTHORITY),
        AccountMeta::new_readonly(pool.amm_config),
        AccountMeta::new_writable(*address),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(swap.output.account(&swap.authority)?),
        AccountMeta::new_writable(input_vault),
        AccountMeta::new_writable(output_vault),
        AccountMeta::new_readonly(swap.input.token_program),
        AccountMeta::new_readonly(swap.output.token_program),
        AccountMeta::new_readonly(swap.input.mint),
        AccountMeta::new_readonly(swap.output.mint),
        AccountMeta::new_writable(pool.observation_key),
    ];
    Ok(Instruction {
        program_id: raydium_cpmm_program(),
        accounts,
        data: instruction_data(SWAP_BASE_INPUT, &swap.amounts()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        raydium_cpmm::testkit::{TEST_JUP_SOL_POOL, mock_pool_snapshot},
        solana_dex::SwapToken,
    };
    use gem_solana::{anchor::global_discriminator, find_program_address, wrapped_sol::wrapped_sol_mint};
    use primitives::hex;

    #[test]
    fn test_program_addresses() {
        let snapshot = mock_pool_snapshot(TEST_JUP_SOL_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let pda = |seeds: &[&[u8]]| find_program_address(&raydium_cpmm_program(), seeds).unwrap().0;

        assert_eq!(pda(&[b"vault_and_lp_mint_auth_seed"]), AUTHORITY);
        assert_eq!(pda(&[b"pool_vault", address.as_bytes(), pool.token_0_mint.as_bytes()]), pool.token_0_vault);
        assert_eq!(pda(&[b"observation", address.as_bytes()]), pool.observation_key);
        assert_eq!(global_discriminator(SWAP_BASE_INPUT), decode_hex_literal("8fbe5adac41e33de"));
    }

    #[test]
    fn test_swap_instruction() {
        let snapshot = mock_pool_snapshot(TEST_JUP_SOL_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let sol = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let jup = SwapToken {
            mint: pool.token_0_mint,
            ..SwapToken::mock()
        };
        let swap = PoolSwap {
            authority,
            input: &sol,
            intermediate: None,
            output: &jup,
            amount: 1_000_000_000,
            minimum_out: 340_000_000,
        };

        let instruction = swap_base_input_instruction(&address, &pool, &swap).unwrap();
        let accounts = instruction.accounts.iter().map(|account| (account.pubkey, account.is_signer, account.is_writable)).collect::<Vec<_>>();

        assert_eq!(instruction.program_id, raydium_cpmm_program());
        assert_eq!(
            instruction.data,
            [hex::decode("8fbe5adac41e33de").unwrap().to_vec(), 1_000_000_000u64.to_le_bytes().to_vec(), 340_000_000u64.to_le_bytes().to_vec()].concat()
        );
        assert_eq!(
            accounts,
            vec![
                (authority, true, false),
                (AUTHORITY, false, false),
                (pool.amm_config, false, false),
                (address, false, true),
                (sol.account(&authority).unwrap(), false, true),
                (jup.account(&authority).unwrap(), false, true),
                (pool.token_1_vault, false, true),
                (pool.token_0_vault, false, true),
                (sol.token_program, false, false),
                (jup.token_program, false, false),
                (sol.mint, false, false),
                (jup.mint, false, false),
                (pool.observation_key, false, true),
            ]
        );
    }
}
