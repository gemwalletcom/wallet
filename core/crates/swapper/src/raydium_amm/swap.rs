use borsh::to_vec;
use gem_solana::{
    AccountMeta, Instruction, Pubkey,
    instructions::program_ids::{raydium_amm_program, token_program},
};
use primitives::decode_hex_literal;

use super::pool::Pool;
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap},
};

const SWAP_BASE_IN_V2: u8 = 16;
const AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("4157b0580f31c5fce44a62582dbcf9d78ee75943a084a393b350368d22899308"));

pub(super) fn swap_base_in_v2_instruction(address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    pool.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let accounts = vec![
        AccountMeta::new_readonly(token_program()),
        AccountMeta::new_writable(*address),
        AccountMeta::new_readonly(AUTHORITY),
        AccountMeta::new_writable(pool.coin_vault),
        AccountMeta::new_writable(pool.pc_vault),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(swap.output.account(&swap.authority)?),
        AccountMeta::new_signer(swap.authority),
    ];
    Ok(Instruction {
        program_id: raydium_amm_program(),
        accounts,
        data: [vec![SWAP_BASE_IN_V2], to_vec(&swap.amounts()).unwrap_or_default()].concat(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        raydium_amm::{
            pool::AUTHORITY_NONCE,
            testkit::{TEST_SOL_USDC_POOL, mock_pool_snapshot},
        },
        solana_dex::SwapToken,
    };
    use gem_solana::{find_program_address, wrapped_sol::wrapped_sol_mint};

    #[test]
    fn test_authority() {
        assert_eq!(find_program_address(&raydium_amm_program(), &[b"amm authority"]).unwrap(), (AUTHORITY, u8::try_from(AUTHORITY_NONCE).unwrap()));
    }

    #[test]
    fn test_swap_instruction() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let sol = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let usdc = SwapToken { mint: pool.pc_mint, ..SwapToken::mock() };
        let swap = PoolSwap {
            authority,
            input: &usdc,
            intermediate: None,
            output: &sol,
            amount: 1_000_000,
            minimum_out: 9_000_000,
        };

        let instruction = swap_base_in_v2_instruction(&address, &pool, &swap).unwrap();
        let accounts = instruction.accounts.iter().map(|account| (account.pubkey, account.is_signer, account.is_writable)).collect::<Vec<_>>();

        assert_eq!(instruction.program_id, raydium_amm_program());
        assert_eq!(instruction.data, [vec![16], 1_000_000u64.to_le_bytes().to_vec(), 9_000_000u64.to_le_bytes().to_vec()].concat());
        assert_eq!(
            accounts,
            vec![
                (token_program(), false, false),
                (address, false, true),
                (AUTHORITY, false, false),
                (pool.coin_vault, false, true),
                (pool.pc_vault, false, true),
                (usdc.account(&authority).unwrap(), false, true),
                (sol.account(&authority).unwrap(), false, true),
                (authority, true, false),
            ]
        );
        assert_eq!(swap_base_in_v2_instruction(&address, &pool, &PoolSwap { output: &usdc, ..swap }).err(), Some(SwapperError::InvalidRoute));
    }
}
