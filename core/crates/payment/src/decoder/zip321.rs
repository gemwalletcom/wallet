use gem_encoding::decode_base64_url;
use primitives::{
    Chain,
    payment::{Payment, PaymentRequest},
};

use super::bip21;
use super::error::{PaymentDecoderError, Result};
use super::query;

const QUERY_ADDRESS: &str = "address";

pub fn decode(path: &str) -> Result<Payment> {
    let (path_address, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);
    if parameters.iter().any(|(key, _)| is_payment_index(key)) {
        return Err(PaymentDecoderError::InvalidFormat("Multiple Zcash payments are not supported".to_string()));
    }
    let address = match (path_address, query::values(&parameters, QUERY_ADDRESS).as_slice()) {
        (address, []) => address.to_string(),
        ("", [address]) => address.clone(),
        _ => return Err(PaymentDecoderError::InvalidFormat("Duplicate Zcash address".to_string())),
    };
    let request = bip21::get_request(Some(Chain::Zcash), &format!("{address}?{query}"))?;
    Ok(Payment::Request {
        request: PaymentRequest {
            memo: request.memo.as_deref().map(decode_memo).transpose()?,
            ..request
        },
    })
}

fn is_payment_index(key: &str) -> bool {
    key.rsplit_once('.').is_some_and(|(_, index)| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
}

fn decode_memo(memo: &str) -> Result<String> {
    let bytes = decode_base64_url(memo).map_err(|_| PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()))?;
    String::from_utf8(bytes).map_err(|_| PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, payment::PaymentAmount};

    const ADDRESS: &str = "u1qhwq6evpqq04ga6dn06mexlk6pcarhru4mhpkv63ar7nsmqvk5057lgtuj2yx6nhxlu0wuhzuw74v4q0eskt7gxy2xplq4qe5m560gukxtnx86mu5qljhgygyxzy7ulavg02qc42hxs";

    #[test]
    fn test_decode() {
        assert_eq!(
            decode(&format!("{ADDRESS}?amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap(),
            Payment::Request {
                request: PaymentRequest {
                    address: ADDRESS.to_string(),
                    amount: Some(PaymentAmount::ExactValue { value: "0.5".to_string() }),
                    memo: Some("Thank you".to_string()),
                    label: Some("Coffee".to_string()),
                    references: None,
                    asset_id: Some(AssetId::from_chain(Chain::Zcash)),
                },
            }
        );
        assert_eq!(
            decode(&format!("?address={ADDRESS}&amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap(),
            decode(&format!("{ADDRESS}?amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap()
        );
        assert_eq!(decode(&format!("{ADDRESS}?memo=not+base64")).unwrap_err(), PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()));
        assert_eq!(decode(&format!("{ADDRESS}?memo=_w")).unwrap_err(), PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()));
    }

    #[test]
    fn test_decode_rejects_ambiguous_payments() {
        let multiple = PaymentDecoderError::InvalidFormat("Multiple Zcash payments are not supported".to_string());
        let duplicate = PaymentDecoderError::InvalidFormat("Duplicate Zcash address".to_string());

        assert_eq!(decode(&format!("{ADDRESS}?amount=0.5&address.1={ADDRESS}&amount.1=0.1")).unwrap_err(), multiple);
        assert_eq!(decode(&format!("?address={ADDRESS}&amount.2=0.1")).unwrap_err(), multiple);
        assert_eq!(decode(&format!("{ADDRESS}?address={ADDRESS}")).unwrap_err(), duplicate);
        assert_eq!(decode(&format!("?address={ADDRESS}&address={ADDRESS}")).unwrap_err(), duplicate);
        assert_eq!(decode("?amount=0.5").unwrap_err(), PaymentDecoderError::MissingField("address".to_string()));
    }
}
