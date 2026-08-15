use crate::{COMMITMENT_CONFIRMED, method};
use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub enum SolanaAccountEncoding {
    Base64,
    JsonParsed,
}

impl SolanaAccountEncoding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Base64 => "base64",
            Self::JsonParsed => "jsonParsed",
        }
    }
}

#[derive(Clone, Debug)]
pub enum SolanaTokenAccountsFilter {
    Mint(String),
    ProgramId(String),
}

impl SolanaTokenAccountsFilter {
    fn value(&self) -> Value {
        match self {
            Self::Mint(mint) => json!({ "mint": mint }),
            Self::ProgramId(program_id) => json!({ "programId": program_id }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SolanaProgramAccountsFilter {
    Memcmp { offset: u8, bytes: String },
}

impl SolanaProgramAccountsFilter {
    fn value(&self) -> Value {
        match self {
            Self::Memcmp { offset, bytes } => json!({ "memcmp": { "offset": offset, "bytes": bytes } }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SolanaRpcConfig {
    #[default]
    Default,
    Confirmed,
}

#[derive(Clone, Debug)]
pub enum SolanaRpc {
    GetAccountInfo(String, SolanaAccountEncoding),
    GetBalance(String),
    GetBlock(u64),
    GetEpochInfo(SolanaRpcConfig),
    GetGenesisHash,
    GetInflationRate,
    GetLatestBlockhash(SolanaRpcConfig),
    GetMultipleAccounts(Vec<String>),
    GetProgramAccounts(String, Vec<SolanaProgramAccountsFilter>),
    GetRecentPrioritizationFees(Vec<String>),
    GetSignaturesForAddress { address: String, limit: usize },
    GetSlot(SolanaRpcConfig),
    GetSupply,
    GetTokenAccountsByOwner(String, SolanaTokenAccountsFilter),
    GetTransaction(String),
    GetVoteAccounts { keep_unstaked_delinquents: bool },
    SendTransaction { data: String, skip_preflight: Option<bool> },
    SimulateTransaction(String),
}

impl ToJsonRpcRequest for SolanaRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::GetAccountInfo(_, _) => method::GET_ACCOUNT_INFO,
            Self::GetBalance(_) => method::GET_BALANCE,
            Self::GetBlock(_) => method::GET_BLOCK,
            Self::GetEpochInfo(_) => method::GET_EPOCH_INFO,
            Self::GetGenesisHash => method::GET_GENESIS_HASH,
            Self::GetInflationRate => method::GET_INFLATION_RATE,
            Self::GetLatestBlockhash(_) => method::GET_LATEST_BLOCKHASH,
            Self::GetMultipleAccounts(_) => method::GET_MULTIPLE_ACCOUNTS,
            Self::GetProgramAccounts(_, _) => method::GET_PROGRAM_ACCOUNTS,
            Self::GetRecentPrioritizationFees(_) => method::GET_RECENT_PRIORITIZATION_FEES,
            Self::GetSignaturesForAddress { .. } => method::GET_SIGNATURES_FOR_ADDRESS,
            Self::GetSlot(_) => method::GET_SLOT,
            Self::GetSupply => method::GET_SUPPLY,
            Self::GetTokenAccountsByOwner(_, _) => method::GET_TOKEN_ACCOUNTS_BY_OWNER,
            Self::GetTransaction(_) => method::GET_TRANSACTION,
            Self::GetVoteAccounts { .. } => method::GET_VOTE_ACCOUNTS,
            Self::SendTransaction { .. } => method::SEND_TRANSACTION,
            Self::SimulateTransaction(_) => method::SIMULATE_TRANSACTION,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::GetAccountInfo(address, encoding) => json!([address, confirmed_encoding_config(*encoding)]),
            Self::GetBalance(address) => json!([address, confirmed_config(json!({}))]),
            Self::GetBlock(slot) => json!([
                slot,
                confirmed_config(json!({
                    "encoding": "json",
                    "transactionDetails": "full",
                    "rewards": false,
                    "maxSupportedTransactionVersion": 0,
                }))
            ]),
            Self::GetEpochInfo(config) | Self::GetLatestBlockhash(config) | Self::GetSlot(config) => config.params(),
            Self::GetGenesisHash | Self::GetInflationRate => json!([]),
            Self::GetMultipleAccounts(addresses) => json!([addresses, confirmed_encoding_config(SolanaAccountEncoding::Base64)]),
            Self::GetProgramAccounts(program, filters) => json!([
                program,
                confirmed_config(json!({
                    "encoding": SolanaAccountEncoding::JsonParsed.as_str(),
                    "filters": filters.iter().map(SolanaProgramAccountsFilter::value).collect::<Vec<_>>(),
                }))
            ]),
            Self::GetRecentPrioritizationFees(addresses) => {
                if addresses.is_empty() {
                    json!([])
                } else {
                    json!([addresses])
                }
            }
            Self::GetSignaturesForAddress { address, limit } => json!([
                address,
                confirmed_config(json!({
                    "limit": limit,
                }))
            ]),
            Self::GetSupply => json!([confirmed_config(json!({
                "excludeNonCirculatingAccountsList": true,
            }))]),
            Self::GetTokenAccountsByOwner(owner, filter) => json!([owner, filter.value(), confirmed_encoding_config(SolanaAccountEncoding::JsonParsed)]),
            Self::GetTransaction(signature) => json!([
                signature,
                confirmed_config(json!({
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0,
                }))
            ]),
            Self::GetVoteAccounts { keep_unstaked_delinquents } => json!([confirmed_config(json!({
                "keepUnstakedDelinquents": keep_unstaked_delinquents,
            }))]),
            Self::SendTransaction { data, skip_preflight } => {
                let mut config = json!({
                    "encoding": SolanaAccountEncoding::Base64.as_str(),
                    "preflightCommitment": COMMITMENT_CONFIRMED,
                });
                if let Some(skip_preflight) = skip_preflight {
                    config["skipPreflight"] = (*skip_preflight).into();
                }
                json!([data, config])
            }
            Self::SimulateTransaction(encoded_transaction) => json!([
                encoded_transaction,
                confirmed_config(json!({
                    "encoding": SolanaAccountEncoding::Base64.as_str(),
                    "sigVerify": false,
                    "replaceRecentBlockhash": true,
                }))
            ]),
        }
    }
}

fn confirmed_config(mut config: Value) -> Value {
    config
        .as_object_mut()
        .expect("Solana RPC configuration must be a JSON object")
        .insert("commitment".to_string(), COMMITMENT_CONFIRMED.into());
    config
}

fn confirmed_encoding_config(encoding: SolanaAccountEncoding) -> Value {
    confirmed_config(json!({ "encoding": encoding.as_str() }))
}

impl SolanaRpcConfig {
    fn params(self) -> Value {
        match self {
            Self::Default => json!([]),
            Self::Confirmed => json!([confirmed_config(json!({}))]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_request(rpc: SolanaRpc, method: &str, params: Value) {
        let request = rpc.to_jsonrpc_request(42);
        assert_eq!(request.id, 42);
        assert_eq!(request.method, method);
        assert_eq!(request.params, params);
    }

    #[test]
    fn builds_configured_account_request() {
        assert_request(
            SolanaRpc::GetAccountInfo("address".into(), SolanaAccountEncoding::Base64),
            method::GET_ACCOUNT_INFO,
            json!(["address", {"commitment": "confirmed", "encoding": "base64"}]),
        );
    }

    #[test]
    fn builds_token_accounts_request() {
        assert_request(
            SolanaRpc::GetTokenAccountsByOwner("owner".into(), SolanaTokenAccountsFilter::Mint("mint".into())),
            method::GET_TOKEN_ACCOUNTS_BY_OWNER,
            json!(["owner", {"mint": "mint"}, {"commitment": "confirmed", "encoding": "jsonParsed"}]),
        );
    }

    #[test]
    fn builds_program_accounts_filter_request() {
        assert_request(
            SolanaRpc::GetProgramAccounts(
                "program".into(),
                vec![SolanaProgramAccountsFilter::Memcmp {
                    offset: 12,
                    bytes: "owner".into(),
                }],
            ),
            method::GET_PROGRAM_ACCOUNTS,
            json!(["program", {
                "commitment": "confirmed",
                "encoding": "jsonParsed",
                "filters": [{"memcmp": {"offset": 12, "bytes": "owner"}}]
            }]),
        );
    }

    #[test]
    fn omits_optional_config_when_absent() {
        assert_request(SolanaRpc::GetLatestBlockhash(SolanaRpcConfig::Default), method::GET_LATEST_BLOCKHASH, json!([]));
    }

    #[test]
    fn includes_confirmed_config_when_requested() {
        assert_request(SolanaRpc::GetSlot(SolanaRpcConfig::Confirmed), method::GET_SLOT, json!([{"commitment": "confirmed"}]));
    }

    #[test]
    fn excludes_supply_account_list() {
        assert_request(
            SolanaRpc::GetSupply,
            method::GET_SUPPLY,
            json!([{"commitment": "confirmed", "excludeNonCirculatingAccountsList": true}]),
        );
    }

    #[test]
    fn omits_empty_prioritization_fee_accounts() {
        assert_request(SolanaRpc::GetRecentPrioritizationFees(vec![]), method::GET_RECENT_PRIORITIZATION_FEES, json!([]));
    }

    #[test]
    fn builds_broadcast_request() {
        assert_request(
            SolanaRpc::SendTransaction {
                data: "signed-transaction".into(),
                skip_preflight: Some(true),
            },
            method::SEND_TRANSACTION,
            json!(["signed-transaction", {"encoding": "base64", "preflightCommitment": "confirmed", "skipPreflight": true}]),
        );
    }
}
