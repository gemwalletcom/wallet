use primitives::testkit::json::load_json;
use primitives::{Chain, ChainAddress, PaymentStatus};

use crate::wallet_connect_pay::config::WalletConnectPayAuth;
use crate::wallet_connect_pay::model::{PaymentOption, PaymentOptionsResponse, PaymentStatusResponse};

pub use primitives::testkit::payment_mock::TEST_PAYMENT_ID;
pub use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;

pub const TEST_ADDRESS: &str = "0x1085c5f70F7F7591D97da281A64688385455c2bD";
pub const TEST_ACCOUNT_ETHEREUM: &str = "eip155:1:0x1085c5f70F7F7591D97da281A64688385455c2bD";
pub const TEST_ACCOUNT_POLYGON: &str = "eip155:137:0x1085c5f70F7F7591D97da281A64688385455c2bD";
pub const TEST_APP_ID: &str = "app_1";
pub const TEST_CLIENT_ID: &str = "client_1";

pub const OPTIONS_RESPONSE_COIN: &str = include_str!("../../testdata/options_response_coin.json");
pub const OPTIONS_RESPONSE_NATIVE: &str = include_str!("../../testdata/options_response_native.json");
pub const FETCH_RESPONSE_NATIVE: &str = include_str!("../../testdata/fetch_response_native.json");
pub const FETCH_RESPONSE_PERMIT2: &str = include_str!("../../testdata/fetch_response_permit2.json");

impl WalletConnectPayAuth {
    pub fn mock() -> Self {
        Self {
            app_id: TEST_APP_ID.to_string(),
            client_id: TEST_CLIENT_ID.to_string(),
        }
    }
}

impl PaymentOptionsResponse {
    pub fn mock() -> Self {
        load_json(OPTIONS_RESPONSE_COIN)
    }

    pub fn mock_with_status(status: PaymentStatus) -> Self {
        let mut response = Self::mock();
        response.info.as_mut().unwrap().status = status;
        response
    }

    pub fn mock_without_info() -> Self {
        Self {
            info: None,
            options: None,
            result_info: None,
        }
    }
}

impl PaymentOption {
    pub fn mock_collect_data() -> Self {
        load_json(include_str!("../../testdata/option_collect_data.json"))
    }

    pub fn mock_collect_data_from(url: &str) -> Self {
        let mut option = Self::mock_collect_data();
        option.collect_data.as_mut().unwrap().url = url.to_string();
        option
    }
}

impl PaymentStatusResponse {
    pub fn mock_succeeded() -> Self {
        load_json(include_str!("../../testdata/status_response_succeeded.json"))
    }
}

pub fn mock_accounts() -> Vec<String> {
    vec![TEST_ACCOUNT_ETHEREUM.to_string()]
}

pub fn mock_addresses() -> Vec<ChainAddress> {
    vec![
        ChainAddress::new(Chain::Ethereum, TEST_ADDRESS.to_string()),
        ChainAddress::new(Chain::Bitcoin, "bc1".to_string()),
    ]
}
