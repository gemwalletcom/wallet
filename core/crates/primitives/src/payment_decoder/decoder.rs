use super::error::{PaymentDecoderError, Result};
use crate::payment::Payment;

use super::{bip21, erc681, solana_pay, ton_pay};

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment> {
        let uri = string.trim();
        let payment = Self::decode_uri(uri.split_once('#').map_or(uri, |(uri, _)| uri))?;

        if matches!(&payment, Payment::Request(request) if request.address.is_empty()) {
            return Err(PaymentDecoderError::MissingField("address".to_string()));
        }
        Ok(payment)
    }

    fn decode_uri(uri: &str) -> Result<Payment> {
        let Some((scheme, path)) = uri.split_once(':') else {
            return bip21::decode(None, uri);
        };
        let scheme = scheme.to_ascii_lowercase();

        match scheme.as_str() {
            erc681::ETHEREUM_SCHEME => erc681::decode(path),
            solana_pay::SOLANA_PAY_SCHEME => solana_pay::decode(path),
            ton_pay::TON_PAY_SCHEME => ton_pay::decode(path),
            _ => bip21::decode(Some(&scheme), path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetId, Chain,
        payment::{PaymentAmount, PaymentRequest},
    };

    const BITCOIN_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

    #[test]
    fn test_address() {
        assert_eq!(
            PaymentURLDecoder::decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").unwrap(),
            Payment::Request(PaymentRequest::new_address("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"))
        );

        assert_eq!(
            PaymentURLDecoder::decode("0x25851Bf7D35293A89F710eBFbD4718322eF7B174?amount=50.72").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0x25851Bf7D35293A89F710eBFbD4718322eF7B174".to_string(),
                amount: Some(PaymentAmount::ExactValue("50.72".to_string())),
                ..PaymentRequest::mock()
            })
        );
    }

    #[test]
    fn test_uri_normalization() {
        let bitcoin = Payment::Request(PaymentRequest {
            address: BITCOIN_ADDRESS.to_string(),
            amount: Some(PaymentAmount::ExactValue("0.1".to_string())),
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..PaymentRequest::mock()
        });
        let address_only = Payment::Request(PaymentRequest {
            address: BITCOIN_ADDRESS.to_string(),
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..PaymentRequest::mock()
        });
        let uri = format!("bitcoin:{BITCOIN_ADDRESS}?amount=0.1");

        assert_eq!(PaymentURLDecoder::decode(&uri).unwrap(), bitcoin);
        assert_eq!(PaymentURLDecoder::decode(&format!("  {uri}\n")).unwrap(), bitcoin);
        assert_eq!(PaymentURLDecoder::decode(&format!("{uri}#note")).unwrap(), bitcoin);
        assert_eq!(PaymentURLDecoder::decode(&format!("BITCOIN:{BITCOIN_ADDRESS}?amount=0.1")).unwrap(), bitcoin);

        assert_eq!(PaymentURLDecoder::decode(&format!("bitcoin:{BITCOIN_ADDRESS}?AMOUNT=0.1")).unwrap(), address_only);
        assert_eq!(PaymentURLDecoder::decode(&format!("bitcoin:{BITCOIN_ADDRESS}?amount=&memo=")).unwrap(), address_only);
        assert_eq!(PaymentURLDecoder::decode(&format!("bitcoin:{BITCOIN_ADDRESS}")).unwrap(), address_only);
        assert_eq!(PaymentURLDecoder::decode(&format!("bitcoin://{BITCOIN_ADDRESS}")).unwrap(), address_only);
        assert_eq!(
            PaymentURLDecoder::decode("algorand://TIQ4WPFJQYLA5PBQFQZLLBKMDNQFGZDLTKLGPKCUOJPLQZQPQFQZLLBKMD").unwrap(),
            Payment::Request(PaymentRequest {
                address: "TIQ4WPFJQYLA5PBQFQZLLBKMDNQFGZDLTKLGPKCUOJPLQZQPQFQZLLBKMD".to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Algorand)),
                ..PaymentRequest::mock()
            })
        );

        assert_eq!(
            PaymentURLDecoder::decode(&format!("bitcoin:{BITCOIN_ADDRESS}?memo=see%20http%3A%2F%2Fx.com")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                memo: Some("see http://x.com".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode(&format!("bitcoin:{BITCOIN_ADDRESS}?memo=YWJjZA==")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                memo: Some("YWJjZA==".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                ..PaymentRequest::mock()
            })
        );
    }

    #[test]
    fn test_refuses_what_it_cannot_sign() {
        assert!(PaymentURLDecoder::decode("wc:abc123@2?relay-protocol=irn&symKey=deadbeef").is_err());
        assert!(PaymentURLDecoder::decode("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_123").is_err());
        assert!(PaymentURLDecoder::decode("https://pay.walletconnect.com/?pid=pay_123").is_err());
        assert!(PaymentURLDecoder::decode("https://gemwallet.com/tokens/bitcoin").is_err());
        assert!(PaymentURLDecoder::decode("gem://wc?sessionTopic=abc").is_err());
        assert!(PaymentURLDecoder::decode("lightning:lnbc1pvjluezpp5qqqsyq").is_err());
        assert!(PaymentURLDecoder::decode("eip155:1:0xcB3028d6120802148f03d6c884D6AD6A210Df62A").is_err());
        assert!(PaymentURLDecoder::decode("web+stellar:pay?destination=GABC").is_err());
        assert!(PaymentURLDecoder::decode("monero:4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skxNgYeYTRj5UzqtReoS44qo9mtmXCqY45DJ852K5Jv2684Rge").is_err());

        assert!(PaymentURLDecoder::decode("bitcoin:").is_err());
        assert!(PaymentURLDecoder::decode("bitcoin:?amount=0.1").is_err());
        assert!(PaymentURLDecoder::decode("ethereum:").is_err());
        assert!(PaymentURLDecoder::decode("").is_err());
    }
}
