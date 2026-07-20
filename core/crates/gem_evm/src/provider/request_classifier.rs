use chain_traits::ChainRequestClassifier;
use primitives::{ChainRequest, ChainRequestType};

use crate::method;
use crate::provider::BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {
    fn classify_request(&self, request: ChainRequest<'_>) -> ChainRequestType {
        if request.is_json_rpc_method(method::ETH_SEND_RAW_TRANSACTION) {
            ChainRequestType::Broadcast
        } else {
            ChainRequestType::Unknown
        }
    }
}
