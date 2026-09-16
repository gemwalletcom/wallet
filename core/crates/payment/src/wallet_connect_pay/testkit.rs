use gem_evm::encode::encode_erc20_approve_max_value;
use num_bigint::BigUint;
use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
use primitives::hex::encode_with_0x;
use primitives::testkit::signer_mock::TEST_EVM_SENDER;
use primitives::{AssetId, ChainAddress, WalletConnectCAIP2, WalletConnectionMethods, serde_name};
use serde_json::Value;

pub const PERMIT_TRANSFER_FROM: &str = include_str!("../../testdata/wallet_connect_pay/permit_transfer_from.json");
pub const TRANSFER_WITH_AUTHORIZATION: &str = include_str!("../../testdata/wallet_connect_pay/transfer_with_authorization.json");
pub const OPTIONS_IDENTITY_REQUIRED: &str = include_str!("../../testdata/wallet_connect_pay/options_identity_required.json");
pub const STATUS_SUCCEEDED: &str = include_str!("../../testdata/wallet_connect_pay/status_succeeded.json");
pub const STATUS_PROCESSING: &str = include_str!("../../testdata/wallet_connect_pay/status_processing.json");
pub const STATUS_SUCCEEDED_WITHOUT_INFO: &str = include_str!("../../testdata/wallet_connect_pay/status_succeeded_without_info.json");

use crate::wallet_connect_pay::model::{Quote, WalletRpcAction};

impl Quote {
    pub fn mock(asset_id: AssetId, value: u64) -> Self {
        Self {
            id: "opt_1".to_string(),
            account: ChainAddress::new(asset_id.chain, TEST_EVM_SENDER.to_string()),
            asset_id,
            value: BigUint::from(value),
            collect_data_url: None,
            actions: Vec::new(),
        }
    }
}

impl WalletRpcAction {
    pub fn mock(method: WalletConnectionMethods, chain_id: &str, params: Value) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            method: serde_name(&method).unwrap(),
            params,
        }
    }

    pub fn mock_send(quote: &Quote, to: &str, value: &str, data: &str) -> Self {
        Self::mock(
            WalletConnectionMethods::EthSendTransaction,
            &chain_id(quote),
            serde_json::json!([{"from": quote.account.address, "to": to, "value": value, "data": data}]),
        )
    }

    pub fn mock_approve(quote: &Quote) -> Self {
        let token = quote.token().to_lowercase();
        let data = encode_with_0x(&encode_erc20_approve_max_value(UNISWAP_PERMIT2_CONTRACT).unwrap());
        Self::mock(
            WalletConnectionMethods::EthSendTransaction,
            &chain_id(quote),
            serde_json::json!([{"from": quote.account.address, "to": token, "value": "0x0", "data": data}]),
        )
    }

    pub fn mock_permit(quote: &Quote, amount: &str) -> Self {
        Self::mock(
            WalletConnectionMethods::EthSignTypedDataV4,
            &chain_id(quote),
            serde_json::json!([quote.account.address, mock_permit_transfer_from(amount).to_string()]),
        )
    }
}

pub fn mock_permit_transfer_from(amount: &str) -> Value {
    let mut typed_data: Value = serde_json::from_str(PERMIT_TRANSFER_FROM).unwrap();
    typed_data["message"]["permitted"]["amount"] = Value::String(amount.to_string());
    typed_data
}

fn chain_id(quote: &Quote) -> String {
    format!(
        "{}:{}",
        WalletConnectCAIP2::get_namespace(quote.account.chain).unwrap(),
        WalletConnectCAIP2::get_reference(quote.account.chain).unwrap()
    )
}
