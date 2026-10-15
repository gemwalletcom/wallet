use super::broker::{AssetsResponse, ChainflipAsset, DepositChannelRequest, QuoteRequest, RefundParameters};
use crate::fees::DEFAULT_CHAINFLIP_FEE_BPS;
use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_POLKADOT_SENDER};

impl AssetsResponse {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("./broker/test/assets.json")).unwrap()
    }
}

impl QuoteRequest {
    pub fn mock(amount: &str, source_asset: &str, destination_asset: &str) -> Self {
        Self {
            amount: amount.parse().unwrap(),
            source_asset: source_asset.to_string(),
            destination_asset: destination_asset.to_string(),
            commission_bps: DEFAULT_CHAINFLIP_FEE_BPS,
            is_vault_swap: true,
        }
    }
}

impl DepositChannelRequest {
    pub fn mock() -> Self {
        Self {
            source_asset: ChainflipAsset {
                chain: "Assethub".to_string(),
                asset: "DOT".to_string(),
            },
            destination_asset: ChainflipAsset {
                chain: "Ethereum".to_string(),
                asset: "ETH".to_string(),
            },
            destination_address: TEST_EVM_RECIPIENT.to_string(),
            boost_fee: None,
            refund_parameters: RefundParameters {
                retry_duration: 150,
                refund_address: TEST_POLKADOT_SENDER.to_string(),
                min_price: "0x1".to_string(),
                max_oracle_price_slippage: None,
            },
            dca_parameters: None,
        }
    }
}
