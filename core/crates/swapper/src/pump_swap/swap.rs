use gem_solana::{
    AccountMeta, Instruction, Pubkey,
    anchor::instruction_data,
    find_program_address,
    instructions::program_ids::{pump_amm_program, system_program},
};
use primitives::decode_hex_literal;

use super::{fee_config::FEE_CONFIG, pool::Pool};
use crate::{
    SwapperError,
    solana_dex::{DexPool, PoolSwap, SwapAmounts},
};

const SELL_V2: &str = "sell_v2";
const BUY_EXACT_QUOTE_IN_V2: &str = "buy_exact_quote_in_v2";
const GLOBAL_CONFIG: Pubkey = Pubkey::new(decode_hex_literal("890ba644fe1f55aa19f11cd2d2ec14d3233b6e0a4beaeef72b69858e21e170d6"));
const EVENT_AUTHORITY: Pubkey = Pubkey::new(decode_hex_literal("e54a709528839f61c0b9b86079891c139216e47a71b62fb73bec72169458745e"));
const BUYBACK_FEE_RECIPIENT: Pubkey = Pubkey::new(decode_hex_literal("439e6510c03d65fad931e89d04be0bb70d51971f51c415fb344c07db419f2122"));
const MIN_BUY_OUTPUT: u64 = 1;

pub(super) fn swap_v2_instruction(address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
    let sells_base = pool.swap_direction(&swap.input.mint, &swap.output.mint)?;
    let (name, base, quote, minimum_out) = if sells_base {
        (SELL_V2, swap.input, swap.output, swap.minimum_out)
    } else {
        (BUY_EXACT_QUOTE_IN_V2, swap.output, swap.input, swap.minimum_out.max(MIN_BUY_OUTPUT))
    };
    let accounts = vec![
        AccountMeta::new_writable(*address),
        AccountMeta::new_signer_writable(swap.authority),
        AccountMeta::new_readonly(GLOBAL_CONFIG),
        AccountMeta::new_readonly(pool.base_mint),
        AccountMeta::new_readonly(pool.quote_mint),
        AccountMeta::new_writable(base.account(&swap.authority)?),
        AccountMeta::new_writable(quote.account(&swap.authority)?),
        AccountMeta::new_writable(pool.pool_base_token_account),
        AccountMeta::new_writable(pool.pool_quote_token_account),
        AccountMeta::new_readonly(base.token_program),
        AccountMeta::new_readonly(quote.token_program),
        AccountMeta::new_readonly(system_program()),
        AccountMeta::new_writable(user_volume_accumulator(&swap.authority)?),
        AccountMeta::new_readonly(FEE_CONFIG),
        AccountMeta::new_writable(quote.account(&BUYBACK_FEE_RECIPIENT)?),
        AccountMeta::new_readonly(EVENT_AUTHORITY),
        AccountMeta::new_readonly(pump_amm_program()),
    ];
    Ok(Instruction {
        program_id: pump_amm_program(),
        accounts,
        data: instruction_data(name, &SwapAmounts { minimum_out, ..swap.amounts() }),
    })
}

fn user_volume_accumulator(user: &Pubkey) -> Result<Pubkey, SwapperError> {
    Ok(find_program_address(&pump_amm_program(), &[b"user_volume_accumulator".as_slice(), user.as_bytes()])?.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pump_swap::{
            fee_config::PUMP_FEES_PROGRAM,
            testkit::{TEST_SOL_POOL, TEST_USDC_POOL, mock_pool_snapshot},
        },
        solana_dex::SwapToken,
    };
    use gem_solana::{
        USDC_TOKEN_MINT,
        instructions::program_ids::{token_2022_program, token_program},
        wrapped_sol::wrapped_sol_mint,
    };
    use primitives::hex;

    #[test]
    fn test_program_addresses() {
        let address = |program: &Pubkey, seeds: &[&[u8]]| find_program_address(program, seeds).unwrap().0;

        assert_eq!(address(&pump_amm_program(), &[b"global_config"]), GLOBAL_CONFIG);
        assert_eq!(address(&pump_amm_program(), &[b"__event_authority"]), EVENT_AUTHORITY);
        assert_eq!(address(&PUMP_FEES_PROGRAM, &[b"fee_config", pump_amm_program().as_bytes()]), FEE_CONFIG);
    }

    #[test]
    fn test_swap_v2_instruction() {
        let snapshot = mock_pool_snapshot(TEST_SOL_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let sol = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let tos = SwapToken { mint: pool.base_mint, ..SwapToken::mock() };
        let sell = PoolSwap {
            authority,
            input: &tos,
            intermediate: None,
            output: &sol,
            amount: 1_000_000_000,
            minimum_out: 29_000_000,
        };
        let buy = PoolSwap {
            input: &sol,
            output: &tos,
            amount: 10_000_000,
            minimum_out: 0,
            ..sell
        };
        let sell_instruction = swap_v2_instruction(&address, &pool, &sell).unwrap();

        assert_eq!(sell_instruction.program_id, pump_amm_program());
        assert_eq!(
            sell_instruction.accounts,
            vec![
                AccountMeta::new_writable(address),
                AccountMeta::new_signer_writable(authority),
                AccountMeta::new_readonly(GLOBAL_CONFIG),
                AccountMeta::new_readonly(tos.mint),
                AccountMeta::new_readonly(sol.mint),
                AccountMeta::new_writable(tos.account(&authority).unwrap()),
                AccountMeta::new_writable(sol.account(&authority).unwrap()),
                AccountMeta::new_writable(pool.pool_base_token_account),
                AccountMeta::new_writable(pool.pool_quote_token_account),
                AccountMeta::new_readonly(token_program()),
                AccountMeta::new_readonly(token_program()),
                AccountMeta::new_readonly(system_program()),
                AccountMeta::new_writable(user_volume_accumulator(&authority).unwrap()),
                AccountMeta::new_readonly(FEE_CONFIG),
                AccountMeta::new_writable(sol.account(&BUYBACK_FEE_RECIPIENT).unwrap()),
                AccountMeta::new_readonly(EVENT_AUTHORITY),
                AccountMeta::new_readonly(pump_amm_program()),
            ]
        );
        assert_eq!(sell_instruction.data, hex::decode("5df6823ce7e940b200ca9a3b000000004081ba0100000000").unwrap());
        assert_eq!(swap_v2_instruction(&address, &pool, &buy).unwrap().accounts, sell_instruction.accounts);
        assert_eq!(swap_v2_instruction(&address, &pool, &buy).unwrap().data, hex::decode("c2ab1c46684d5b2f80969800000000000100000000000000").unwrap());
        assert_eq!(swap_v2_instruction(&address, &pool, &PoolSwap { output: &tos, ..sell }), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_swap_v2_token_2022_instruction() {
        let snapshot = mock_pool_snapshot(TEST_USDC_POOL);
        let (address, pool) = (snapshot.address, snapshot.pool);
        let authority = Pubkey::new([9; 32]);
        let usdc = SwapToken {
            mint: Pubkey::from_base58(USDC_TOKEN_MINT).unwrap(),
            ..SwapToken::mock()
        };
        let coin = SwapToken {
            mint: pool.base_mint,
            token_program: token_2022_program(),
            ..SwapToken::mock()
        };
        let buy = PoolSwap {
            authority,
            input: &usdc,
            intermediate: None,
            output: &coin,
            amount: 1_000_000,
            minimum_out: 500_000_000,
        };
        let accounts = swap_v2_instruction(&address, &pool, &buy).unwrap().accounts;

        assert_eq!(
            [&accounts[5..7], &accounts[9..11], &accounts[14..15]].concat(),
            vec![
                AccountMeta::new_writable(coin.account(&authority).unwrap()),
                AccountMeta::new_writable(usdc.account(&authority).unwrap()),
                AccountMeta::new_readonly(token_2022_program()),
                AccountMeta::new_readonly(token_program()),
                AccountMeta::new_writable(usdc.account(&BUYBACK_FEE_RECIPIENT).unwrap()),
            ]
        );
    }
}
