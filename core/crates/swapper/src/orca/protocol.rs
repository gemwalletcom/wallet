use gem_solana::{Instruction, Pubkey, anchor::AnchorAccount, instructions::program_ids::orca_whirlpool_program, models::Clock};

use super::{
    oracle::Oracle,
    swap::{swap_v2_instruction, two_hop_swap_v2_instruction},
    tick_array::{TickArray, swap_ticks},
    whirlpool::Whirlpool,
};
use crate::{
    SwapperError, SwapperProvider,
    clmm::{StaticFee, quote_exact_in},
    solana_dex::{DexPool, DexProtocol, INTERMEDIATE_TOKENS, NativeTwoHop, PoolSwap, QuoteAccount, SwapToken},
};

pub(crate) struct Orca;

impl DexProtocol for Orca {
    type Pool = Whirlpool;

    const PROVIDER: SwapperProvider = SwapperProvider::Orca;
    const PROGRAM_ID: Pubkey = orca_whirlpool_program();
    const PROGRAM: &'static str = gem_solana::ORCA_WHIRLPOOL_PROGRAM_ID;
    const INTERMEDIATE_TOKEN_ACCOUNT: bool = false;

    fn pool_addresses(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        Whirlpool::addresses(input_mint, output_mint)
    }

    fn decode_pool(&self, data: &[u8]) -> Result<Whirlpool, SwapperError> {
        Ok(Whirlpool::decode(data)?)
    }

    fn is_tradable(&self, pool: &Whirlpool, _clock: &Clock) -> bool {
        pool.liquidity > 0
    }

    fn intermediate_tokens(&self) -> &'static [SwapToken] {
        &INTERMEDIATE_TOKENS
    }

    fn quote_accounts(&self, address: &Pubkey, pool: &Whirlpool, input_mint: &Pubkey) -> Result<Vec<QuoteAccount>, SwapperError> {
        let tick_arrays = TickArray::addresses(address, &pool.swap_start_tick_indexes(pool.is_a_to_b(input_mint)?))?;
        let oracle = pool.has_adaptive_fee().then(|| Whirlpool::oracle_address(address)).transpose()?;
        Ok(tick_arrays.into_iter().chain(oracle).map(|address| QuoteAccount::Owned { address, owner: Self::PROGRAM_ID }).collect())
    }

    fn quote(&self, address: &Pubkey, pool: &Whirlpool, input_mint: &Pubkey, amount_in: u64, accounts: &[Option<Vec<u8>>], clock: &Clock) -> Result<u64, SwapperError> {
        let a_to_b = pool.is_a_to_b(input_mint)?;
        let (oracle, tick_arrays) = if pool.has_adaptive_fee() {
            accounts.split_last().map(|(oracle, tick_arrays)| (oracle.as_ref(), tick_arrays)).ok_or(SwapperError::NoQuoteAvailable)?
        } else {
            (None, accounts)
        };
        let ticks = swap_ticks(address, pool, a_to_b, tick_arrays)?;
        let swap = pool.clmm_swap(a_to_b);
        match oracle {
            Some(data) => {
                let fee = Oracle::decode_for_whirlpool(data, address)?.adaptive_fee(pool.fee_rate, pool.tick_current_index, a_to_b, clock.timestamp())?;
                quote_exact_in(&swap, fee, amount_in, &ticks)
            }
            None => quote_exact_in(&swap, StaticFee(u32::from(pool.fee_rate)), amount_in, &ticks),
        }
    }

    fn swap_instruction(&self, address: &Pubkey, pool: &Whirlpool, swap: &PoolSwap) -> Result<Instruction, SwapperError> {
        swap_v2_instruction(address, pool, swap)
    }

    fn native_two_hop(&self) -> Option<&dyn NativeTwoHop<Whirlpool>> {
        Some(self)
    }
}

impl NativeTwoHop<Whirlpool> for Orca {
    fn liquidity(&self, pool: &Whirlpool) -> u128 {
        pool.liquidity
    }

    fn two_hop_swap_instruction(&self, first: &(Pubkey, Whirlpool), second: &(Pubkey, Whirlpool), swap: &PoolSwap, intermediate: &SwapToken) -> Result<Instruction, SwapperError> {
        two_hop_swap_v2_instruction(first, second, swap, intermediate)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        SwapperQuoteAsset,
        orca::testkit::TEST_PUMP_MINT,
        solana_dex::testkit::{TEST_FUNDED_WALLET, assert_swaps_simulate, mock_solana_asset},
    };
    use gem_solana::USDC_TOKEN_MINT;
    use primitives::AssetType;

    const TEST_JUP_MINT: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

    #[tokio::test]
    async fn test_orca_swap_simulates() -> Result<(), SwapperError> {
        let sol = mock_solana_asset(None, 9);
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let pump = SwapperQuoteAsset {
            asset_type: AssetType::SPL2022,
            ..mock_solana_asset(Some(TEST_PUMP_MINT), 6)
        };
        let jup = mock_solana_asset(Some(TEST_JUP_MINT), 6);

        assert_swaps_simulate(
            Orca,
            vec![
                (sol.clone(), usdc.clone(), 1_000_000_000, TEST_FUNDED_WALLET),
                (usdc.clone(), sol.clone(), 100_000_000, TEST_FUNDED_WALLET),
                (sol, pump, 100_000_000, TEST_FUNDED_WALLET),
                (usdc, jup, 1_000_000_000, TEST_FUNDED_WALLET),
            ],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FetchQuoteData, ProviderData, Quote, QuoteRequest, Route, Swapper, SwapperQuoteAsset,
        orca::{
            testkit::{
                TEST_AQUA_MINT, TEST_PUMP_MINT, TEST_PUMP_SNAPSHOT, TEST_PUMP_WHIRLPOOL, TEST_TOKEN_2022_SNAPSHOT, TEST_WHIRLPOOL, mock_adaptive_snapshot, mock_snapshot_rpc_accounts, mock_tick_array_accounts, mock_whirlpool_account,
            },
            whirlpool::FEE_TIER_INDEXES,
        },
        solana_dex::{
            SolanaDex,
            testkit::{mock_clock, mock_rpc_accounts, mock_rpc_client, mock_rpc_client_with_sizes, mock_solana_asset},
        },
        testkit::mock_quote,
    };
    use gem_encoding::{decode_base64, encode_base64};
    use gem_solana::{USDC_TOKEN_MINT, USDT_TOKEN_MINT, instructions::program_ids::token_program};
    use num_bigint::BigUint;
    use primitives::{AssetType, swap::SOLANA_REFERRAL_ADDRESS, testkit::signer_mock::TEST_SOLANA_SENDER};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    fn sol_to_usdc_request(value: u64) -> QuoteRequest {
        QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(value),
            ..mock_quote(mock_solana_asset(None, 9), mock_solana_asset(Some(USDC_TOKEN_MINT), 6))
        }
    }

    #[tokio::test]
    async fn test_swap_requests() {
        let requests = Arc::new(AtomicUsize::new(0));
        let orca = SolanaDex::with_client(Orca, mock_rpc_client(mock_snapshot_rpc_accounts(), requests.clone()));
        let request = sol_to_usdc_request(1_000_000_000);

        let quote = orca.get_quote(&request).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(quote.to_value, BigUint::from(104_629_923u64));

        assert_eq!(orca.get_quote(&request).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 3);

        let unsupported = QuoteRequest {
            to_asset: mock_solana_asset(Some(USDT_TOKEN_MINT), 6),
            ..request.clone()
        };
        assert_eq!(orca.get_quote(&unsupported).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 4);
        assert_eq!(orca.get_quote(&unsupported).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 4);

        let quote_data = orca.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        crate::contract_check::verify_solana_programs(&quote_data.data, &orca.contracts(&quote.request.from_asset.asset_id()).targets).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 5);
        assert_eq!(quote_data.to, orca_whirlpool_program().to_base58());

        let route_data = quote.data.routes[0].route_data.replace(TEST_WHIRLPOOL, &Pubkey::new([1; 32]).to_base58());
        let tampered = Quote {
            data: ProviderData {
                routes: vec![Route { route_data, ..quote.data.routes[0].clone() }],
                ..quote.data.clone()
            },
            ..quote
        };
        assert_eq!(orca.get_quote_data(&tampered, FetchQuoteData::None).await, Err(SwapperError::InvalidRoute));
        assert_eq!(requests.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_quote_requests_stay_under_the_account_limit() {
        let (input, output) = (Pubkey::new([7; 32]), Pubkey::new([9; 32]));
        let (sol, usdc) = (Whirlpool::mock().token_mint_a, Whirlpool::mock().token_mint_b);
        let account = |data: Vec<u8>, owner: Pubkey| serde_json::json!({ "data": [encode_base64(&data), "base64"], "owner": owner.to_base58() });
        let pools = [(input, output), (input, sol), (sol, output), (input, usdc), (usdc, output)].into_iter().flat_map(|(mint_0, mint_1)| {
            let (mint_a, mint_b) = if mint_0 < mint_1 { (mint_0, mint_1) } else { (mint_1, mint_0) };
            let addresses = Whirlpool::addresses(&mint_a, &mint_b).unwrap();
            addresses
                .into_iter()
                .zip(FEE_TIER_INDEXES)
                .map(move |(address, tier)| (address.to_base58(), account(mock_whirlpool_account(&mint_a, &mint_b, tier), orca_whirlpool_program())))
        });
        let referral = Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap();
        let tokens = [input, output].map(|mint| SwapToken { mint, ..SwapToken::mock() });
        let mints = tokens
            .iter()
            .flat_map(|token| [token.mint, token.account(&referral).unwrap()])
            .map(|address| (address.to_base58(), account(vec![0; 82], token_program())));
        let accounts = mock_snapshot_rpc_accounts().into_iter().chain(pools).chain(mints).collect();
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let orca = SolanaDex::with_client(Orca, mock_rpc_client_with_sizes(accounts, Arc::default(), sizes.clone()));
        let request = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(1_000_000u64),
            ..mock_quote(mock_solana_asset(Some(&input.to_base58()), 6), mock_solana_asset(Some(&output.to_base58()), 6))
        };

        assert_eq!(orca.get_quote(&request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(orca.get_quote(&request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(*sizes.lock().unwrap(), [50, 47, 93]);
    }

    #[tokio::test]
    async fn test_untradable_pool_is_not_requested_again() {
        let requests = Arc::new(AtomicUsize::new(0));
        let liquidity = Whirlpool::mock().liquidity.to_le_bytes();
        let accounts = mock_snapshot_rpc_accounts()
            .into_iter()
            .map(|(address, account)| match address.as_str() {
                TEST_WHIRLPOOL => {
                    let data = decode_base64(account["data"][0].as_str().unwrap()).unwrap();
                    let offset = data.windows(liquidity.len()).position(|window| window == liquidity).unwrap();
                    let drained = [&data[..offset], &[0; 16], &data[offset + 16..]].concat();
                    (address, serde_json::json!({ "data": [encode_base64(&drained), "base64"], "owner": account["owner"] }))
                }
                _ => (address, account),
            })
            .collect();
        let orca = SolanaDex::with_client(Orca, mock_rpc_client(accounts, requests.clone()));
        let request = sol_to_usdc_request(1_000_000_000);

        assert_eq!(orca.get_quote(&request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        assert_eq!(orca.get_quote(&request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_route_failing_alone_is_asked_again() {
        let requests = Arc::new(AtomicUsize::new(0));
        let orca = SolanaDex::with_client(Orca, mock_rpc_client(mock_snapshot_rpc_accounts(), requests.clone()));
        let too_large = sol_to_usdc_request(1_000_000_000_000_000);

        assert_eq!(orca.get_quote(&too_large).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(orca.get_quote(&too_large).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(requests.load(Ordering::SeqCst), 3);

        assert_eq!(orca.get_quote(&sol_to_usdc_request(1_000_000_000)).await.unwrap().to_value, BigUint::from(104_629_923u64));
        assert_eq!(requests.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_quote_transfer_fee() {
        let fee_mint = mock_rpc_accounts(TEST_TOKEN_2022_SNAPSHOT)[TEST_AQUA_MINT].clone();
        let accounts = mock_snapshot_rpc_accounts().into_iter().chain([(USDC_TOKEN_MINT.to_string(), fee_mint)]).collect();
        let orca = SolanaDex::with_client(Orca, mock_rpc_client(accounts, Arc::default()));
        let sol = mock_solana_asset(None, 9);
        let aqua = SwapperQuoteAsset {
            asset_type: AssetType::SPL2022,
            ..mock_solana_asset(Some(USDC_TOKEN_MINT), 6)
        };
        let request = |from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset| QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(100_000_000u64),
            ..mock_quote(from_asset.clone(), to_asset.clone())
        };
        let spl_aqua = SwapperQuoteAsset { asset_type: AssetType::SPL, ..aqua.clone() };

        assert_eq!(orca.get_quote(&request(&sol, &aqua)).await.unwrap().to_value, BigUint::from(10_267_028u64));
        assert_eq!(orca.get_quote(&request(&aqua, &sol)).await.unwrap().to_value, BigUint::from(919_174_743u64));
        assert_eq!(orca.get_quote(&request(&spl_aqua, &sol)).await, Err(SwapperError::NotSupportedAsset));
    }

    #[tokio::test]
    async fn test_two_hop_quote() {
        let requests = Arc::new(AtomicUsize::new(0));
        let accounts = mock_snapshot_rpc_accounts().into_iter().chain(mock_rpc_accounts(TEST_PUMP_SNAPSHOT)).collect();
        let orca = SolanaDex::with_client(Orca, mock_rpc_client(accounts, requests.clone()));
        let request = QuoteRequest {
            wallet_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
            value: BigUint::from(100_000_000u64),
            ..mock_quote(
                mock_solana_asset(Some(USDC_TOKEN_MINT), 6),
                SwapperQuoteAsset {
                    asset_type: AssetType::SPL2022,
                    ..mock_solana_asset(Some(TEST_PUMP_MINT), 6)
                },
            )
        };

        let quote = orca.get_quote(&request).await.unwrap();
        let route: serde_json::Value = serde_json::from_str(&quote.data.routes[0].route_data).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(quote.to_value, BigUint::from(21_763_608_986u64));
        assert_eq!([&route["hops"][0]["pool"], &route["hops"][1]["pool"]], [TEST_WHIRLPOOL, TEST_PUMP_WHIRLPOOL]);

        assert_eq!(orca.get_quote(&request).await.unwrap(), quote);
        assert_eq!(requests.load(Ordering::SeqCst), 3);

        let quote_data = orca.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        crate::contract_check::verify_solana_programs(&quote_data.data, &orca.contracts(&quote.request.from_asset.asset_id()).targets).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 4);
        assert_eq!(quote_data.to, orca_whirlpool_program().to_base58());
    }

    #[test]
    fn test_quote_adaptive_fee() {
        let (address, accounts, clock) = mock_adaptive_snapshot();
        let whirlpool = Whirlpool::decode(accounts[&address].as_ref().unwrap()).unwrap();
        let (mint_a, mint_b) = (whirlpool.token_mint_a, whirlpool.token_mint_b);
        let quote = |input_mint: &Pubkey, amount_in: u64, with_oracle: bool| {
            let addresses = Orca.quote_accounts(&address, &whirlpool, input_mint).unwrap();
            let (oracle, tick_arrays) = addresses.split_last().unwrap();
            let data = tick_arrays
                .iter()
                .map(|account| accounts.get(&account.address()).cloned().flatten())
                .chain([with_oracle.then(|| accounts[&oracle.address()].clone()).flatten()])
                .collect::<Vec<_>>();
            Orca.quote(&address, &whirlpool, input_mint, amount_in, &data, &clock)
        };

        assert_eq!(quote(&mint_a, 5_000_000_000, true), Ok(16_699_318_026));
        assert_eq!(quote(&mint_a, 5_000_000_000, false), Ok(16_699_318_026));
        assert_eq!(quote(&mint_a, 50_000_000_000, true), Ok(166_443_339_402));
        assert_eq!(quote(&mint_a, 50_000_000_000, false), Ok(166_643_734_366));
        assert_eq!(quote(&mint_b, 1_000_000_000_000, true), Ok(288_939_129_049));
        assert_eq!(quote(&mint_b, 1_000_000_000_000, false), Ok(291_424_735_094));
        assert_eq!(quote(&mint_b, 3_000_000_000_000, true), Err(SwapperError::NoQuoteAvailable));
        let oracle = accounts[&Whirlpool::oracle_address(&address).unwrap()].clone();
        assert_eq!(
            Orca.quote(&address, &whirlpool, &mint_a, 1, &[None, None, None, oracle], &mock_clock()),
            Err(SwapperError::compute_quote_error("Oracle is ahead of the chain clock"))
        );
    }

    #[test]
    fn test_quote() {
        let whirlpool = Whirlpool::mock();
        let address = Pubkey::from_base58(TEST_WHIRLPOOL).unwrap();
        let (sol, usdc) = (whirlpool.token_mint_a, whirlpool.token_mint_b);
        let quote = |input_mint: &Pubkey, amount_in: u64| {
            let accounts = mock_tick_array_accounts(&whirlpool.swap_start_tick_indexes(*input_mint == sol));
            Orca.quote(&address, &whirlpool, input_mint, amount_in, &accounts, &mock_clock())
        };

        assert_eq!(quote(&sol, 1_000_000_000), Ok(105_154_941));
        assert_eq!(quote(&sol, 1_000_000_000_000), Ok(52_579_665_306));
        assert_eq!(quote(&sol, 5_000_000_000_000), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(quote(&usdc, 100_000_000), Ok(942_620_829));
        assert_eq!(quote(&usdc, 1_000_000_000), Ok(9_311_427_487));
        assert_eq!(quote(&usdc, 100_000_000_000), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(quote(&Pubkey::new([1; 32]), 1), Err(SwapperError::InvalidRoute));
    }
}
