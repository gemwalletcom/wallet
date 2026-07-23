#[cfg(test)]
use crate::address::TronAddress;
#[cfg(test)]
use crate::models::account::{TronAccount, TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey, TronFrozen, TronVote};
#[cfg(test)]
use crate::models::{InternalTransaction, InternalTransactionCallValue, Transaction, TransactionReceipt, TransactionReceiptData, TronLog};
#[cfg(test)]
use crate::rpc::client::TronClient;
#[cfg(test)]
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;
#[cfg(test)]
use crate::rpc::trongrid::client::TronGridClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(test)]
use gem_client::{ClientError, testkit::MockClient};
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::asset_constants::TRON_USDT_TOKEN_ID;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "5a9935a1b7be0150a511111582bbfed62ddb873333b3986bd712e6105fe90ad5";
#[cfg(test)]
pub const TEST_TOKEN_APPROVAL_TRANSACTION_ID: &str = "5c8c1556e2c124dd74ed3639f9fc7d063d6a43729910ea415bfbe88cabbdbe7f";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_USDT_TOKEN_ID: &str = TRON_USDT_TOKEN_ID;

#[cfg(test)]
impl TronAccount {
    pub fn mock(address: &str) -> Self {
        Self {
            balance: None,
            address: Some(address.to_string()),
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                threshold: Some(1),
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
            }),
            active_permission: Some(vec![TronAccountPermission {
                id: None,
                threshold: 1,
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
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
    pub fn mock_transaction_receipt_with_result(result: &str) -> Self {
        Self {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 79874795,
            block_time_stamp: 1770288900000,
            result: None,
            receipt: TransactionReceipt { result: Some(result.to_string()) },
            log: None,
            internal_transactions: None,
        }
    }

    pub fn mock_transaction_receipt_with_logs(log: Vec<TronLog>, internal_transactions: Vec<InternalTransaction>) -> Self {
        Self {
            log: Some(log),
            internal_transactions: Some(internal_transactions),
            ..Self::mock_transaction_receipt_with_result("SUCCESS")
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
                token_id: token_id.map(|token_id| token_id.to_string()),
            }],
            rejected,
        }
    }
}

#[cfg(test)]
impl TronClient<MockClient> {
    pub fn mock(get_handler: impl Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static) -> Self {
        let mock = MockClient::new().with_get(get_handler);
        Self::new(mock.clone(), TronGridClient::new(mock, String::new()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.tron.url, gem_client::reqwest_client());
    let trongrid = settings.indexer.trongrid.remote_provider_config();
    let trongrid_client = TronGridClient::new(trongrid.configure_client(reqwest_client.clone()), trongrid.key);
    TronClient::new(reqwest_client, trongrid_client)
}
