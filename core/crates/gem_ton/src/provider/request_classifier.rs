use chain_traits::ChainRequestClassifier;
use primitives::{ChainRequest, ChainRequestType};

use crate::provider::BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {
    fn classify_request(&self, request: ChainRequest<'_>) -> ChainRequestType {
        // TODO(2027-01-01): Remove v2 classification with Dynode legacy wallet routes.
        if request.is_http_post_path("/api/v3/message") || request.is_http_post_path("/api/v2/sendBocReturnHash") {
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
    fn test_classify_broadcast() {
        for (method, path, expected) in [
            ("POST", "/api/v3/message", ChainRequestType::Broadcast),
            ("GET", "/api/v3/message", ChainRequestType::Unknown),
            ("POST", "/api/v2/sendBocReturnHash", ChainRequestType::Broadcast),
            ("GET", "/api/v2/sendBocReturnHash", ChainRequestType::Unknown),
        ] {
            assert_eq!(BroadcastProvider.classify_request(ChainRequest::new(ChainRequestProtocol::Http, method, path, b"{}")), expected);
        }
    }
}
