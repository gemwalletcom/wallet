use gem_solana::{Instruction, Pubkey, anchor::AnchorAccount, instructions::program_ids::raydium_cpmm_program, models::Clock};

use super::{
    pool::{AmmConfig, LAYOUT, Pool},
    swap::swap_base_input_instruction,
};
use crate::{
    SwapperError, SwapperProvider,
    solana_dex::{DexPool, DexProtocol, PoolQuery, PoolSwap, QuoteAccount, account_data, token_amount},
};

pub(crate) struct RaydiumCpmm;

impl DexProtocol for RaydiumCpmm {
    type Pool = Pool;

    const PROVIDER: SwapperProvider = SwapperProvider::RaydiumClmm;
    const PROGRAM_ID: Pubkey = raydium_cpmm_program();
    const PROGRAM: &'static str = gem_solana::RAYDIUM_CPMM_PROGRAM_ID;

    fn decode_pool(&self, data: &[u8]) -> Result<Pool, SwapperError> {
        Ok(Pool::decode(data)?)
    }

    fn constant_accounts(&self, pool: &Pool) -> Vec<Pubkey> {
        vec![pool.amm_config]
    }

    fn is_tradable(&self, pool: &Pool, clock: &Clock) -> bool {
        pool.is_tradable(clock.timestamp())
    }

    fn pool_query(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Option<PoolQuery> {
        Some(LAYOUT.sorted_query(input_mint, output_mint))
    }

    fn quote_accounts(&self, _address: &Pubkey, pool: &Pool, _input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        Ok(vec![
            QuoteAccount::Owned {
                address: pool.amm_config,
                owner: Self::PROGRAM_ID,
            },
            QuoteAccount::Token(pool.token_0_vault),
            QuoteAccount::Token(pool.token_1_vault),
        ])
    }

    fn quote(&self, _address: &Pubkey, pool: &Pool, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], _clock: &Clock) -> Result<u64, SwapperError> {
        let zero_for_one = pool.is_a_to_b(input_mint)?;
        let [config, vault_0, vault_1] = accounts else {
            return Err(SwapperError::NoQuoteAvailable);
        };
        let config = AmmConfig::decode(account_data(config)?)?;
        pool.quote(&config, zero_for_one, amount_in, token_amount(vault_0)?, token_amount(vault_1)?).ok_or(SwapperError::NoQuoteAvailable)
    }

    fn swap_instruction(&self, address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap_base_input_instruction(address, pool, swap)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::solana_dex::testkit::{TEST_FUNDED_WALLET, assert_swaps_simulate, mock_solana_asset};

    const TEST_JUP_MINT: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

    #[tokio::test]
    async fn test_raydium_cpmm_swap_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let jup = mock_solana_asset(Some(TEST_JUP_MINT), 6);

        assert_swaps_simulate(RaydiumCpmm, vec![(sol, jup, 10_000_000, TEST_FUNDED_WALLET)]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QuoteDataInput, QuoteRequest, Swapper,
        raydium_cpmm::testkit::TEST_JUP_SOL_POOL,
        solana_dex::{
            SolanaDex,
            testkit::{mock_rpc_accounts, mock_rpc_client, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let cpmm = SolanaDex::with_client(RaydiumCpmm, mock_rpc_client(mock_rpc_accounts(TEST_JUP_SOL_POOL), requests.clone()));
        let sol = mock_solana_asset(None, 9);
        let jup = mock_solana_asset(Some("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"), 6);
        let sol_to_jup = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(1_000_000_000u64),
            ..mock_quote(sol.clone(), jup.clone())
        };

        let quote = cpmm.get_quote(&sol_to_jup).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 3);
        assert_eq!(quote.to_value, BigUint::from(340_535_080u64));

        assert_eq!(cpmm.get_quote(&sol_to_jup).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 4);

        let jup_to_sol = QuoteRequest {
            from_asset: jup,
            to_asset: sol,
            value: BigUint::from(100_000_000u64),
            ..sol_to_jup
        };
        assert_eq!(cpmm.get_quote(&jup_to_sol).await.unwrap().to_value, BigUint::from(274_233_898u64));
        assert_eq!(requests.load(Ordering::SeqCst), 5);

        let quote_data = cpmm.get_quote_data(&quote, QuoteDataInput::None).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 6);
        assert_eq!(quote_data.to, raydium_cpmm_program().to_base58());
    }
}
