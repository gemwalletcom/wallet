use gem_client::{Client, ClientError, ClientExt};

use crate::PaymentError;
use crate::solana_pay::model::{TransactionRequest, TransactionRequestInfo, TransactionResponse};
use crate::solana_pay::provider::{MAX_MESSAGE_CHARS, merchant_text};

fn merchant_error(error: ClientError) -> PaymentError {
    match PaymentError::from(error) {
        PaymentError::InvalidRequest { reason } => PaymentError::invalid_request(merchant_text(&reason, MAX_MESSAGE_CHARS)),
        PaymentError::Network { reason } => PaymentError::Network {
            reason: merchant_text(&reason, MAX_MESSAGE_CHARS),
        },
        error @ (PaymentError::NoPaymentOptions | PaymentError::Status { .. }) => error,
    }
}

#[derive(Debug)]
pub(super) struct SolanaPayClient<C: Client> {
    client: C,
}

impl<C: Client> SolanaPayClient<C> {
    pub(super) fn new(client: C) -> Self {
        Self { client }
    }

    pub(super) async fn get_info(&self) -> Result<TransactionRequestInfo, PaymentError> {
        self.client.get("").await.map_err(merchant_error)
    }

    pub(super) async fn get_transaction(&self, account: &str) -> Result<TransactionResponse, PaymentError> {
        self.client.post("", &TransactionRequest { account }).await.map_err(merchant_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn maps_gateway_error_response() {
        let client = SolanaPayClient::new(MockClient::new().with_get(|_| {
            Err(ClientError::Http {
                status: 410,
                body: br#"{"error":"Payment link expired"}"#.to_vec(),
            })
        }));

        assert_eq!(client.get_info().await.unwrap_err(), PaymentError::invalid_request("Payment link expired"));
    }

    #[tokio::test]
    async fn test_merchant_error() {
        let client = SolanaPayClient::new(
            MockClient::new()
                .with_get(|_| {
                    Err(ClientError::Http {
                        status: 400,
                        body: br#"{"message":"Link\n\u202eexpired"}"#.to_vec(),
                    })
                })
                .with_post(|_, _| Err(ClientError::Network("Gateway\u{2066} down".to_string()))),
        );

        assert_eq!(client.get_info().await.unwrap_err(), PaymentError::invalid_request("Link expired"));
        assert_eq!(client.get_transaction("account").await.unwrap_err(), PaymentError::Network { reason: "Gateway down".to_string() });
    }

    #[tokio::test]
    async fn falls_back_to_transport_error() {
        let client = SolanaPayClient::new(MockClient::new().with_get(|_| {
            Err(ClientError::Http {
                status: 503,
                body: b"upstream unavailable".to_vec(),
            })
        }));

        assert_eq!(
            client.get_info().await.unwrap_err(),
            PaymentError::Network {
                reason: "Payment gateway returned HTTP 503".to_string(),
            }
        );
    }
}
