use gem_tracing::info_with_fields;
use primitives::{Chain, ValueAccess};
use prometheus_client::encoding::EncodeLabelSet;
use serde_json::{Error as JsonError, Value};
use settings_chain::BroadcastProviders;

use super::Metrics;
use super::traffic::{TrafficLabels, chain_group};
use crate::BoxError;
use crate::failure_reason::FailureReason;
use crate::proxy::ProxyResponse;
use crate::proxy::proxy_request::ProxyRequest;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct BroadcastLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    outcome: &'static str,
}

impl Metrics {
    pub(crate) fn initialize_transaction_broadcasts(&self, chains: impl IntoIterator<Item = Chain>) {
        for chain in chains {
            for outcome in ["success", "failure"] {
                let labels = self.transaction_broadcast_labels(chain, outcome);
                drop(self.transaction_broadcasts.get_or_create(&labels));
                drop(self.transaction_broadcast_latency.get_or_create(&labels));
            }
        }
    }

    fn transaction_broadcast_labels(&self, chain: Chain, outcome: &'static str) -> BroadcastLabels {
        BroadcastLabels {
            traffic: TrafficLabels::node(&self.source, chain.as_ref()),
            chain: chain.to_string(),
            outcome,
        }
    }

    pub(crate) fn record_transaction_broadcast(&self, request: &ProxyRequest, response: &Result<ProxyResponse, BoxError>, providers: &BroadcastProviders, remote_host: &str) {
        let outcome = match broadcast_result(request.chain, &request.body, response, providers) {
            Ok(transaction_id) => {
                info_with_fields!(
                    "Broadcast accepted",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    remote_host = remote_host,
                    transaction_id = transaction_id.as_str(),
                );
                "success"
            }
            Err(error) => {
                info_with_fields!(
                    "Broadcast failed",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    group = chain_group(request.chain),
                    remote_host = remote_host,
                    error = error.as_str(),
                );
                "failure"
            }
        };
        let labels = self.transaction_broadcast_labels(request.chain, outcome);
        self.transaction_broadcasts.get_or_create(&labels).inc();
        self.transaction_broadcast_latency.get_or_create(&labels).observe(request.elapsed().as_secs_f64() * 1000.0);
    }
}

fn broadcast_result(chain: Chain, request: &[u8], response: &Result<ProxyResponse, BoxError>, providers: &BroadcastProviders) -> Result<String, String> {
    match response {
        Ok(upstream) => match providers.decode_transaction_broadcast(chain, request, &upstream.body) {
            Ok(identifier) if (200..300).contains(&upstream.status) && !identifier.is_empty() => Ok(identifier),
            Ok(_) => Err(broadcast_error_message(response)),
            Err(error) if error.is::<JsonError>() => Err(broadcast_error_message(response)),
            Err(error) => Err(format_broadcast_error_message(&error.to_string())),
        },
        Err(_) => Err(broadcast_error_message(response)),
    }
}

fn broadcast_error_message(response: &Result<ProxyResponse, BoxError>) -> String {
    let message = match response {
        Err(error) => FailureReason::from_error(error.as_ref()).to_string(),
        Ok(response) => serde_json::from_slice::<Value>(&response.body)
            .ok()
            .and_then(|body| {
                body.get_value("error")
                    .and_then(|error| error.get_string("message").or_else(|_| error.string()))
                    .or_else(|_| body.get_string("message"))
                    .ok()
                    .filter(|message| !message.trim().is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| format!("Broadcast rejected or response could not be decoded (HTTP {})", response.status)),
    };
    format_broadcast_error_message(&message)
}

fn format_broadcast_error_message(message: &str) -> String {
    const MAX_ERROR_LENGTH: usize = 1024;
    message.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(MAX_ERROR_LENGTH).collect()
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use reqwest::Method;
    use reqwest::header::HeaderMap;

    use super::*;

    #[test]
    fn test_broadcast_result_hypercore() {
        let providers = BroadcastProviders::from_chains([Chain::HyperCore]);
        let request = br#"{"action":{"type":"updateLeverage"},"nonce":123}"#;
        let response = Ok(ProxyResponse::new(200, HeaderMap::new(), br#"{"status":"ok","response":{"type":"default"}}"#.to_vec()));
        assert_eq!(broadcast_result(Chain::HyperCore, request, &response, &providers), Ok("action:123".to_string()));
    }

    #[test]
    fn test_broadcast_result_requires_chain_acceptance() {
        let providers = BroadcastProviders::from_chains([Chain::Ethereum, Chain::Tron]);
        for (chain, status, body, expected) in [
            (Chain::Ethereum, 200, r#"{"jsonrpc":"2.0","id":1,"result":"0xabc"}"#, Ok("0xabc")),
            (
                Chain::Ethereum,
                200,
                r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"insufficient funds"}}"#,
                Err("insufficient funds (-32000)"),
            ),
            (
                Chain::Ethereum,
                429,
                r#"{"jsonrpc":"2.0","id":1,"result":"0xabc"}"#,
                Err("Broadcast rejected or response could not be decoded (HTTP 429)"),
            ),
            (
                Chain::Ethereum,
                200,
                "invalid response",
                Err("Broadcast rejected or response could not be decoded (HTTP 200)"),
            ),
            (
                Chain::Ethereum,
                200,
                r#"{"jsonrpc":"2.0","id":1,"result":""}"#,
                Err("Broadcast rejected or response could not be decoded (HTTP 200)"),
            ),
            (Chain::Tron, 200, r#"{"result":true,"txid":"abc"}"#, Ok("abc")),
            (
                Chain::Tron,
                200,
                r#"{"result":false,"txid":"abc","code":"SIGERROR","message":"invalid signature"}"#,
                Err("invalid signature"),
            ),
        ] {
            let response = Ok(ProxyResponse::new(status, HeaderMap::new(), body.as_bytes().to_vec()));
            assert_eq!(broadcast_result(chain, b"", &response, &providers).as_deref().map_err(String::as_str), expected);
        }
        assert_eq!(
            broadcast_result(Chain::Ethereum, b"", &Err("connection failed".into()), &providers),
            Err("request_error".into())
        );
    }

    #[test]
    fn test_bitcoin_http_error_preserves_broadcast_message() {
        let providers = BroadcastProviders::from_chains([Chain::Bitcoin]);
        for (body, message) in [
            (br#"{"error":"-26: min relay fee not met, 432 < 576"}"#.as_slice(), "-26: min relay fee not met, 432 < 576"),
            (
                br#"{"error":{"message":"transaction already in block chain"}}"#.as_slice(),
                "transaction already in block chain",
            ),
        ] {
            let response = Ok(ProxyResponse::new(400, HeaderMap::new(), body.to_vec()));
            assert_eq!(broadcast_result(Chain::Bitcoin, b"", &response, &providers), Err(message.to_string()));
        }
    }

    #[test]
    fn test_broadcast_result_preserves_chain_errors() {
        let cases = [
            (
                Chain::Solana,
                br#"{"id":1,"error":{"code":-32002,"message":"Transaction simulation failed"}}"#.as_slice(),
                "Transaction simulation failed (-32002)",
            ),
            (
                Chain::Cosmos,
                br#"{"tx_response":{"txhash":"abc","code":4,"raw_log":"signature verification failed"}}"#.as_slice(),
                "signature verification failed",
            ),
            (
                Chain::Tron,
                include_bytes!("../../../../crates/gem_tron/testdata/transaction_broadcast_error.json").as_slice(),
                "Contract validate error : Cannot transfer TRX to yourself.",
            ),
            (Chain::Ton, br#"{"message":"invalid BOC"}"#.as_slice(), "invalid BOC"),
            (Chain::Sui, b"\x00\x00\x00\x00\x00".as_slice(), "missing Sui broadcast transaction digest"),
            (
                Chain::Xrp,
                br#"{"id":1,"result":{"accepted":false,"engine_result_message":"Insufficient XRP balance"}}"#.as_slice(),
                "Transaction rejected: Insufficient XRP balance",
            ),
            (
                Chain::Near,
                br#"{"id":1,"error":{"code":-32000,"message":"Invalid nonce"}}"#.as_slice(),
                "Invalid nonce (-32000)",
            ),
            (Chain::Aptos, br#"{"message":"SEQUENCE_NUMBER_TOO_OLD"}"#.as_slice(), "SEQUENCE_NUMBER_TOO_OLD"),
            (
                Chain::Stellar,
                br#"{"tx_status":"ERROR","title":"Transaction Failed"}"#.as_slice(),
                "Broadcast error: Transaction Failed",
            ),
            (
                Chain::Algorand,
                include_bytes!("../../../../crates/gem_algorand/testdata/transaction_broadcast_error.json").as_slice(),
                "txgroup had 0 in fees, which is less than the minimum 1 * 1000",
            ),
            (Chain::Cardano, br#"{"errors":[{"message":"BadInputsUTxO"}]}"#.as_slice(), "Failed to broadcast transaction"),
            (
                Chain::Polkadot,
                br#"{"error":"Invalid Transaction","cause":"Stale"}"#.as_slice(),
                "Invalid Transaction: Stale",
            ),
            (
                Chain::HyperCore,
                include_bytes!("../../../../crates/gem_hypercore/testdata/order_broadcast_error.json").as_slice(),
                "Reduce only order would increase position. asset=159",
            ),
        ];
        let providers = BroadcastProviders::from_chains(cases.iter().map(|(chain, _, _)| *chain));
        for (chain, body, message) in cases {
            let response = Ok(ProxyResponse::new(400, HeaderMap::new(), body.to_vec()));
            assert_eq!(broadcast_result(chain, b"", &response, &providers), Err(message.to_string()), "{chain}");
        }
    }

    #[test]
    fn test_broadcast_result_preserves_sui_response_protocols() {
        let providers = BroadcastProviders::from_chains([Chain::Sui]);
        for body in [br#"{"digest":"abc"}"#.as_slice(), b"\x00\x00\x00\x00\x07\x0a\x05\x0a\x03abc".as_slice()] {
            let response = Ok(ProxyResponse::new(200, HeaderMap::new(), body.to_vec()));
            assert_eq!(broadcast_result(Chain::Sui, b"", &response, &providers), Ok("abc".to_string()));
        }
    }

    #[test]
    fn test_broadcast_error_message_excludes_response_data_and_transport_details() {
        let response = Ok(ProxyResponse::new(
            200,
            HeaderMap::new(),
            br#"{"error":{"message":"insufficient\nfunds","data":"signed-payload"},"id":1}"#.to_vec(),
        ));
        assert_eq!(broadcast_error_message(&response), "insufficient funds");
        let response = Ok(ProxyResponse::new(503, HeaderMap::new(), b"private-response-body".to_vec()));
        assert_eq!(broadcast_error_message(&response), "Broadcast rejected or response could not be decoded (HTTP 503)");
        assert_eq!(broadcast_error_message(&Err("https://node.example/secret-key".into())), "request_error");
    }

    #[test]
    fn test_broadcast_metrics_exclude_transaction_data() {
        let metrics = Metrics::mock();
        let providers = BroadcastProviders::from_chains([Chain::Ethereum]);
        let request = ProxyRequest::mock(Chain::Ethereum, Method::POST, "/", &[]);
        let response = Ok(ProxyResponse::new(
            200,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","id":1,"result":"private-identifier"}"#.to_vec(),
        ));
        metrics.record_transaction_broadcast(&request, &response, &providers, "rpc.example.com");
        let encoded = metrics.get_metrics();
        assert_eq!(encoded.find("private-identifier"), None);
        for name in ["dynode_transaction_broadcasts_total", "dynode_transaction_broadcast_latency_milliseconds_count"] {
            assert_eq!(
                encoded.lines().filter(|line| line.starts_with(&format!("{name}{{"))).collect::<Vec<_>>(),
                vec![format!(
                    "{name}{{source=\"public\",group=\"evm\",service=\"ethereum\",chain=\"ethereum\",outcome=\"success\"}} 1"
                )]
            );
        }
    }
}
