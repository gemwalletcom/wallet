use gem_client::{Client, ClientExt};

use crate::PaymentError;
use crate::solana_pay::model::{TransactionRequest, TransactionRequestInfo, TransactionResponse};

#[derive(Debug)]
pub(super) struct SolanaPayClient<C: Client> {
    client: C,
}

impl<C: Client> SolanaPayClient<C> {
    pub(super) fn new(client: C) -> Self {
        Self { client }
    }

    pub(super) async fn get_info(&self) -> Result<TransactionRequestInfo, PaymentError> {
        Ok(self.client.get("").await?)
    }

    pub(super) async fn get_transaction(&self, account: &str) -> Result<TransactionResponse, PaymentError> {
        Ok(self.client.post("", &TransactionRequest { account }).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::{ClientError, testkit::MockClient};

    #[tokio::test]
    async fn maps_gateway_error_response() {
        let client = SolanaPayClient::new(MockClient::new().with_get(|_| {
            Err(ClientError::Http {
                status: 410,
                body: br#"{"error":"Payment link expired"}"#.to_vec(),
            })
        }));

        assert_eq!(
            client.get_info().await.unwrap_err(),
            PaymentError::InvalidRequest {
                reason: "Payment link expired".to_string(),
            }
        );
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
