use primitives::{AssetId, Chain, ChainAddress, WalletConnectCAIP2};

use crate::wallet_connect_pay::model::{FetchActionsResponse, Invoice, Options, Quote, WalletRpcAction};
use crate::wallet_connect_pay::payment_mapper::map_options;

pub const OPTIONS: &str = include_str!("../../testdata/wallet_connect_pay/options.json");
pub const OPTIONS_WITHOUT_ALLOWANCE: &str = include_str!("../../testdata/wallet_connect_pay/options_without_allowance.json");
pub const OPTIONS_IDENTITY_REQUIRED: &str = include_str!("../../testdata/wallet_connect_pay/options_identity_required.json");
pub const OPTIONS_FAILED: &str = include_str!("../../testdata/wallet_connect_pay/options_failed.json");
pub const FETCH_SEND: &str = include_str!("../../testdata/wallet_connect_pay/fetch_send.json");
pub const FETCH_RECEIVE_WITH_AUTHORIZATION: &str = include_str!("../../testdata/wallet_connect_pay/fetch_receive_with_authorization.json");
pub const FETCH_IDENTITY_REQUIRED: &str = include_str!("../../testdata/wallet_connect_pay/fetch_identity_required.json");
pub const STATUS_REQUIRES_ACTION: &str = include_str!("../../testdata/wallet_connect_pay/status_requires_action.json");
pub const STATUS_PROCESSING: &str = include_str!("../../testdata/wallet_connect_pay/status_processing.json");
pub const STATUS_SUCCEEDED: &str = include_str!("../../testdata/wallet_connect_pay/status_succeeded.json");
pub const STATUS_SUCCEEDED_COIN: &str = include_str!("../../testdata/wallet_connect_pay/status_succeeded_coin.json");
pub const STATUS_FAILED: &str = include_str!("../../testdata/wallet_connect_pay/status_failed.json");
pub const STATUS_EXPIRED: &str = include_str!("../../testdata/wallet_connect_pay/status_expired.json");
pub const PERMIT_TRANSFER_FROM: &str = include_str!("../../testdata/wallet_connect_pay/permit_transfer_from.json");
pub const RECEIVE_WITH_AUTHORIZATION: &str = include_str!("../../testdata/wallet_connect_pay/receive_with_authorization.json");

pub const TEST_ACCOUNT: &str = "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB";
pub const TEST_ACCOUNT_WITHOUT_ALLOWANCE: &str = "0xF977814e90dA44bFA03b6295A0616a897441aceC";
pub const TEST_PAYMENT_ID: &str = "pay_6fa2ecc101M2NV05JCTQGE0FXR387X4TJB";
pub const TEST_ROUTER: &str = "0x57b2b4288220005234c0e88a04a7943193971d21";
pub const TEST_PERMIT_SPENDER: &str = "0x5fae5310F257437fBfed0dA3c417DA410078058E";
pub const TEST_AUTHORIZATION_RECIPIENT: &str = "0x35e6Cb35647076539F8c4d14F51e03841bCd1f0e";
pub const PYUSD_TOKEN_ID: &str = "0x6c3ea9036406852006290770BEdFcAbA0e23A0e8";

const CHAINS: [Chain; 6] = [Chain::Ethereum, Chain::Optimism, Chain::SmartChain, Chain::Polygon, Chain::Base, Chain::Arbitrum];

pub fn addresses(address: &str) -> Vec<ChainAddress> {
    CHAINS.iter().map(|chain| ChainAddress::new(*chain, address.to_string())).collect()
}

pub fn accounts(address: &str) -> Vec<String> {
    CHAINS.iter().filter_map(|chain| WalletConnectCAIP2::format_account(*chain, address)).collect()
}

pub fn invoice(options: &str, address: &str) -> Invoice {
    match map_options(serde_json::from_str(options).unwrap(), &accounts(address)).unwrap() {
        Options::Invoice(invoice) => invoice,
        Options::Status { status } => panic!("payment is {status:?}"),
    }
}

pub fn quotes(options: &str, address: &str) -> Vec<Quote> {
    invoice(options, address).quotes
}

pub fn quote(options: &str, address: &str, asset_id: &AssetId) -> Quote {
    quotes(options, address).into_iter().find(|quote| &quote.asset_id == asset_id).unwrap()
}

pub fn quote_actions(quote: &Quote) -> Vec<WalletRpcAction> {
    quote.actions.clone().into_iter().map(|action| WalletRpcAction::try_from(action).unwrap()).collect()
}

pub fn fetch_actions(fetch: &str) -> Vec<WalletRpcAction> {
    let response: FetchActionsResponse = serde_json::from_str(fetch).unwrap();
    response.actions.into_iter().map(|action| WalletRpcAction::try_from(action).unwrap()).collect()
}
