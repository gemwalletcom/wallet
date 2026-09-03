use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

use super::model::{ChainflipAsset, DcaParameters, VaultSwapExtras};

const REQUEST_SWAP_PARAMETER_ENCODING: &str = "broker_request_swap_parameter_encoding";

#[derive(Clone, Debug)]
pub(super) struct RequestSwapParameterEncoding {
    pub source_asset: ChainflipAsset,
    pub destination_asset: ChainflipAsset,
    pub destination_address: String,
    pub broker_commission: u32,
    pub extra_params: VaultSwapExtras,
    pub boost_fee: Option<u32>,
    pub dca_params: Option<DcaParameters>,
}

impl ToJsonRpcRequest for RequestSwapParameterEncoding {
    fn method(&self) -> &'static str {
        REQUEST_SWAP_PARAMETER_ENCODING
    }

    fn params(&self) -> Value {
        json!([
            self.source_asset,
            self.destination_asset,
            self.destination_address,
            self.broker_commission,
            self.extra_params,
            Value::Null,
            self.boost_fee,
            Vec::<Value>::new(),
            self.dca_params,
        ])
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use super::*;
    use crate::chainflip::broker::model::{RefundParameters, VaultSwapChainExtras};

    #[test]
    fn test_params() {
        let extras = VaultSwapExtras::Evm(VaultSwapChainExtras {
            chain: "Ethereum".into(),
            input_amount: BigUint::from(1000u32),
            refund_parameters: RefundParameters::default(),
        });
        let request = RequestSwapParameterEncoding {
            source_asset: ChainflipAsset {
                chain: "Ethereum".into(),
                asset: "ETH".into(),
            },
            destination_asset: ChainflipAsset {
                chain: "Solana".into(),
                asset: "SOL".into(),
            },
            destination_address: "destination".into(),
            broker_commission: 10,
            extra_params: extras.clone(),
            boost_fee: Some(5),
            dca_params: None,
        }
        .to_jsonrpc_request(7);

        assert_eq!(request.id, 7);
        assert_eq!(request.method, REQUEST_SWAP_PARAMETER_ENCODING);
        assert_eq!(
            request.params,
            json!([
                {"chain": "Ethereum", "asset": "ETH"},
                {"chain": "Solana", "asset": "SOL"},
                "destination",
                10,
                serde_json::to_value(&extras).unwrap(),
                null,
                5,
                [],
                null
            ])
        );
    }
}
