use gem_client::testkit::MockClient;
use gem_encoding::{borsh::BorshDeserialize, decode_base64, encode_base64};
use gem_jsonrpc::{client::JsonRpcClient, testkit::mock_jsonrpc_client};
use gem_solana::{
    DEFAULT_SWAP_GAS_LIMIT, Instruction, Pubkey, USDC_TOKEN_MINT,
    instructions::program_ids::{sysvar_clock, token_program},
    models::Clock,
};
use primitives::{AssetId, Chain, swap::SOLANA_REFERRAL_ADDRESS};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::{
    protocol::{DexProtocol, PoolSwap, QuoteAccount},
    token::SwapToken,
    transaction::DexTransaction,
};
use crate::{
    SwapperQuoteAsset,
    fees::{FeeSide, ReferralTransfer},
    testkit::TEST_SOLANA_PAYER,
};

pub(crate) fn mock_solana_asset(token_id: Option<&str>, decimals: u32) -> SwapperQuoteAsset {
    SwapperQuoteAsset {
        decimals,
        ..SwapperQuoteAsset::from(AssetId::from(Chain::Solana, token_id.map(str::to_string)))
    }
}

#[cfg(feature = "swap_integration_tests")]
pub(crate) const TEST_FUNDED_WALLET: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
pub(crate) const TEST_TOS_MINT: &str = "HmjCoarLh5duURfJ333DwfFiPyTCgFT35pRSAoP8pump";
#[cfg(feature = "swap_integration_tests")]
pub(crate) const TEST_TOS_HOLDER: &str = "2keyaLG5fUBEGFAboUZ7LajhpGygXSxKJBovHyTbJ9KB";

#[cfg(feature = "swap_integration_tests")]
pub(crate) async fn assert_swaps_simulate<P: DexProtocol>(protocol: P, swaps: Vec<(SwapperQuoteAsset, SwapperQuoteAsset, u64, &str)>) -> Result<(), crate::SwapperError> {
    use crate::{QuoteDataInput, QuoteRequest, Swapper, client_factory::create_client_with_chain, models::Options, solana_dex::SolanaDex};
    use gem_solana::SolanaClient;
    use num_bigint::BigUint;

    let rpc_provider = crate::testkit::dynode_provider([Chain::Solana]);
    let dex = SolanaDex::new(protocol, rpc_provider.clone()).unwrap();
    let client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana)?);
    for (from_asset, to_asset, value, wallet) in swaps {
        let request = QuoteRequest {
            from_asset,
            to_asset,
            wallet_address: wallet.to_string(),
            destination_address: wallet.to_string(),
            value: BigUint::from(value),
            options: Options::default(),
        };
        let quote = dex.get_quote(&request).await?;
        let quote_data = dex.get_quote_data(&quote, QuoteDataInput::None).await?;
        let simulation = client.simulate_encoded_transaction(&quote_data.data).await?;
        let units = simulation.units_consumed.unwrap();
        let gas_limit: u64 = quote_data.gas_limit.as_deref().unwrap().parse().unwrap();
        println!("to_value={} units={units} gas_limit={gas_limit} route={}", quote.to_value, quote.data.routes[0].route_data);

        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(simulation.err, None);
        assert!(units <= gas_limit, "{units} units exceed the {gas_limit} limit");
    }
    Ok(())
}

impl SwapToken {
    pub fn mock() -> Self {
        Self {
            mint: Pubkey::from_base58(USDC_TOKEN_MINT).unwrap(),
            token_program: token_program(),
            decimals: 6,
            is_native: false,
        }
    }
}

impl<'a> DexTransaction<'a> {
    pub fn mock(input: &'a SwapToken, output: &'a SwapToken) -> Self {
        Self {
            swap: PoolSwap {
                authority: Pubkey::from_base58(TEST_SOLANA_PAYER).unwrap(),
                input,
                intermediate: None,
                output,
                amount: 995_000,
                minimum_out: 0,
            },
            referral: Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap(),
            fee: ReferralTransfer { side: FeeSide::Output, value: 5_000 },
            swap_instructions: vec![Instruction {
                program_id: Pubkey::new([9; 32]),
                accounts: vec![],
                data: vec![],
            }],
            intermediate_token_account: false,
            compute_unit_limit: DEFAULT_SWAP_GAS_LIMIT,
        }
    }
}

pub(crate) fn mock_clock() -> Clock {
    Clock {
        slot: 0,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 0,
    }
}

pub(crate) fn mock_rpc_snapshot(snapshot: &str) -> Vec<(String, Value)> {
    let snapshot: Value = serde_json::from_str(snapshot).unwrap();
    snapshot["addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|address| address.as_str().unwrap().to_string())
        .zip(snapshot["result"]["value"].as_array().unwrap().iter().cloned())
        .collect()
}

pub(crate) fn mock_account_snapshot(snapshot: &str) -> (Vec<Pubkey>, HashMap<Pubkey, Option<Vec<u8>>>, Clock) {
    let snapshot = mock_rpc_snapshot(snapshot);
    let addresses = snapshot.iter().map(|(address, _)| Pubkey::from_base58(address).unwrap()).collect::<Vec<_>>();
    let accounts = snapshot
        .iter()
        .map(|(address, account)| (Pubkey::from_base58(address).unwrap(), account["data"][0].as_str().map(|data| decode_base64(data).unwrap())))
        .collect::<HashMap<_, _>>();
    let clock = Clock::try_from_slice(accounts[&sysvar_clock()].as_ref().unwrap()).unwrap();
    (addresses, accounts, clock)
}

pub(crate) struct PoolSnapshot<Pool> {
    pub address: Pubkey,
    pub pool: Pool,
    pub accounts: HashMap<Pubkey, Option<Vec<u8>>>,
    pub clock: Clock,
}

impl<Pool> PoolSnapshot<Pool> {
    pub fn quote_data(&self, accounts: &[QuoteAccount]) -> Vec<Option<Vec<u8>>> {
        accounts.iter().map(|account| self.accounts.get(&account.address()).cloned().flatten()).collect()
    }
}

pub(crate) fn mock_dex_pool_snapshot<P: DexProtocol>(protocol: &P, snapshot: &str) -> PoolSnapshot<P::Pool> {
    let (addresses, accounts, clock) = mock_account_snapshot(snapshot);
    PoolSnapshot {
        address: addresses[0],
        pool: protocol.decode_pool(accounts[&addresses[0]].as_ref().unwrap()).unwrap(),
        accounts,
        clock,
    }
}

pub(crate) fn mock_rpc_accounts(snapshot: &str) -> HashMap<String, Value> {
    mock_rpc_snapshot(snapshot).into_iter().collect()
}

pub(crate) fn mock_rpc_client(accounts: HashMap<String, Value>, requests: Arc<AtomicUsize>) -> JsonRpcClient<MockClient> {
    mock_rpc_client_with_sizes(accounts, requests, Arc::default())
}

pub(crate) fn mock_rpc_client_with_sizes(accounts: HashMap<String, Value>, requests: Arc<AtomicUsize>, sizes: Arc<Mutex<Vec<usize>>>) -> JsonRpcClient<MockClient> {
    mock_jsonrpc_client(move |method, params| match method {
        "getMultipleAccounts" => {
            requests.fetch_add(1, Ordering::SeqCst);
            let addresses = params[0].as_array().unwrap();
            sizes.lock().unwrap().push(addresses.len());
            let value = addresses
                .iter()
                .map(|address| accounts.get(address.as_str().unwrap()).map_or(Value::Null, |account| mock_account_slice(account, &params[1]["dataSlice"])));
            Ok(json!({ "context": { "slot": 1 }, "value": value.collect::<Vec<_>>() }))
        }
        "getProgramAccounts" => {
            requests.fetch_add(1, Ordering::SeqCst);
            Ok(Value::Array(accounts.iter().filter_map(|(address, account)| mock_program_account(address, account, params)).collect()))
        }
        method => panic!("unexpected method {method}"),
    })
}

fn mock_account_slice(account: &Value, slice: &Value) -> Value {
    let (Some(offset), Some(length)) = (slice["offset"].as_u64(), slice["length"].as_u64()) else {
        return account.clone();
    };
    let data = decode_base64(account["data"][0].as_str().unwrap()).unwrap();
    let range = usize::try_from(offset).unwrap()..usize::try_from(offset + length).unwrap();
    json!({ "data": [encode_base64(&data[range]), "base64"], "owner": account["owner"] })
}

fn mock_program_account(address: &str, account: &Value, params: &Value) -> Option<Value> {
    let data = decode_base64(account["data"][0].as_str()?).ok()?;
    let config = &params[1];
    let number = |value: &Value| usize::try_from(value.as_u64().unwrap()).unwrap();
    let matches = config["filters"].as_array().unwrap().iter().all(|filter| match (filter.get("dataSize"), filter.get("memcmp")) {
        (Some(size), _) => data.len() == number(size),
        (_, Some(memcmp)) => {
            let bytes = Pubkey::from_base58(memcmp["bytes"].as_str().unwrap()).unwrap();
            let start = number(&memcmp["offset"]);
            data.get(start..start + bytes.as_bytes().len()) == Some(bytes.as_bytes())
        }
        _ => false,
    });
    let slice_start = number(&config["dataSlice"]["offset"]);
    let slice = data.get(slice_start..slice_start + number(&config["dataSlice"]["length"]))?;
    (account["owner"] == params[0] && matches).then(|| json!({ "pubkey": address, "account": { "data": [encode_base64(slice), "base64"], "owner": account["owner"] } }))
}
