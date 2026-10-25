use gem_solana::{Instruction, Pubkey, instructions::program_ids::raydium_amm_program, models::Clock};

use super::{
    pool::{LAYOUT, Pool},
    swap::swap_base_in_v2_instruction,
};
use crate::{
    SwapperError, SwapperProvider,
    solana_dex::{DexPool, DexProtocol, PoolQuery, PoolSwap, QuoteAccount, token_amount},
};

pub(crate) struct RaydiumAmm;

impl DexProtocol for RaydiumAmm {
    type Pool = Pool;

    const PROVIDER: SwapperProvider = SwapperProvider::RaydiumClmm;
    const PROGRAM_ID: Pubkey = raydium_amm_program();
    const PROGRAM: &'static str = gem_solana::RAYDIUM_AMM_PROGRAM_ID;

    fn decode_pool(&self, data: &[u8]) -> Result<Pool, SwapperError> {
        Pool::decode(data)
    }

    fn is_tradable(&self, pool: &Pool, clock: &Clock) -> bool {
        pool.is_tradable(clock.timestamp())
    }

    fn pool_query(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Option<PoolQuery> {
        Some(LAYOUT.query(input_mint, output_mint))
    }

    fn quote_accounts(&self, _address: &Pubkey, pool: &Pool, _input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        Ok(vec![QuoteAccount::Token(pool.coin_vault), QuoteAccount::Token(pool.pc_vault)])
    }

    fn quote(&self, _address: &Pubkey, pool: &Pool, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], _clock: &Clock) -> Result<u64, SwapperError> {
        let coin_to_pc = pool.is_a_to_b(input_mint)?;
        let [coin_vault, pc_vault] = accounts else {
            return Err(SwapperError::NoQuoteAvailable);
        };
        pool.quote(coin_to_pc, amount_in, token_amount(coin_vault)?, token_amount(pc_vault)?).ok_or(SwapperError::NoQuoteAvailable)
    }

    fn swap_instruction(&self, address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap_base_in_v2_instruction(address, pool, swap)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::solana_dex::testkit::{TEST_FUNDED_WALLET, assert_swaps_simulate, mock_solana_asset};
    use gem_solana::USDC_TOKEN_MINT;

    #[tokio::test]
    async fn test_raydium_amm_swap_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);

        assert_swaps_simulate(RaydiumAmm, vec![(sol.clone(), usdc.clone(), 1_000_000_000, TEST_FUNDED_WALLET), (usdc, sol, 100_000_000, TEST_FUNDED_WALLET)]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FetchQuoteData, QuoteRequest, Swapper,
        raydium_amm::testkit::TEST_SOL_USDC_POOL,
        solana_dex::{
            SolanaDex,
            testkit::{mock_rpc_accounts, mock_rpc_client, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use gem_solana::USDC_TOKEN_MINT;
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let amm = SolanaDex::with_client(RaydiumAmm, mock_rpc_client(mock_rpc_accounts(TEST_SOL_USDC_POOL), requests.clone()));
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let sol_to_usdc = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(1_000_000_000u64),
            ..mock_quote(sol.clone(), usdc.clone())
        };

        let quote = amm.get_quote(&sol_to_usdc).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 4);
        assert_eq!(quote.to_value, BigUint::from(102_627_053u64));

        assert_eq!(amm.get_quote(&sol_to_usdc).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 5);

        let usdc_to_sol = QuoteRequest {
            from_asset: usdc,
            to_asset: sol,
            value: BigUint::from(100_000_000u64),
            ..sol_to_usdc
        };
        assert_eq!(amm.get_quote(&usdc_to_sol).await.unwrap().to_value, BigUint::from(959_847_099u64));
        assert_eq!(requests.load(Ordering::SeqCst), 6);

        let quote_data = amm.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        crate::contract_check::verify_solana_programs(&quote_data.data, &amm.contracts(&quote.request.from_asset.asset_id()).targets).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 7);
        assert_eq!(quote_data.to, raydium_amm_program().to_base58());
    }
}
