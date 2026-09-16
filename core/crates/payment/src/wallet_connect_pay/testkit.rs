use gem_evm::encode::encode_erc20_approve_max_value;
use num_bigint::BigUint;
use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
use primitives::hex::encode_with_0x;
use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};
use primitives::{AssetId, ChainAddress, WalletConnectCAIP2, WalletConnectionMethods, serde_name};
use serde_json::Value;

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
        let token = quote.asset_id.token_id.clone().unwrap_or_default().to_lowercase();
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
            serde_json::json!([quote.account.address, mock_permit_transfer_from(quote, amount).to_string()]),
        )
    }
}

pub fn mock_permit_transfer_from(quote: &Quote, amount: &str) -> Value {
    serde_json::json!({
        "domain": {"name": "Permit2", "chainId": quote.asset_id.chain.network_id(), "verifyingContract": UNISWAP_PERMIT2_CONTRACT},
        "types": {
            "PermitTransferFrom": [{"name": "permitted", "type": "TokenPermissions"}, {"name": "spender", "type": "address"}, {"name": "nonce", "type": "uint256"}, {"name": "deadline", "type": "uint256"}],
            "TokenPermissions": [{"name": "token", "type": "address"}, {"name": "amount", "type": "uint256"}]
        },
        "primaryType": "PermitTransferFrom",
        "message": {"permitted": {"token": quote.asset_id.token_id, "amount": amount}, "spender": TEST_EVM_RECIPIENT, "nonce": "0x08", "deadline": "1785175272"}
    })
}

fn chain_id(quote: &Quote) -> String {
    format!(
        "{}:{}",
        WalletConnectCAIP2::get_namespace(quote.account.chain).unwrap(),
        WalletConnectCAIP2::get_reference(quote.account.chain).unwrap()
    )
}
