use chain_traits::ChainRequestClassifier;
use primitives::{ChainRequest, ChainRequestType};

use crate::provider::BroadcastProvider;
use crate::rpc::client::PATH_EXECUTE_TRANSACTION;

impl ChainRequestClassifier for BroadcastProvider {
    fn classify_request(&self, request: ChainRequest<'_>) -> ChainRequestType {
        if request.is_http_post_path(PATH_EXECUTE_TRANSACTION) {
            ChainRequestType::Broadcast
        } else {
            ChainRequestType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::ChainRequestProtocol;

    #[test]
    fn test_classify_request() {
        let provider = BroadcastProvider;

        let broadcast = ChainRequest::new(ChainRequestProtocol::Http, "POST", PATH_EXECUTE_TRANSACTION, &[]);
        assert_eq!(provider.classify_request(broadcast), ChainRequestType::Broadcast);

        let wrong_method = ChainRequest::new(ChainRequestProtocol::Http, "GET", PATH_EXECUTE_TRANSACTION, &[]);
        assert_eq!(provider.classify_request(wrong_method), ChainRequestType::Unknown);

        let simulation = ChainRequest::new(ChainRequestProtocol::Http, "POST", "/sui.rpc.v2.TransactionExecutionService/SimulateTransaction", &[]);
        assert_eq!(provider.classify_request(simulation), ChainRequestType::Unknown);
    }
}
