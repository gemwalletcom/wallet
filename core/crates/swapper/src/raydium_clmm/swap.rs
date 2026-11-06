use borsh::BorshSerialize;
use gem_solana::{
    AccountMeta, Instruction, Pubkey,
    anchor::instruction_data,
    instructions::program_ids::{memo_program, raydium_clmm_program, token_2022_program, token_program},
};

use super::{
    pool::{PoolState, SwapTickArrays},
    tick_array::TickArray,
};
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap, SwapToken},
};

const SWAP_V2: &str = "swap_v2";
const SWAP_ROUTER_BASE_IN: &str = "swap_router_base_in";

#[derive(BorshSerialize)]
struct SwapV2Args {
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit_x64: u128,
    is_base_input: bool,
}

#[derive(BorshSerialize)]
struct SwapRouterBaseInArgs {
    amount_in: u64,
    amount_out_minimum: u64,
}

pub(super) fn swap_v2_instruction(address: &Pubkey, pool: &PoolState, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let zero_for_one = pool.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let (input_vault, output_vault) = pool.vaults(zero_for_one);
    let args = SwapV2Args {
        amount: swap.amount,
        other_amount_threshold: swap.minimum_out,
        sqrt_price_limit_x64: 0,
        is_base_input: true,
    };
    let accounts = [
        AccountMeta::new_signer(swap.authority),
        AccountMeta::new_readonly(pool.amm_config),
        AccountMeta::new_writable(*address),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(swap.output.account(&swap.authority)?),
        AccountMeta::new_writable(input_vault),
        AccountMeta::new_writable(output_vault),
        AccountMeta::new_writable(pool.observation_key),
        AccountMeta::new_readonly(token_program()),
        AccountMeta::new_readonly(token_2022_program()),
        AccountMeta::new_readonly(memo_program()),
        AccountMeta::new_readonly(swap.input.mint),
        AccountMeta::new_readonly(swap.output.mint),
    ]
    .into_iter()
    .chain(swap_tick_arrays(address, pool, zero_for_one)?)
    .collect();
    raydium_instruction(SWAP_V2, &args, accounts)
}

pub(super) fn swap_router_base_in_instruction(first: &(Pubkey, PoolState), second: &(Pubkey, PoolState), swap: &PoolSwap, intermediate: &SwapToken) -> Result<Instruction, SwapperError> {
    let args = SwapRouterBaseInArgs {
        amount_in: swap.amount,
        amount_out_minimum: swap.minimum_out,
    };
    let hops = [(first, swap.input, intermediate), (second, intermediate, swap.output)]
        .into_iter()
        .map(|((address, pool), input, output)| router_hop_accounts(address, pool, swap.authority, input, output))
        .collect::<Result<Vec<_>, SwapperError>>()?;
    let accounts = [
        AccountMeta::new_signer(swap.authority),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(swap.input.mint),
        AccountMeta::new_readonly(token_program()),
        AccountMeta::new_readonly(token_2022_program()),
        AccountMeta::new_readonly(memo_program()),
    ]
    .into_iter()
    .chain(hops.into_iter().flatten())
    .collect();
    raydium_instruction(SWAP_ROUTER_BASE_IN, &args, accounts)
}

fn router_hop_accounts(address: &Pubkey, pool: &PoolState, authority: Pubkey, input: &SwapToken, output: &SwapToken) -> Result<Vec<AccountMeta>, SwapperError> {
    let zero_for_one = pool.swap_direction(&input.mint, &output.mint)?;
    let (input_vault, output_vault) = pool.vaults(zero_for_one);
    Ok([
        AccountMeta::new_readonly(pool.amm_config),
        AccountMeta::new_writable(*address),
        AccountMeta::new_writable(output.account(&authority)?),
        AccountMeta::new_writable(input_vault),
        AccountMeta::new_writable(output_vault),
        AccountMeta::new_readonly(output.mint),
        AccountMeta::new_writable(pool.observation_key),
    ]
    .into_iter()
    .chain(swap_tick_arrays(address, pool, zero_for_one)?)
    .collect())
}

fn swap_tick_arrays(address: &Pubkey, pool: &PoolState, zero_for_one: bool) -> Result<Vec<AccountMeta>, SwapperError> {
    let tick_arrays = pool.swap_tick_arrays(zero_for_one)?;
    let bitmap_extension = match tick_arrays {
        SwapTickArrays::Initialized(_) => vec![],
        SwapTickArrays::Consecutive(_) => vec![AccountMeta::new_writable(PoolState::bitmap_extension_address(address)?)],
    };
    Ok(bitmap_extension.into_iter().chain(TickArray::addresses(address, tick_arrays.starts())?.into_iter().map(AccountMeta::new_writable)).collect())
}

fn raydium_instruction(name: &str, args: &impl BorshSerialize, accounts: Vec<AccountMeta>) -> Result<Instruction, SwapperError> {
    Ok(Instruction {
        program_id: raydium_clmm_program(),
        accounts,
        data: instruction_data(name, args),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raydium_clmm::testkit::{TEST_EXTENSION_POOL, TEST_LIMIT_ORDER_POOL, mock_pool_snapshot};

    #[test]
    fn test_instruction_data() {
        let swap_v2 = SwapV2Args {
            amount: 99_500_000,
            other_amount_threshold: 2_800_000_000,
            sqrt_price_limit_x64: 0,
            is_base_input: true,
        };
        let router = SwapRouterBaseInArgs {
            amount_in: 99_500_000,
            amount_out_minimum: 2_800_000_000,
        };

        assert_eq!(instruction_data(SWAP_V2, &swap_v2), hex::decode("2b04ed0b1ac91e62e03fee0500000000009ce4a6000000000000000000000000000000000000000001").unwrap());
        assert_eq!(instruction_data(SWAP_ROUTER_BASE_IN, &router), hex::decode("457d73daf5baf2c4e03fee0500000000009ce4a600000000").unwrap());
    }

    #[test]
    fn test_swap_v2_instruction() {
        let snapshot = mock_pool_snapshot(TEST_EXTENSION_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let input = SwapToken {
            mint: pool.token_mint_1,
            ..SwapToken::mock()
        };
        let output = SwapToken {
            mint: pool.token_mint_0,
            ..SwapToken::mock()
        };
        let swap = PoolSwap {
            authority: Pubkey::new([9; 32]),
            input: &input,
            intermediate: None,
            output: &output,
            amount: 100,
            minimum_out: 90,
        };
        let instruction = swap_v2_instruction(&address, &pool, &swap).unwrap();
        let tick_arrays = TickArray::addresses(&address, pool.swap_tick_arrays(false).unwrap().starts()).unwrap();

        assert_eq!(instruction.program_id, raydium_clmm_program());
        assert_eq!(
            instruction.accounts,
            [
                vec![
                    AccountMeta::new_signer(swap.authority),
                    AccountMeta::new_readonly(pool.amm_config),
                    AccountMeta::new_writable(address),
                    AccountMeta::new_writable(input.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(output.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(pool.token_vault_1),
                    AccountMeta::new_writable(pool.token_vault_0),
                    AccountMeta::new_writable(pool.observation_key),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(token_2022_program()),
                    AccountMeta::new_readonly(memo_program()),
                    AccountMeta::new_readonly(input.mint),
                    AccountMeta::new_readonly(output.mint),
                    AccountMeta::new_writable(PoolState::bitmap_extension_address(&address).unwrap()),
                ],
                tick_arrays.into_iter().map(AccountMeta::new_writable).collect(),
            ]
            .concat()
        );
        assert_eq!(swap_v2_instruction(&address, &pool, &PoolSwap { output: &input, ..swap }), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_swap_router_base_in_instruction() {
        let first = mock_pool_snapshot(TEST_LIMIT_ORDER_POOL);
        let second = mock_pool_snapshot(TEST_EXTENSION_POOL);
        let intermediate_mint = second.pool.token_mint_1;
        let other_mint = if first.pool.token_mint_0 == intermediate_mint { first.pool.token_mint_1 } else { first.pool.token_mint_0 };
        let input = SwapToken { mint: other_mint, ..SwapToken::mock() };
        let intermediate = SwapToken {
            mint: intermediate_mint,
            ..SwapToken::mock()
        };
        let output = SwapToken {
            mint: second.pool.token_mint_0,
            ..SwapToken::mock()
        };
        let swap = PoolSwap {
            authority: Pubkey::new([9; 32]),
            input: &input,
            intermediate: Some(&intermediate),
            output: &output,
            amount: 100,
            minimum_out: 90,
        };
        let first_pool = (first.address, first.pool);
        let second_pool = (second.address, second.pool);
        let instruction = swap_router_base_in_instruction(&first_pool, &second_pool, &swap, &intermediate).unwrap();
        let hop = |(address, pool): &(Pubkey, PoolState), input: &SwapToken, output: &SwapToken| router_hop_accounts(address, pool, swap.authority, input, output).unwrap();

        assert_eq!(
            instruction.accounts,
            [
                vec![
                    AccountMeta::new_signer(swap.authority),
                    AccountMeta::new_writable(input.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(input.mint),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(token_2022_program()),
                    AccountMeta::new_readonly(memo_program()),
                ],
                hop(&first_pool, &input, &intermediate),
                hop(&second_pool, &intermediate, &output),
            ]
            .concat()
        );
        assert_eq!(hop(&second_pool, &intermediate, &output)[..7].to_vec(), {
            let (input_vault, output_vault) = second_pool.1.vaults(false);
            vec![
                AccountMeta::new_readonly(second_pool.1.amm_config),
                AccountMeta::new_writable(second_pool.0),
                AccountMeta::new_writable(output.account(&swap.authority).unwrap()),
                AccountMeta::new_writable(input_vault),
                AccountMeta::new_writable(output_vault),
                AccountMeta::new_readonly(output.mint),
                AccountMeta::new_writable(second_pool.1.observation_key),
            ]
        });
    }
}
