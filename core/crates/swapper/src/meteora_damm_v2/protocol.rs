use gem_solana::{Instruction, Pubkey, anchor::AnchorAccount, instructions::program_ids::meteora_damm_v2_program, models::Clock};

use super::{
    pool::{LAYOUT, Pool},
    swap::swap_instruction,
};
use crate::{
    SwapperError, SwapperProvider,
    solana_dex::{DexPool, DexProtocol, PoolQuery, PoolSwap, QuoteAccount},
};

pub(crate) struct MeteoraDammV2;

impl DexProtocol for MeteoraDammV2 {
    type Pool = Pool;

    const PROVIDER: SwapperProvider = SwapperProvider::MeteoraDammV2;
    const PROGRAM_ID: Pubkey = meteora_damm_v2_program();
    const PROGRAM: &'static str = gem_solana::METEORA_DAMM_V2_PROGRAM_ID;

    fn decode_pool(&self, data: &[u8]) -> Result<Pool, SwapperError> {
        Ok(Pool::decode(data)?)
    }

    fn is_tradable(&self, pool: &Pool, _clock: &Clock) -> bool {
        pool.is_tradable()
    }

    fn pool_query(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Option<PoolQuery> {
        Some(LAYOUT.query(input_mint, output_mint))
    }

    fn quote_accounts(&self, _address: &Pubkey, _pool: &Pool, _input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        Ok(vec![])
    }

    fn quote(&self, _address: &Pubkey, pool: &Pool, input_mint: &Pubkey, amount_in: u64, _accounts: &[Option<Vec<u8>>], clock: &Clock) -> Result<u64, SwapperError> {
        pool.quote(pool.is_a_to_b(input_mint)?, amount_in, clock).ok_or(SwapperError::NoQuoteAvailable)
    }

    fn swap_instruction(&self, address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap_instruction(address, pool, swap)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::solana_dex::testkit::{TEST_FUNDED_WALLET, TEST_TOS_HOLDER, TEST_TOS_MINT, assert_swaps_simulate, mock_solana_asset};

    #[tokio::test]
    async fn test_meteora_damm_v2_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let tos = mock_solana_asset(Some(TEST_TOS_MINT), 6);

        assert_swaps_simulate(MeteoraDammV2, vec![(tos.clone(), sol.clone(), 326_754_966, TEST_TOS_HOLDER), (sol, tos, 10_000_000, TEST_FUNDED_WALLET)]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QuoteDataInput, QuoteRequest, Swapper,
        meteora_damm_v2::testkit::{TEST_BOTH_TOKEN_POOL, TEST_COMPOUNDING_POOL, TEST_MARKET_CAP_SCHEDULER_POOL, TEST_TIME_SCHEDULER_POOL, mock_pool_snapshot},
        solana_dex::{
            SolanaDex,
            testkit::{TEST_TOS_MINT, mock_rpc_accounts, mock_rpc_client, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn test_quote() {
        let quote = |snapshot: &str, a_to_b: bool, amount_in: u64| {
            let snapshot = mock_pool_snapshot(snapshot);
            let input_mint = if a_to_b { snapshot.pool.token_a_mint } else { snapshot.pool.token_b_mint };
            let data = snapshot.quote_data(&MeteoraDammV2.quote_accounts(&snapshot.address, &snapshot.pool, &input_mint).unwrap());
            MeteoraDammV2.quote(&snapshot.address, &snapshot.pool, &input_mint, amount_in, &data, &snapshot.clock)
        };

        assert_eq!(quote(TEST_BOTH_TOKEN_POOL, true, 326_754_966), Ok(9_936_136));
        assert_eq!(quote(TEST_BOTH_TOKEN_POOL, false, 10_000_000), Ok(322_182_789));
        assert_eq!(quote(TEST_BOTH_TOKEN_POOL, false, 1_000_000_000), Ok(31_597_813_375));
        assert_eq!(quote(TEST_MARKET_CAP_SCHEDULER_POOL, true, 36_855_149_004_397), Ok(1_151_687_122));
        assert_eq!(quote(TEST_MARKET_CAP_SCHEDULER_POOL, false, 1_156_370_961), Ok(36_707_319_414_102));
        assert_eq!(quote(TEST_TIME_SCHEDULER_POOL, true, 78_056), Ok(1_158));
        assert_eq!(quote(TEST_COMPOUNDING_POOL, true, 10_944_450_962), Ok(569_090_679));
    }

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let accounts = [TEST_BOTH_TOKEN_POOL, TEST_COMPOUNDING_POOL].into_iter().flat_map(mock_rpc_accounts).collect();
        let damm = SolanaDex::with_client(MeteoraDammV2, mock_rpc_client(accounts, requests.clone()));
        let sol = mock_solana_asset(None, 9);
        let tos = mock_solana_asset(Some(TEST_TOS_MINT), 6);
        let tos_to_sol = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(326_754_966u64),
            ..mock_quote(tos.clone(), sol.clone())
        };

        let quote = damm.get_quote(&tos_to_sol).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 3);
        assert_eq!(quote.to_value, BigUint::from(9_886_456u64));

        assert_eq!(damm.get_quote(&tos_to_sol).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 4);

        let sol_to_tos = QuoteRequest {
            from_asset: sol.clone(),
            to_asset: tos,
            value: BigUint::from(10_000_000u64),
            ..tos_to_sol.clone()
        };
        assert_eq!(damm.get_quote(&sol_to_tos).await.unwrap().to_value, BigUint::from(320_572_193u64));
        assert_eq!(requests.load(Ordering::SeqCst), 5);

        let sol_to_usdc = QuoteRequest {
            to_asset: mock_solana_asset(Some(gem_solana::USDC_TOKEN_MINT), 6),
            ..sol_to_tos
        };
        assert_eq!(damm.get_quote(&sol_to_usdc).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(damm.get_quote(&sol_to_usdc).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 7);

        let quote_data = damm.get_quote_data(&quote, QuoteDataInput::None).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 8);
        assert_eq!(quote_data.to, meteora_damm_v2_program().to_base58());
    }
}
