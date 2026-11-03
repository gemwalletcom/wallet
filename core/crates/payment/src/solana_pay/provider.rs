use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, ChainAddress, OptionStringExt, PaymentInvoice, PaymentLink, PaymentMerchant, TransferDataOutputType};

use crate::provider::PaymentProvider;
use crate::solana_pay::client::SolanaPayClient;
use crate::solana_pay::transaction::prepare;
use crate::{PaymentError, PaymentLoad, PaymentTransaction};

const MAX_MERCHANT_CHARS: usize = 64;
const MAX_MESSAGE_CHARS: usize = 280;

fn is_direction_character(character: char) -> bool {
    matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// Merchant text shows above the warnings on Confirm, so it stays short, on one line and in reading order.
fn merchant_text(text: &str, max_chars: usize) -> String {
    let readable: String = text.chars().filter(|character| character.is_whitespace() || !(character.is_control() || is_direction_character(*character))).collect();
    let line = readable.split_whitespace().collect::<Vec<_>>().join(" ");
    match line.chars().nth(max_chars) {
        Some(_) => format!("{}…", line.chars().take(max_chars).collect::<String>()),
        None => line,
    }
}

#[derive(Debug)]
pub(crate) struct SolanaPayProvider<C: Client> {
    client: SolanaPayClient<C>,
    url: String,
}

impl<C: Client> SolanaPayProvider<C> {
    pub(crate) fn new(client: C, url: String) -> Self {
        Self { client: SolanaPayClient::new(client), url }
    }
}

#[async_trait]
impl<C: Client> PaymentProvider for SolanaPayProvider<C> {
    fn supported_chains(&self) -> &'static [Chain] {
        &[Chain::Solana]
    }

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        let account = addresses.iter().find(|address| address.chain == Chain::Solana).cloned().ok_or(PaymentError::NoPaymentOptions)?;
        let (info, response) = futures::try_join!(self.client.get_info(), self.client.get_transaction(&account.address))?;
        let prepared = prepare(&response.transaction, &account.address).map_err(PaymentError::invalid_request)?;

        Ok(PaymentLoad::Sign {
            transaction: PaymentTransaction {
                invoice: PaymentInvoice {
                    link: PaymentLink::SolanaPay { url: self.url.clone() },
                    merchant: PaymentMerchant {
                        name: merchant_text(&info.label, MAX_MERCHANT_CHARS),
                        icon: info.icon,
                    },
                    price: None,
                    message: response.message.map(|message| merchant_text(&message, MAX_MESSAGE_CHARS)).non_empty(),
                    quotes: vec![],
                    verification: None,
                },
                account,
                transaction: prepared.transaction,
                transaction_type: prepared.transaction_type,
                memo: prepared.memo,
                request: prepared.request,
                output_type: TransferDataOutputType::EncodedTransaction,
                approval: None,
            },
        })
    }

    async fn confirm(&self, _quote_id: &str, _action_results: Vec<String>) -> Result<(), PaymentError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    const ACCOUNT: &str = "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN";
    const TRANSACTION: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAECC4JMKqNplIXybGb/GhK1ofdVWeuEjXnQor7gi0Y2hMcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQECAAAMAgAAAAAAAAAAAAAA";

    fn provider(response: String) -> SolanaPayProvider<MockClient> {
        SolanaPayProvider::new(
            MockClient::new()
                .with_get(|path| {
                    assert_eq!(path, "");
                    Ok(br#"{"label":"Constant K","icon":"https://constant-k.com/icon.png"}"#.to_vec())
                })
                .with_post(move |path, body| {
                    assert_eq!(path, "");
                    assert_eq!(serde_json::from_slice::<serde_json::Value>(body).unwrap(), serde_json::json!({ "account": ACCOUNT }));
                    Ok(response.clone().into_bytes())
                }),
            "https://constant-k.com/checkout".to_string(),
        )
    }

    async fn message(response: String) -> Option<String> {
        let PaymentLoad::Sign { transaction } = provider(response).load(&[ChainAddress::new(Chain::Solana, ACCOUNT.to_string())]).await.unwrap() else {
            panic!("a transaction request is signed");
        };
        transaction.invoice.message
    }

    #[tokio::test]
    async fn test_transaction_request() {
        let provider = provider(format!(r#"{{"message":"Annual plan","transaction":"{TRANSACTION}"}}"#));
        let prepared = prepare(TRANSACTION, ACCOUNT).unwrap();

        assert_eq!(
            provider.load(&[ChainAddress::new(Chain::Solana, ACCOUNT.to_string())]).await.unwrap(),
            PaymentLoad::Sign {
                transaction: PaymentTransaction {
                    invoice: PaymentInvoice {
                        link: PaymentLink::SolanaPay {
                            url: "https://constant-k.com/checkout".to_string()
                        },
                        merchant: PaymentMerchant {
                            name: "Constant K".to_string(),
                            icon: "https://constant-k.com/icon.png".to_string(),
                        },
                        price: None,
                        message: Some("Annual plan".to_string()),
                        quotes: vec![],
                        verification: None,
                    },
                    account: ChainAddress::new(Chain::Solana, ACCOUNT.to_string()),
                    transaction: prepared.transaction,
                    transaction_type: prepared.transaction_type,
                    memo: prepared.memo,
                    request: prepared.request,
                    output_type: TransferDataOutputType::EncodedTransaction,
                    approval: None,
                },
            }
        );
    }

    #[tokio::test]
    async fn test_merchant_text_reads_as_it_will_show() {
        assert_eq!(
            message(format!(r#"{{"message":"Annual\n\n\nplan\u202e\u0000 for \u2066you\u2069","transaction":"{TRANSACTION}"}}"#)).await,
            Some("Annual plan for you".to_string()),
            "line breaks fold to a space, and direction and control characters are gone"
        );
        assert_eq!(message(format!(r#"{{"message":"\u202e\n","transaction":"{TRANSACTION}"}}"#)).await, None, "nothing readable is no message");

        let shown = message(format!(r#"{{"message":"{}","transaction":"{TRANSACTION}"}}"#, "é".repeat(MAX_MESSAGE_CHARS + 1))).await.unwrap();
        assert_eq!(shown, format!("{}…", "é".repeat(MAX_MESSAGE_CHARS)));

        let exact = "é".repeat(MAX_MESSAGE_CHARS);
        assert_eq!(message(format!(r#"{{"message":"{exact}","transaction":"{TRANSACTION}"}}"#)).await, Some(exact));
    }

    #[test]
    fn test_merchant_name_drops_direction_characters() {
        assert_eq!(merchant_text("Constant\u{202e} K", MAX_MERCHANT_CHARS), "Constant K");
        assert_eq!(merchant_text(&"k".repeat(MAX_MERCHANT_CHARS + 1), MAX_MERCHANT_CHARS), format!("{}…", "k".repeat(MAX_MERCHANT_CHARS)));
    }

    #[tokio::test]
    async fn test_transaction_request_without_message() {
        assert_eq!(message(format!(r#"{{"transaction":"{TRANSACTION}"}}"#)).await, None);
        assert_eq!(message(format!(r#"{{"message":"","transaction":"{TRANSACTION}"}}"#)).await, None, "an empty message is no message");
    }
}
