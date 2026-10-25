use gem_solana::{
    Instruction, Pubkey,
    anchor::AnchorAccount,
    instructions::program_ids::pump_amm_program,
    models::{Clock, Mint},
};

use super::{
    fee_config::{FEE_CONFIG, FeeConfig, PUMP_FEES_PROGRAM},
    pool::Pool,
    swap::swap_v2_instruction,
};
use crate::{
    SwapperError, SwapperProvider,
    solana_dex::{DexPool, DexProtocol, PoolSwap, QuoteAccount, account_data, token_amount},
};

pub(crate) struct PumpSwap;

impl DexProtocol for PumpSwap {
    type Pool = Pool;

    const PROVIDER: SwapperProvider = SwapperProvider::PumpSwap;
    const PROGRAM_ID: Pubkey = pump_amm_program();
    const PROGRAM: &'static str = gem_solana::PUMP_AMM_PROGRAM_ID;

    fn pool_addresses(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        Pool::addresses(input_mint, output_mint)
    }

    fn decode_pool(&self, data: &[u8]) -> Result<Pool, SwapperError> {
        Pool::decode_padded(data)
    }

    fn constant_accounts(&self, _pool: &Pool) -> Vec<Pubkey> {
        vec![FEE_CONFIG]
    }

    fn is_tradable(&self, pool: &Pool, _clock: &Clock) -> bool {
        pool.lp_supply > 0 && !pool.is_cashback_coin
    }

    fn quote_accounts(&self, _address: &Pubkey, pool: &Pool, _input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        Ok(vec![
            QuoteAccount::Owned {
                address: FEE_CONFIG,
                owner: PUMP_FEES_PROGRAM,
            },
            QuoteAccount::Token(pool.pool_base_token_account),
            QuoteAccount::Token(pool.pool_quote_token_account),
            QuoteAccount::Token(pool.base_mint),
        ])
    }

    fn quote(&self, _address: &Pubkey, pool: &Pool, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], _clock: &Clock) -> Result<u64, SwapperError> {
        let sells_base = pool.is_a_to_b(input_mint)?;
        let [fee_config, base_vault, quote_vault, base_mint] = accounts else {
            return Err(SwapperError::NoQuoteAvailable);
        };
        let fee_config = FeeConfig::decode(account_data(fee_config)?)?;
        let reserves = pool.reserves(token_amount(base_vault)?, token_amount(quote_vault)?).ok_or(SwapperError::NoQuoteAvailable)?;
        let fees = pool.fees(&fee_config, Mint::decode(account_data(base_mint)?)?.supply, &reserves).ok_or(SwapperError::NoQuoteAvailable)?;
        let output_value = if sells_base { reserves.sell(amount_in, &fees) } else { reserves.buy(amount_in, &fees) };
        output_value.ok_or(SwapperError::NoQuoteAvailable)
    }

    fn swap_instruction(&self, address: &Pubkey, pool: &Pool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap_v2_instruction(address, pool, swap)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        SwapperQuoteAsset,
        solana_dex::testkit::{TEST_FUNDED_WALLET, TEST_TOS_HOLDER, TEST_TOS_MINT, assert_swaps_simulate, mock_solana_asset},
    };
    use primitives::AssetType;

    const TEST_CVNT_MINT: &str = "2mNVZ6aEjrGwiUVCfz7XGWpiXuWzgBDoznwE579upump";

    #[tokio::test]
    async fn test_pump_swap_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let tos = mock_solana_asset(Some(TEST_TOS_MINT), 6);
        let cvnt = SwapperQuoteAsset {
            asset_type: AssetType::SPL2022,
            ..mock_solana_asset(Some(TEST_CVNT_MINT), 6)
        };

        assert_swaps_simulate(
            PumpSwap,
            vec![
                (sol.clone(), tos.clone(), 10_000_000, TEST_FUNDED_WALLET),
                (tos.clone(), sol, 10_000_000, TEST_TOS_HOLDER),
                (tos, cvnt, 10_000_000, TEST_TOS_HOLDER),
            ],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FetchQuoteData, QuoteRequest, Swapper, SwapperQuoteAsset,
        fees::subtract_bps,
        pump_swap::testkit::{TEST_CASHBACK_POOL, TEST_MAYHEM_POOL, TEST_SOL_POOL, TEST_TOKEN_2022_MINT, TEST_USDC_POOL, mock_pool_snapshot},
        solana_dex::{
            SolanaDex, SwapToken,
            testkit::{PoolSnapshot, TEST_TOS_MINT, mock_clock, mock_rpc_accounts, mock_rpc_client, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use gem_solana::{USDC_TOKEN_MINT, decode_transaction, instructions::program_ids::token_program, wrapped_sol::wrapped_sol_mint};
    use num_bigint::BigUint;
    use primitives::{AssetType, swap::SOLANA_REFERRAL_ADDRESS, testkit::signer_mock::TEST_SOLANA_SENDER};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn test_quote() {
        let quote = |snapshot: &str, sells_base: bool, amount_in: u64| {
            let snapshot = mock_pool_snapshot(snapshot);
            let input_mint = if sells_base { snapshot.pool.base_mint } else { snapshot.pool.quote_mint };
            let data = snapshot.quote_data(&PumpSwap.quote_accounts(&snapshot.address, &snapshot.pool, &input_mint).unwrap());
            PumpSwap.quote(&snapshot.address, &snapshot.pool, &input_mint, amount_in, &data, &snapshot.clock)
        };

        assert_eq!(quote(TEST_SOL_POOL, false, 10_000_000), Ok(328_453_047));
        assert_eq!(quote(TEST_SOL_POOL, false, 1_000_000_000), Ok(32_810_685_455));
        assert_eq!(quote(TEST_SOL_POOL, true, 1_000_000_000), Ok(29_991_164));
        assert_eq!(quote(TEST_SOL_POOL, true, 30_000_000_000_000), Ok(455_884_699_615));
        assert_eq!(quote(TEST_SOL_POOL, false, 1), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(quote(TEST_USDC_POOL, false, 1_000_000), Ok(517_432_058));
        assert_eq!(quote(TEST_USDC_POOL, true, 1_000_000_000), Ok(1_894_298));
        assert_eq!(quote(TEST_MAYHEM_POOL, false, 10_000_000), Ok(19_617_747_422));
        assert_eq!(quote(TEST_MAYHEM_POOL, true, 1_000_000_000), Ok(497_605));
        assert_eq!(quote(TEST_CASHBACK_POOL, true, 1_000_000_000), Ok(16_282_726));
        assert_eq!(quote(TEST_CASHBACK_POOL, true, 10_000_000_000_000_000), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_is_tradable() {
        assert!(PumpSwap.is_tradable(&mock_pool_snapshot(TEST_SOL_POOL).pool, &mock_clock()));
        assert!(!PumpSwap.is_tradable(&mock_pool_snapshot(TEST_CASHBACK_POOL).pool, &mock_clock()));
    }

    #[tokio::test]
    async fn test_two_hop_through_sol_spends_the_first_hop_minimum() {
        let tos_pool = mock_pool_snapshot(TEST_SOL_POOL);
        let coin_pool = mock_pool_snapshot(TEST_MAYHEM_POOL);
        let tos = mock_solana_asset(Some(TEST_TOS_MINT), 6);
        let coin = SwapperQuoteAsset {
            asset_type: AssetType::SPL2022,
            ..mock_solana_asset(Some(&coin_pool.pool.base_mint.to_base58()), 6)
        };
        let referral = Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap();
        let referral_account = (
            SwapToken::from_asset(&tos).unwrap().account(&referral).unwrap().to_base58(),
            serde_json::json!({ "data": ["", "base64"], "owner": token_program().to_base58() }),
        );
        let accounts = mock_rpc_accounts(TEST_SOL_POOL).into_iter().chain(mock_rpc_accounts(TEST_MAYHEM_POOL)).chain([referral_account]).collect();
        let pump_swap = SolanaDex::with_client(PumpSwap, mock_rpc_client(accounts, Arc::default()));
        let request = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(1_000_000_000u64),
            ..mock_quote(tos, coin)
        };
        let quote = |snapshot: &PoolSnapshot<Pool>, input_mint: &Pubkey, amount: u64| {
            let data = snapshot.quote_data(&PumpSwap.quote_accounts(&snapshot.address, &snapshot.pool, input_mint).unwrap());
            PumpSwap.quote(&snapshot.address, &snapshot.pool, input_mint, amount, &data, &snapshot.clock).unwrap()
        };

        let result = pump_swap.get_quote(&request).await.unwrap();
        let route: serde_json::Value = serde_json::from_str(&result.data.routes[0].route_data).unwrap();
        let swap_value = route["swap_value"].as_u64().unwrap();
        let intermediate = u64::try_from(subtract_bps(&u128::from(quote(&tos_pool, &tos_pool.pool.base_mint, swap_value)), request.options.slippage.bps)).unwrap();

        assert_eq!([&route["hops"][0]["pool"], &route["hops"][1]["pool"]], [&tos_pool.address.to_base58(), &coin_pool.address.to_base58()]);
        assert_eq!(route["intermediate_value"], intermediate);
        assert_eq!(route["output_value"], quote(&coin_pool, &wrapped_sol_mint(), intermediate));

        let transaction = decode_transaction(&pump_swap.get_quote_data(&result, FetchQuoteData::None).await.unwrap().data).unwrap();
        let programs = transaction.instructions().iter().map(|instruction| transaction.account_keys()[usize::from(instruction.program_id_index)]).collect::<Vec<_>>();
        assert_eq!(programs.iter().filter(|program| **program == pump_amm_program()).count(), 2);
    }

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let accounts = mock_rpc_accounts(TEST_SOL_POOL).into_iter().chain(mock_rpc_accounts(TEST_USDC_POOL)).collect();
        let pump_swap = SolanaDex::with_client(PumpSwap, mock_rpc_client(accounts, requests.clone()));
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let tos = mock_solana_asset(Some(TEST_TOS_MINT), 6);
        let coin = SwapperQuoteAsset {
            asset_type: AssetType::SPL2022,
            ..mock_solana_asset(Some(TEST_TOKEN_2022_MINT), 6)
        };
        let sol_to_tos = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(10_000_000u64),
            ..mock_quote(sol.clone(), tos.clone())
        };

        let quote = pump_swap.get_quote(&sol_to_tos).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(quote.to_value, BigUint::from(326_810_825u64));

        assert_eq!(pump_swap.get_quote(&sol_to_tos).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 3);

        let tos_to_sol = QuoteRequest {
            from_asset: tos,
            to_asset: sol,
            value: BigUint::from(1_000_000_000u64),
            ..sol_to_tos.clone()
        };
        assert_eq!(pump_swap.get_quote(&tos_to_sol).await.unwrap().to_value, BigUint::from(29_841_209u64));
        assert_eq!(requests.load(Ordering::SeqCst), 4);

        let usdc_to_coin = QuoteRequest {
            from_asset: usdc.clone(),
            to_asset: coin,
            value: BigUint::from(1_000_000u64),
            ..sol_to_tos.clone()
        };
        assert_eq!(pump_swap.get_quote(&usdc_to_coin).await.unwrap().to_value, BigUint::from(514_845_170u64));
        assert_eq!(pump_swap.get_quote(&QuoteRequest { to_asset: usdc, ..sol_to_tos }).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 6);

        let quote_data = pump_swap.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        crate::contract_check::verify_solana_programs(&quote_data.data, &pump_swap.contracts(&quote.request.from_asset.asset_id()).targets).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 7);
        assert_eq!(quote_data.to, pump_amm_program().to_base58());
    }
}
