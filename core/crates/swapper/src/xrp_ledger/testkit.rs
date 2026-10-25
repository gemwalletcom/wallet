use std::sync::{Arc, Mutex};

use gem_client::testkit::MockClient;
use gem_jsonrpc::testkit::mock_jsonrpc_client;
use gem_xrp::{method, rpc::XrpClient};
use num_bigint::BigUint;
use primitives::{Asset, asset_constants::XRP_RLUSD_TOKEN_ID};
use serde_json::{Value, json};

use crate::{Options, QuoteRequest, SwapperQuoteAsset};

pub const TEST_WALLET: &str = "rLVkkxk7THbb3dCZ6aETvLnzc8HabyubEs";
const RLUSD_CURRENCY: &str = "524C555344000000000000000000000000000000";

pub fn mock_request(from_asset: &Asset, to_asset: &Asset, value: u64) -> QuoteRequest {
    QuoteRequest {
        from_asset: SwapperQuoteAsset::from(from_asset),
        to_asset: SwapperQuoteAsset::from(to_asset),
        wallet_address: TEST_WALLET.to_string(),
        destination_address: TEST_WALLET.to_string(),
        value: BigUint::from(value),
        options: Options::default(),
    }
}

pub fn mock_xrp_client(path_find: &'static str, has_rlusd_line: bool, path_requests: Arc<Mutex<Vec<Value>>>) -> XrpClient<MockClient> {
    let rlusd_line = json!({
        "LowLimit": { "currency": RLUSD_CURRENCY, "issuer": TEST_WALLET, "value": "690000000000" },
        "HighLimit": { "currency": RLUSD_CURRENCY, "issuer": XRP_RLUSD_TOKEN_ID, "value": "0" },
        "Balance": { "currency": RLUSD_CURRENCY, "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji", "value": "0" }
    });
    let account_objects = json!({ "account_objects": if has_rlusd_line { vec![rlusd_line] } else { vec![] } });
    XrpClient::new(mock_jsonrpc_client(move |rpc_method, params| match rpc_method {
        method::RIPPLE_PATH_FIND => {
            path_requests.lock().unwrap().push(params[0].clone());
            Ok(serde_json::from_str(path_find).unwrap())
        }
        method::ACCOUNT_OBJECTS => Ok(account_objects.clone()),
        _ => panic!("unexpected method: {rpc_method}"),
    }))
}
