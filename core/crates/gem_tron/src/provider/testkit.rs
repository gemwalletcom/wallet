#[cfg(test)]
use primitives::DateTime;

#[cfg(test)]
use crate::address::TronAddress;
#[cfg(test)]
use crate::models::account::{TronAccount, TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey, TronAccountUsage, TronFrozen, TronUnfrozen, TronVote};
#[cfg(test)]
use crate::models::{ChainParameter, InternalTransaction, InternalTransactionCallValue, Transaction, TransactionReceipt, TransactionReceiptData, TriggerSmartContractRequest, TronLog, WitnessAccount, WitnessesList};
#[cfg(test)]
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;
#[cfg(test)]
use crate::rpc::trongrid::client::TronGridClient;
#[cfg(test)]
use crate::rpc::trongrid::model::TronGridTransaction;
#[cfg(test)]
use crate::rpc::{TronClient, TronProvider};
#[cfg(test)]
use gem_client::{ClientError, testkit::MockClient};
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_reqwest::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::asset_constants::TRON_USDT_TOKEN_ID;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;
#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "5a9935a1b7be0150a511111582bbfed62ddb873333b3986bd712e6105fe90ad5";
#[cfg(test)]
pub const TEST_TOKEN_APPROVAL_TRANSACTION_ID: &str = "5c8c1556e2c124dd74ed3639f9fc7d063d6a43729910ea415bfbe88cabbdbe7f";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_USDT_TOKEN_ID: &str = TRON_USDT_TOKEN_ID;

#[cfg(test)]
impl TronAddress {
    pub fn mock() -> Self {
        TronAddress::from_hex_or_base58("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM").unwrap()
    }
}

#[cfg(test)]
impl TronAccountUsage {
    pub fn mock(free_bandwidth: u64, staked_bandwidth: u64, available_energy: u64) -> Self {
        Self {
            free_net_used: 0,
            free_net_limit: free_bandwidth,
            net_used: 0,
            net_limit: staked_bandwidth,
            energy_used: 0,
            energy_limit: available_energy,
        }
    }
}

#[cfg(test)]
impl TronUnfrozen {
    pub fn mock(unfreeze_amount: u64, unfreeze_expire_millis: i64) -> Self {
        Self {
            unfreeze_amount,
            unfreeze_expire_time: Some(DateTime::from_timestamp_millis(unfreeze_expire_millis)),
        }
    }
}

#[cfg(test)]
impl ChainParameter {
    pub fn mock(key: &str, value: i64) -> Self {
        Self { key: key.to_string(), value: Some(value) }
    }
}

#[cfg(test)]
impl WitnessesList {
    pub fn mock() -> Self {
        Self {
            witnesses: vec![
                WitnessAccount {
                    address: "4159f3440fd40722f716144e4490a4de162d3b3fcb".to_string(),
                    vote_count: Some(1000000),
                    url: "https://validator1.com".to_string(),
                },
                WitnessAccount {
                    address: "41357a7401a0f0c2d4a44a1881a0c622f15d986291".to_string(),
                    vote_count: Some(500000),
                    url: "https://validator2.com".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
impl WitnessAccount {
    pub fn mock_with_vote_count(vote_count: i64) -> Self {
        Self {
            address: format!("41{vote_count:040x}"),
            vote_count: Some(vote_count),
            url: String::new(),
        }
    }
}

#[cfg(test)]
impl TronGridTransaction {
    pub fn mock(transaction_id: &str, block_timestamp: u64) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            block_timestamp,
        }
    }
}

#[cfg(test)]
impl TronAccountPermissionKey {
    pub fn mock(address: &str, weight: u64) -> Self {
        Self { address: address.to_string(), weight }
    }
}

#[cfg(test)]
impl TronAccount {
    pub fn mock(address: &str) -> Self {
        Self {
            balance: None,
            address: Some(address.to_string()),
            owner_permission: Some(TronAccountOwnerPermission {
                threshold: Some(1),
                keys: Some(vec![TronAccountPermissionKey { address: address.to_string(), weight: 1 }]),
            }),
            active_permission: Some(vec![TronAccountPermission {
                id: None,
                threshold: 1,
                keys: Some(vec![TronAccountPermissionKey { address: address.to_string(), weight: 1 }]),
            }]),
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        }
    }

    pub fn mock_with_staking(votes: Option<Vec<TronVote>>, frozen_v2: Option<Vec<TronFrozen>>) -> Self {
        Self {
            balance: None,
            address: None,
            owner_permission: None,
            active_permission: None,
            votes,
            frozen_v2,
            unfrozen_v2: None,
        }
    }
}

#[cfg(test)]
impl Transaction {
    pub fn mock_token_approval(contract_ret: &str) -> Self {
        let mut transaction: Self = serde_json::from_str(include_str!("../../testdata/transaction_token_approval.json")).unwrap();
        transaction.ret[0].contract_ret = contract_ret.to_string();
        transaction
    }
}

#[cfg(test)]
impl TransactionReceiptData {
    pub fn mock_with_result(result: &str) -> Self {
        Self {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 79874795,
            block_time_stamp: DateTime::from_timestamp_millis(1770288900000),
            result: None,
            receipt: TransactionReceipt { result: Some(result.to_string()) },
            log: None,
            internal_transactions: None,
        }
    }
}

#[cfg(test)]
impl TronLog {
    pub fn mock_transfer(token_address: &str, from_topic: &str, to_topic: &str, amount_hex: &str) -> Self {
        Self {
            address: TronAddress::from_hex_or_base58(token_address),
            topics: Some(vec![ERC20_TRANSFER_EVENT_SIGNATURE.to_string(), from_topic.to_string(), to_topic.to_string()]),
            data: Some(amount_hex.to_string()),
        }
    }
}

#[cfg(test)]
impl InternalTransaction {
    pub fn mock(caller: &str, transfer_to: &str, call_value: u64, token_id: Option<&str>, rejected: bool) -> Self {
        Self {
            caller_address: TronAddress::from_hex_or_base58(caller),
            transfer_to_address: TronAddress::from_hex_or_base58(transfer_to),
            call_value_info: vec![InternalTransactionCallValue {
                call_value,
                token_id: token_id.map(ToString::to_string),
            }],
            rejected,
        }
    }
}

#[cfg(test)]
impl TriggerSmartContractRequest {
    pub fn mock(owner_address: &str, contract_address: &str, data: &str, call_value: Option<u64>) -> Self {
        Self {
            owner_address: owner_address.to_string(),
            contract_address: contract_address.to_string(),
            data: data.to_string(),
            fee_limit: None,
            call_value,
            visible: true,
        }
    }
}

#[cfg(test)]
impl TronProvider<MockClient> {
    pub fn mock(get_handler: impl Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static) -> Self {
        Self::mock_with_client(MockClient::new().with_get(get_handler))
    }

    pub fn mock_with_post(get_handler: impl Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static, post_handler: impl Fn(&str, &[u8]) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static) -> Self {
        Self::mock_with_client(MockClient::new().with_get(get_handler).with_post(post_handler))
    }

    pub fn mock_estimates() -> (Self, Arc<Mutex<Vec<TriggerSmartContractRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let provider = Self::mock_with_post(
            |_| Ok(include_bytes!("../../testdata/chain_parameters.json").to_vec()),
            move |path, body| match path {
                "/wallet/getnowblock" => Ok(include_bytes!("../../testdata/now_block.json").to_vec()),
                "/wallet/getcontract" => Ok(br#"{"consume_user_resource_percent":100}"#.to_vec()),
                "/wallet/getaccount" => Ok(br#"{"address":"TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi"}"#.to_vec()),
                "/wallet/triggerconstantcontract" => {
                    recorded.lock().unwrap().push(serde_json::from_slice(body).unwrap());
                    Ok(include_bytes!("../../testdata/trigger_constant_contract_with_penalty.json").to_vec())
                }
                _ => Ok(b"{}".to_vec()),
            },
        );
        (provider, requests)
    }

    fn mock_with_client(mock: MockClient) -> Self {
        let trongrid = TronGridClient::new(mock.clone(), String::new());
        Self::new(TronClient::new(mock), Box::new(trongrid.clone()), Box::new(trongrid))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronProvider<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.tron.url, gem_reqwest::reqwest_client());
    let trongrid = settings.indexer.trongrid.remote_provider_config();
    let trongrid_client = TronGridClient::new(reqwest_client.clone().with_base_url(trongrid.url), trongrid.key);
    TronProvider::new(TronClient::new(reqwest_client), Box::new(trongrid_client.clone()), Box::new(trongrid_client))
}
