use gem_encoding::borsh::BorshDeserialize;
use gem_solana::{Instruction, MAX_COMPUTE_UNIT_LIMIT, Pubkey, anchor::AnchorAccount, instructions::program_ids::meteora_dlmm_program, models::Clock};

use super::{
    bin::BinArray,
    pair::{LAYOUT, Pair},
    swap::{SWAP_BIN_ARRAYS, swap2_instruction},
};
use crate::{
    SwapperError, SwapperProvider,
    solana_dex::{DexPool, DexProtocol, PoolDepth, PoolQuery, PoolSwap, QuoteAccount},
};

const SWAP_COMPUTE_UNIT_LIMIT: u32 = 600_000;

pub(crate) struct MeteoraDlmm;

impl DexProtocol for MeteoraDlmm {
    type Pool = Pair;

    const PROVIDER: SwapperProvider = SwapperProvider::MeteoraDammV2;
    const PROGRAM_ID: Pubkey = meteora_dlmm_program();
    const PROGRAM: &'static str = gem_solana::METEORA_DLMM_PROGRAM_ID;

    fn decode_pool(&self, data: &[u8]) -> Result<Pair, SwapperError> {
        Ok(Pair::decode(data)?)
    }

    fn is_tradable(&self, pair: &Pair, clock: &Clock) -> bool {
        pair.is_tradable(clock)
    }

    fn pool_query(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Option<PoolQuery> {
        Some(LAYOUT.query(input_mint, output_mint))
    }

    fn compute_unit_limit(&self, swap_instructions: usize) -> u32 {
        u32::try_from(swap_instructions).map_or(MAX_COMPUTE_UNIT_LIMIT, |swaps| SWAP_COMPUTE_UNIT_LIMIT.saturating_mul(swaps).min(MAX_COMPUTE_UNIT_LIMIT))
    }

    fn pool_depth(&self, slice: &[u8]) -> PoolDepth {
        match <(Pubkey, Pubkey)>::try_from_slice(slice) {
            Ok((reserve_x, reserve_y)) => PoolDepth::Reserves(reserve_x, reserve_y),
            Err(_) => PoolDepth::Value(0),
        }
    }

    fn quote_accounts(&self, address: &Pubkey, pair: &Pair, input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        let swap_for_y = pair.is_a_to_b(input_mint)?;
        pair.arrays_for_swap(swap_for_y, SWAP_BIN_ARRAYS)
            .into_iter()
            .map(|index| {
                Ok(QuoteAccount::Owned {
                    address: BinArray::address(address, index)?,
                    owner: Self::PROGRAM_ID,
                })
            })
            .collect()
    }

    fn quote(&self, address: &Pubkey, pair: &Pair, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], clock: &Clock) -> Result<u64, SwapperError> {
        let swap_for_y = pair.is_a_to_b(input_mint)?;
        let expected = pair.arrays_for_swap(swap_for_y, SWAP_BIN_ARRAYS);
        let bin_arrays = accounts
            .iter()
            .zip(expected)
            .map_while(|(data, index)| BinArray::decode(data.as_deref()?).ok().filter(|array| array.index == index && array.lb_pair == *address))
            .collect::<Vec<_>>();
        pair.quote(&bin_arrays, swap_for_y, amount_in, clock).ok_or(SwapperError::NoQuoteAvailable)
    }

    fn swap_instruction(&self, address: &Pubkey, pair: &Pair, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap2_instruction(address, pair, swap)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::solana_dex::testkit::{TEST_FUNDED_WALLET, assert_swaps_simulate, mock_solana_asset};
    use gem_solana::{USDC_TOKEN_MINT, USDT_TOKEN_MINT};

    const TEST_JUP_MINT: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

    #[tokio::test]
    async fn test_meteora_dlmm_swap_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let usdt = mock_solana_asset(Some(USDT_TOKEN_MINT), 6);
        let jup = mock_solana_asset(Some(TEST_JUP_MINT), 6);

        assert_swaps_simulate(
            MeteoraDlmm,
            vec![
                (sol.clone(), usdc.clone(), 1_000_000_000, TEST_FUNDED_WALLET),
                (usdc.clone(), sol, 100_000_000, TEST_FUNDED_WALLET),
                (usdt, jup, 7_000_000_000, TEST_FUNDED_WALLET),
            ],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QuoteDataInput, QuoteRequest, Swapper,
        meteora_dlmm::testkit::{TEST_SOL_USDC_POOL, mock_pool_snapshot},
        solana_dex::{
            SolanaDex,
            testkit::{mock_rpc_accounts, mock_rpc_client, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use gem_solana::{USDC_TOKEN_MINT, decode_transaction};
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn test_quote_matches_the_sdk() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let (address, pair) = (snapshot.address, snapshot.pool.clone());
        let quote = |input_mint: Pubkey, amount_in: u64| {
            let data = snapshot.quote_data(&MeteoraDlmm.quote_accounts(&address, &pair, &input_mint).unwrap());
            MeteoraDlmm.quote(&address, &pair, &input_mint, amount_in, &data, &snapshot.clock)
        };
        let (sol, usdc) = (pair.token_x_mint, pair.token_y_mint);

        assert_eq!(quote(sol, 1_000_000_000), Ok(103_478_946));
        assert_eq!(quote(usdc, 100_000_000), Ok(966_085_679));
        assert_eq!(quote(sol, 400_000_000_000), Ok(41_317_612_262));
        assert_eq!(quote(sol, 450_000_000_000), Ok(46_434_836_994));
        assert_eq!(quote(usdc, 5_000_000_000), Ok(48_297_362_721));
        assert_eq!(quote(usdc, 15_000_000_000), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(quote(sol, 1), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_layout() {
        let snapshot = mock_pool_snapshot(TEST_SOL_USDC_POOL);
        let data = snapshot.accounts[&snapshot.address].as_ref().unwrap();
        let at = |offset: usize, len: usize| &data[offset..offset + len];
        let pair = snapshot.pool;

        assert_eq!(u64::try_from(data.len()).unwrap(), LAYOUT.account_len);
        assert_eq!(at(LAYOUT.mint_offsets[0], 32), pair.token_x_mint.as_bytes());
        assert_eq!(at(LAYOUT.mint_offsets[1], 32), pair.token_y_mint.as_bytes());
        assert_eq!(MeteoraDlmm.pool_depth(at(LAYOUT.depth.offset, LAYOUT.depth.length)), PoolDepth::Reserves(pair.reserve_x, pair.reserve_y));
        assert_eq!((pair.active_id, pair.bin_step), (-22_684, 1));
        assert_eq!(pair.arrays_for_swap(true, SWAP_BIN_ARRAYS), vec![-325, -326, -327]);
        assert_eq!(pair.arrays_for_swap(false, SWAP_BIN_ARRAYS), vec![-325, -324, -323]);
    }

    #[test]
    fn test_compute_unit_limit() {
        assert_eq!([1, 2, 3].map(|swaps| MeteoraDlmm.compute_unit_limit(swaps)), [600_000, 1_200_000, MAX_COMPUTE_UNIT_LIMIT]);
    }

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let dlmm = SolanaDex::with_client(MeteoraDlmm, mock_rpc_client(mock_rpc_accounts(TEST_SOL_USDC_POOL), requests.clone()));
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let sol_to_usdc = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(1_000_000_000u64),
            ..mock_quote(sol, usdc)
        };

        let quote = dlmm.get_quote(&sol_to_usdc).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 5);
        assert_eq!(quote.to_value, BigUint::from(102_961_551u64));

        assert_eq!(dlmm.get_quote(&sol_to_usdc).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 6);

        let quote_data = dlmm.get_quote_data(&quote, QuoteDataInput::None).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 7);
        assert_eq!(quote_data.to, meteora_dlmm_program().to_base58());
        assert_eq!(decode_transaction(&quote_data.data).unwrap().get_compute_unit_limit().map(|limit| limit.to_string()), quote_data.gas_limit);
    }
}
