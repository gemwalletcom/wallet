use borsh::BorshSerialize;
use gem_solana::{
    AccountMeta, Instruction, Pubkey,
    anchor::instruction_data,
    instructions::program_ids::{memo_program, orca_whirlpool_program},
};
use std::iter::repeat;

use super::{tick_array::TickArray, whirlpool::Whirlpool};
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap, SwapToken},
};

const SWAP_V2: &str = "swap_v2";
const TWO_HOP_SWAP_V2: &str = "two_hop_swap_v2";
const SUPPLEMENTAL_TICK_ARRAYS: u8 = 6;
const SWAP_TICK_ARRAYS: usize = 3;

#[derive(BorshSerialize)]
struct RemainingAccountsSlice {
    accounts_type: u8,
    length: u8,
}

#[derive(BorshSerialize)]
struct SwapV2Args {
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    remaining_accounts_info: Option<Vec<RemainingAccountsSlice>>,
}

#[derive(BorshSerialize)]
struct TwoHopSwapV2Args {
    amount: u64,
    other_amount_threshold: u64,
    amount_specified_is_input: bool,
    a_to_b_one: bool,
    a_to_b_two: bool,
    sqrt_price_limit_one: u128,
    sqrt_price_limit_two: u128,
    remaining_accounts_info: Option<Vec<RemainingAccountsSlice>>,
}

pub(super) fn swap_v2_instruction(address: &Pubkey, whirlpool: &Whirlpool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let a_to_b = whirlpool.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let (token_a, token_b) = if a_to_b { (swap.input, swap.output) } else { (swap.output, swap.input) };
    let supplemental = TickArray::addresses(address, &whirlpool.supplemental_start_tick_indexes(a_to_b))?;
    let args = SwapV2Args {
        amount: swap.amount,
        other_amount_threshold: swap.minimum_out,
        sqrt_price_limit: 0,
        amount_specified_is_input: true,
        a_to_b,
        remaining_accounts_info: (!supplemental.is_empty()).then(|| {
            vec![RemainingAccountsSlice {
                accounts_type: SUPPLEMENTAL_TICK_ARRAYS,
                length: supplemental.len() as u8,
            }]
        }),
    };
    let accounts = [
        AccountMeta::new_readonly(token_a.token_program),
        AccountMeta::new_readonly(token_b.token_program),
        AccountMeta::new_readonly(memo_program()),
        AccountMeta::new_signer(swap.authority),
        AccountMeta::new_writable(*address),
        AccountMeta::new_readonly(whirlpool.token_mint_a),
        AccountMeta::new_readonly(whirlpool.token_mint_b),
        AccountMeta::new_writable(token_a.account(&swap.authority)?),
        AccountMeta::new_writable(whirlpool.token_vault_a),
        AccountMeta::new_writable(token_b.account(&swap.authority)?),
        AccountMeta::new_writable(whirlpool.token_vault_b),
    ]
    .into_iter()
    .chain(swap_tick_arrays(address, whirlpool, a_to_b)?)
    .chain([AccountMeta::new_writable(Whirlpool::oracle_address(address)?)])
    .chain(supplemental.into_iter().map(AccountMeta::new_writable))
    .collect();
    whirlpool_instruction(SWAP_V2, &args, accounts)
}

pub(super) fn two_hop_swap_v2_instruction(first: &(Pubkey, Whirlpool), second: &(Pubkey, Whirlpool), swap: &PoolSwap, intermediate: &SwapToken) -> Result<Instruction, SwapperError> {
    let ((address_one, whirlpool_one), (address_two, whirlpool_two)) = (first, second);
    let a_to_b_one = whirlpool_one.swap_direction(&swap.input.mint, &intermediate.mint)?;
    let a_to_b_two = whirlpool_two.swap_direction(&intermediate.mint, &swap.output.mint)?;
    let (vault_one_input, vault_one_intermediate) = whirlpool_one.vaults(a_to_b_one);
    let (vault_two_intermediate, vault_two_output) = whirlpool_two.vaults(a_to_b_two);
    let args = TwoHopSwapV2Args {
        amount: swap.amount,
        other_amount_threshold: swap.minimum_out,
        amount_specified_is_input: true,
        a_to_b_one,
        a_to_b_two,
        sqrt_price_limit_one: 0,
        sqrt_price_limit_two: 0,
        remaining_accounts_info: None,
    };
    let accounts = [
        AccountMeta::new_writable(*address_one),
        AccountMeta::new_writable(*address_two),
        AccountMeta::new_readonly(swap.input.mint),
        AccountMeta::new_readonly(intermediate.mint),
        AccountMeta::new_readonly(swap.output.mint),
        AccountMeta::new_readonly(swap.input.token_program),
        AccountMeta::new_readonly(intermediate.token_program),
        AccountMeta::new_readonly(swap.output.token_program),
        AccountMeta::new_writable(swap.input.account(&swap.authority)?),
        AccountMeta::new_writable(vault_one_input),
        AccountMeta::new_writable(vault_one_intermediate),
        AccountMeta::new_writable(vault_two_intermediate),
        AccountMeta::new_writable(vault_two_output),
        AccountMeta::new_writable(swap.output.account(&swap.authority)?),
        AccountMeta::new_signer(swap.authority),
    ]
    .into_iter()
    .chain(swap_tick_arrays(address_one, whirlpool_one, a_to_b_one)?)
    .chain(swap_tick_arrays(address_two, whirlpool_two, a_to_b_two)?)
    .chain([
        AccountMeta::new_writable(Whirlpool::oracle_address(address_one)?),
        AccountMeta::new_writable(Whirlpool::oracle_address(address_two)?),
        AccountMeta::new_readonly(memo_program()),
    ])
    .collect();
    whirlpool_instruction(TWO_HOP_SWAP_V2, &args, accounts)
}

fn swap_tick_arrays(address: &Pubkey, whirlpool: &Whirlpool, a_to_b: bool) -> Result<Vec<AccountMeta>, SwapperError> {
    let tick_arrays = TickArray::addresses(address, &whirlpool.swap_start_tick_indexes(a_to_b))?;
    let last_tick_array = *tick_arrays.last().ok_or(SwapperError::NoQuoteAvailable)?;
    Ok(tick_arrays.iter().copied().chain(repeat(last_tick_array)).take(SWAP_TICK_ARRAYS).map(AccountMeta::new_writable).collect())
}

fn whirlpool_instruction(name: &str, args: &impl BorshSerialize, accounts: Vec<AccountMeta>) -> Result<Instruction, SwapperError> {
    Ok(Instruction {
        program_id: orca_whirlpool_program(),
        accounts,
        data: instruction_data(name, args),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orca::testkit::{TEST_WHIRLPOOL, mock_aqua_whirlpool};
    use gem_solana::instructions::program_ids::token_program;

    #[test]
    fn test_swap_v2_data() {
        let args = SwapV2Args {
            amount: 1_194_000,
            other_amount_threshold: 9_883_563,
            sqrt_price_limit: 0,
            amount_specified_is_input: true,
            a_to_b: false,
            remaining_accounts_info: Some(vec![RemainingAccountsSlice {
                accounts_type: SUPPLEMENTAL_TICK_ARRAYS,
                length: 2,
            }]),
        };

        assert_eq!(
            instruction_data(SWAP_V2, &args),
            hex::decode("2b04ed0b1ac91e621038120000000000abcf96000000000000000000000000000000000000000000010001010000000602").unwrap()
        );
    }

    #[test]
    fn test_swap_v2_instruction() {
        let whirlpool = Whirlpool::mock();
        let address = Pubkey::from_base58(TEST_WHIRLPOOL).unwrap();
        let input = SwapToken {
            mint: whirlpool.token_mint_b,
            ..SwapToken::mock()
        };
        let output = SwapToken {
            mint: whirlpool.token_mint_a,
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
        let instruction = swap_v2_instruction(&address, &whirlpool, &swap).unwrap();
        let tick_arrays = TickArray::addresses(&address, &[-22_528, -16_896, -11_264, -28_160, -5_632]).unwrap();

        assert_eq!(instruction.program_id, orca_whirlpool_program());
        assert_eq!(
            instruction.accounts,
            [
                vec![
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(memo_program()),
                    AccountMeta::new_signer(swap.authority),
                    AccountMeta::new_writable(address),
                    AccountMeta::new_readonly(whirlpool.token_mint_a),
                    AccountMeta::new_readonly(whirlpool.token_mint_b),
                    AccountMeta::new_writable(output.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(whirlpool.token_vault_a),
                    AccountMeta::new_writable(input.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(whirlpool.token_vault_b),
                ],
                tick_arrays[..3].iter().copied().map(AccountMeta::new_writable).collect(),
                vec![AccountMeta::new_writable(Whirlpool::oracle_address(&address).unwrap())],
                tick_arrays[3..].iter().copied().map(AccountMeta::new_writable).collect(),
            ]
            .concat()
        );
        assert_eq!(swap_v2_instruction(&address, &whirlpool, &PoolSwap { output: &input, ..swap }), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_two_hop_swap_v2_data() {
        let args = TwoHopSwapV2Args {
            amount: 1_000_000,
            other_amount_threshold: 900_000,
            amount_specified_is_input: true,
            a_to_b_one: false,
            a_to_b_two: false,
            sqrt_price_limit_one: 0,
            sqrt_price_limit_two: 0,
            remaining_accounts_info: None,
        };

        assert_eq!(
            instruction_data(TWO_HOP_SWAP_V2, &args),
            hex::decode("ba8fd11dfe02c27540420f0000000000a0bb0d0000000000010000000000000000000000000000000000000000000000000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn test_two_hop_swap_v2_instruction() {
        let first = (Pubkey::from_base58(TEST_WHIRLPOOL).unwrap(), Whirlpool::mock());
        let second = mock_aqua_whirlpool();
        let input = SwapToken {
            mint: first.1.token_mint_b,
            ..SwapToken::mock()
        };
        let intermediate = SwapToken {
            mint: first.1.token_mint_a,
            ..SwapToken::mock()
        };
        let output = SwapToken {
            mint: second.1.token_mint_a,
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
        let instruction = two_hop_swap_v2_instruction(&first, &second, &swap, &intermediate).unwrap();
        let tick_arrays = |(address, whirlpool): &(Pubkey, Whirlpool)| {
            let addresses = TickArray::addresses(address, &whirlpool.swap_start_tick_indexes(false)).unwrap();
            addresses.iter().chain(std::iter::repeat(addresses.last().unwrap())).take(3).copied().map(AccountMeta::new_writable).collect::<Vec<_>>()
        };

        assert_eq!(
            instruction.accounts,
            [
                vec![
                    AccountMeta::new_writable(first.0),
                    AccountMeta::new_writable(second.0),
                    AccountMeta::new_readonly(input.mint),
                    AccountMeta::new_readonly(intermediate.mint),
                    AccountMeta::new_readonly(output.mint),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_readonly(token_program()),
                    AccountMeta::new_writable(input.account(&swap.authority).unwrap()),
                    AccountMeta::new_writable(first.1.token_vault_b),
                    AccountMeta::new_writable(first.1.token_vault_a),
                    AccountMeta::new_writable(second.1.token_vault_b),
                    AccountMeta::new_writable(second.1.token_vault_a),
                    AccountMeta::new_writable(output.account(&swap.authority).unwrap()),
                    AccountMeta::new_signer(swap.authority),
                ],
                tick_arrays(&first),
                tick_arrays(&second),
                vec![
                    AccountMeta::new_writable(Whirlpool::oracle_address(&first.0).unwrap()),
                    AccountMeta::new_writable(Whirlpool::oracle_address(&second.0).unwrap()),
                    AccountMeta::new_readonly(memo_program()),
                ],
            ]
            .concat()
        );
        assert_eq!(two_hop_swap_v2_instruction(&first, &second, &swap, &input), Err(SwapperError::InvalidRoute));
    }
}
