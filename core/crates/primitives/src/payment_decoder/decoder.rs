use std::str::FromStr;

use super::error::{PaymentDecoderError, Result};
use crate::{Chain, ChainType, payment::Payment};

use super::{bip21, erc681, solana_pay, ton_pay};

const DOGECOIN_SCHEME: &str = "dogecoin";
const RIPPLE_SCHEME: &str = "ripple";
const XRPL_SCHEME: &str = "xrpl";

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment> {
        let uri = string.trim();
        let payment = Self::decode_uri(uri.split_once('#').map_or(uri, |(uri, _)| uri))?;

        match payment {
            Payment::Request(request) if request.address.is_empty() => Err(PaymentDecoderError::MissingField("address".to_string())),
            payment @ (Payment::Request(_) | Payment::Link(_)) => Ok(payment),
        }
    }

    fn decode_uri(uri: &str) -> Result<Payment> {
        let Some((scheme, path)) = uri.split_once(':') else {
            return bip21::decode(None, uri);
        };
        let path = path.strip_prefix("//").unwrap_or(path);

        match get_chain(&scheme.to_ascii_lowercase()).ok_or(PaymentDecoderError::InvalidScheme)? {
            Chain::Ethereum => erc681::decode(path),
            Chain::Solana => solana_pay::decode(path),
            Chain::Ton => ton_pay::decode(path),
            chain => bip21::decode(Some(chain), path),
        }
    }
}

fn get_chain(scheme: &str) -> Option<Chain> {
    let chain = match scheme {
        DOGECOIN_SCHEME => Chain::Doge,
        RIPPLE_SCHEME | XRPL_SCHEME => Chain::Xrp,
        scheme => Chain::from_str(scheme).ok()?,
    };
    match chain {
        Chain::Ethereum | Chain::Solana | Chain::Ton | Chain::Xrp => Some(chain),
        chain if chain.chain_type() == ChainType::Bitcoin => Some(chain),
        _ => None,
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
            Payment::Request(PaymentRequest {
                address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string(),
                ..PaymentRequest::mock()
            })
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
            PaymentURLDecoder::decode("dogecoin:DH5yaieqoZN36fDVciNyRueRGvGLR3mr7L").unwrap(),
            PaymentURLDecoder::decode("doge:DH5yaieqoZN36fDVciNyRueRGvGLR3mr7L").unwrap()
        );
        assert_eq!(
            PaymentURLDecoder::decode("ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh").unwrap(),
            PaymentURLDecoder::decode("xrpl:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh").unwrap()
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
        assert!(PaymentURLDecoder::decode("algorand://TIQ4WPFJQYLA5PBQFQZLLBKMDNQFGZDLTKLGPKCUOJPLQZQPQFQZLLBKMD").is_err());
        assert!(PaymentURLDecoder::decode("polygon:0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").is_err());
        assert!(PaymentURLDecoder::decode("cosmos:cosmos1qypqxpq9qcrsszg2pvxq6rs0zqg3yyc5lzv7xu").is_err());

        assert!(PaymentURLDecoder::decode("bitcoin:").is_err());
        assert!(PaymentURLDecoder::decode("bitcoin:?amount=0.1").is_err());
        assert!(PaymentURLDecoder::decode("ethereum:").is_err());
        assert!(PaymentURLDecoder::decode("").is_err());
    }
}
