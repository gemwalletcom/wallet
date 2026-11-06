use borsh::BorshSerialize;
use gem_solana::{
    AccountMeta, Instruction, Pubkey,
    anchor::instruction_data,
    instructions::program_ids::{memo_program, meteora_dlmm_program},
};
use primitives::decode_hex_literal;

use super::{bin::BinArray, pair::Pair};
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap, SwapAmounts},
};

const SWAP2: &str = "swap2";
const EVENT_AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("b270d67fa98c51cf0213051358962baf35742bed59c9d9445e9c0d0c85c7cd91"));
pub(super) const SWAP_BIN_ARRAYS: usize = 3;

#[derive(BorshSerialize)]
struct SwapParameters {
    amounts: SwapAmounts,
    remaining_accounts_slices: Vec<(u8, u8)>,
}

pub(super) fn swap2_instruction(address: &Pubkey, pair: &Pair, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let swap_for_y = pair.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let (token_x_program, token_y_program) = pair.token_programs();
    let bin_arrays = pair
        .arrays_for_swap(swap_for_y, SWAP_BIN_ARRAYS)
        .into_iter()
        .map(|index| Ok(AccountMeta::new_writable(BinArray::address(address, index)?)))
        .collect::<Result<Vec<_>, SwapperError>>()?;
    let accounts = [
        vec![
            AccountMeta::new_writable(*address),
            AccountMeta::new_readonly(meteora_dlmm_program()),
            AccountMeta::new_writable(pair.reserve_x),
            AccountMeta::new_writable(pair.reserve_y),
            AccountMeta::new_writable(swap.input.account(&swap.authority)?),
            AccountMeta::new_writable(swap.output.account(&swap.authority)?),
            AccountMeta::new_readonly(pair.token_x_mint),
            AccountMeta::new_readonly(pair.token_y_mint),
            AccountMeta::new_writable(pair.oracle),
            AccountMeta::new_readonly(meteora_dlmm_program()),
            AccountMeta::new_signer(swap.authority),
            AccountMeta::new_readonly(token_x_program),
            AccountMeta::new_readonly(token_y_program),
            AccountMeta::new_readonly(memo_program()),
            AccountMeta::new_readonly(EVENT_AUTHORITY),
            AccountMeta::new_readonly(meteora_dlmm_program()),
        ],
        bin_arrays,
    ]
    .concat();
    let parameters = SwapParameters {
        amounts: swap.amounts(),
        remaining_accounts_slices: vec![],
    };
    Ok(Instruction {
        program_id: meteora_dlmm_program(),
        accounts,
        data: instruction_data(SWAP2, &parameters),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        meteora_dlmm::testkit::{TEST_SOL_USDC_POOL, mock_pool_snapshot},
        solana_dex::SwapToken,
    };
    use gem_solana::{anchor::global_discriminator, find_program_address, instructions::program_ids::token_program, wrapped_sol::wrapped_sol_mint};
    use primitives::hex;

    #[test]
    fn test_program_addresses() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let (address, pair) = (snapshot.address, snapshot.pool);
        let pda = |seeds: &[&[u8]]| find_program_address(&meteora_dlmm_program(), seeds).unwrap().0;

        assert_eq!(pda(&[b"__event_authority"]), EVENT_AUTHORITY);
        assert_eq!(pda(&[b"oracle", address.as_bytes()]), pair.oracle);
        assert_eq!(pda(&[address.as_bytes(), pair.token_x_mint.as_bytes()]), pair.reserve_x);
        assert_eq!(global_discriminator(SWAP2), decode_hex_literal("414b3f4ceb5b5b88"));
    }

    #[test]
    fn test_swap_instruction() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let (address, pair) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let sol = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let usdc = SwapToken {
            mint: pair.token_y_mint,
            ..SwapToken::mock()
        };
        let swap = PoolSwap {
            authority,
            input: &sol,
            intermediate: None,
            output: &usdc,
            amount: 1_000_000_000,
            minimum_out: 100_000_000,
        };

        let instruction = swap2_instruction(&address, &pair, &swap).unwrap();
        let accounts = instruction.accounts.iter().map(|account| (account.pubkey, account.is_signer, account.is_writable)).collect::<Vec<_>>();
        let bin_arrays = [-325, -326, -327].map(|index| (BinArray::address(&address, index).unwrap(), false, true));

        assert_eq!(instruction.program_id, meteora_dlmm_program());
        assert_eq!(
            instruction.data,
            [hex::decode("414b3f4ceb5b5b88").unwrap().to_vec(), 1_000_000_000u64.to_le_bytes().to_vec(), 100_000_000u64.to_le_bytes().to_vec(), vec![0; 4]].concat()
        );
        assert_eq!(
            accounts,
            [
                vec![
                    (address, false, true),
                    (meteora_dlmm_program(), false, false),
                    (pair.reserve_x, false, true),
                    (pair.reserve_y, false, true),
                    (sol.account(&authority).unwrap(), false, true),
                    (usdc.account(&authority).unwrap(), false, true),
                    (pair.token_x_mint, false, false),
                    (pair.token_y_mint, false, false),
                    (pair.oracle, false, true),
                    (meteora_dlmm_program(), false, false),
                    (authority, true, false),
                    (token_program(), false, false),
                    (token_program(), false, false),
                    (memo_program(), false, false),
                    (EVENT_AUTHORITY, false, false),
                    (meteora_dlmm_program(), false, false),
                ],
                bin_arrays.to_vec(),
            ]
            .concat()
        );
    }
}
