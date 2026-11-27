use gem_encoding::{decode_base64, encode_base64};
use gem_jsonrpc::types::JsonRpcResponse;
use gem_solana::{
    Pubkey, USDT_TOKEN_MINT,
    anchor::AnchorAccount,
    instructions::program_ids::{sysvar_clock, sysvar_program, token_program},
    models::{AccountData, Clock, ValueResult},
};
use serde_json::{Value, json};
use std::collections::HashMap;

use super::{tick_array::TickArray, whirlpool::Whirlpool};
use crate::solana_dex::testkit::{mock_account_snapshot, mock_rpc_accounts};

pub(super) const TEST_WHIRLPOOL: &str = "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ";
pub(super) const TEST_AQUA_MINT: &str = "AQVcP67EpMyu4cBZZjqMu91cVsWy1aX98JmcZm1FyY9";
pub(super) const TEST_AQUA_WHIRLPOOL: &str = "9TT3NbWogWjbQh7b3s1hfcqYF7VJHmg8uNcXDjtPRnh4";
pub(super) const TEST_PUMP_MINT: &str = "pumpCmXqMfrsAkQ5r49WcJnRayYRqmXz6ae8H7H9Dfn";
pub(super) const TEST_PUMP_WHIRLPOOL: &str = "BofA2ViUSudPBTUms2KRuG6AHNeMawjNfwqTJDgx5BKW";
pub(super) const TEST_XAUT0_MINT: &str = "AymATz4TCL9sWNEEV9Kvyz45CHVhDZ6kUgjTJPzLpU9P";
pub(super) const TEST_XAUT0_SOL_WHIRLPOOL: &str = "B2q3X5QA47NTaVrQKNhGo2ii1Zs8x5h6CZK6qydRdJrA";
pub(super) const TEST_XAUT0_USDC_WHIRLPOOL: &str = "B3QsKvpsBYVAuJBzyKRBWBJw8CAM45zStdJhtE1miBri";
pub(super) const TEST_MNDE_MINT: &str = "MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey";
pub(super) const TEST_MNDE_SOL_WHIRLPOOL: &str = "ByXmPDEzK68zV2werup4fhhhUPwXDWD1PWBqkXEdbQyi";
pub(super) const TEST_MON_MINT: &str = "CrAr4RRJMBVwRsZtT62pEhfA9H5utymC2mVx8e7FreP2";
pub(super) const TEST_MON_SOL_WHIRLPOOL: &str = "14K3BM1FA6gn4F9X1h5E9uriMZZY2xyz3fQ72bL628un";
pub(super) const TEST_TOKEN_2022_SNAPSHOT: &str = include_str!("../../testdata/orca/token_2022_snapshot.json");
pub(super) const TEST_PUMP_SNAPSHOT: &str = include_str!("../../testdata/orca/pump_snapshot.json");
const TEST_CLOCK: &str = "+ZJIGwAAAAB7mtFqAAAAACMEAAAAAAAAJAQAAAAAAAD0b9JqAAAAAA==";

fn mock_snapshot_accounts() -> Vec<Vec<u8>> {
    let response: JsonRpcResponse<ValueResult<Vec<Option<AccountData>>>> = serde_json::from_str(include_str!("../../testdata/orca/snapshot.json")).unwrap();
    response.result.value.into_iter().map(|account| decode_base64(&account.unwrap().data[0]).unwrap()).collect()
}

pub(super) fn mock_tick_array_accounts(start_tick_indexes: &[i32]) -> Vec<Option<Vec<u8>>> {
    let accounts = mock_snapshot_accounts();
    start_tick_indexes
        .iter()
        .map(|start| accounts.iter().skip(1).find(|data| TickArray::decode(data).unwrap().start_tick_index == *start).cloned())
        .collect()
}

pub(super) fn mock_dynamic_tick_array() -> Vec<u8> {
    decode_base64(include_str!("../../testdata/orca/dynamic_tick_array.base64").trim()).unwrap()
}

pub(super) fn mock_whirlpool_account(mint_a: &Pubkey, mint_b: &Pubkey, fee_tier_index: u16) -> Vec<u8> {
    let data = mock_snapshot_accounts().into_iter().next().unwrap();
    let tick_spacing = if fee_tier_index < 1024 { fee_tier_index } else { 64 };
    let liquidity = u128::from(fee_tier_index).to_le_bytes();
    [
        &data[..41],
        &tick_spacing.to_le_bytes(),
        &fee_tier_index.to_le_bytes(),
        &data[45..49],
        &liquidity,
        &data[65..101],
        mint_a.as_bytes(),
        &data[133..181],
        mint_b.as_bytes(),
        &data[213..],
    ]
    .concat()
}

impl Whirlpool {
    pub fn mock() -> Self {
        Self::decode(&mock_snapshot_accounts()[0]).unwrap()
    }
}

pub(super) fn mock_snapshot_rpc_accounts() -> HashMap<String, Value> {
    let response: Value = serde_json::from_str(include_str!("../../testdata/orca/snapshot.json")).unwrap();
    let whirlpool = Pubkey::from_base58(TEST_WHIRLPOOL).unwrap();
    let clock = json!({ "data": [TEST_CLOCK, "base64"], "owner": sysvar_program().to_base58() });
    let pool = Whirlpool::mock();
    let mints = [pool.token_mint_a, pool.token_mint_b, Pubkey::from_base58(USDT_TOKEN_MINT).unwrap()].map(|mint| (mint.to_base58(), json!({ "data": [encode_base64(&[0; 82]), "base64"], "owner": token_program().to_base58() })));
    response["result"]["value"]
        .as_array()
        .unwrap()
        .iter()
        .zip(mock_snapshot_accounts())
        .enumerate()
        .map(|(index, (account, data))| {
            let address = if index == 0 {
                whirlpool
            } else {
                TickArray::address(&whirlpool, TickArray::decode(&data).unwrap().start_tick_index).unwrap()
            };
            (address.to_base58(), account.clone())
        })
        .chain([(sysvar_clock().to_base58(), clock)])
        .chain(mints)
        .collect()
}

pub(super) fn mock_aqua_whirlpool() -> (Pubkey, Whirlpool) {
    let data = mock_rpc_accounts(TEST_TOKEN_2022_SNAPSHOT)[TEST_AQUA_WHIRLPOOL]["data"][0].as_str().map(|data| decode_base64(data).unwrap()).unwrap();
    (Pubkey::from_base58(TEST_AQUA_WHIRLPOOL).unwrap(), Whirlpool::decode(&data).unwrap())
}

pub(super) fn mock_adaptive_snapshot() -> (Pubkey, HashMap<Pubkey, Option<Vec<u8>>>, Clock) {
    let (addresses, accounts, clock) = mock_account_snapshot(include_str!("../../testdata/orca/adaptive_snapshot.json"));
    (addresses[0], accounts, clock)
}
