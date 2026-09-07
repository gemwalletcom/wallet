use super::bip21;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{Chain, payment::Payment};

const QUERY_SEGWIT_ADDRESS: &str = "bc";

pub fn decode(path: &str) -> Result<Payment> {
    let (address, query) = path.split_once('?').unwrap_or((path, ""));
    if !address.is_empty() {
        return bip21::decode(Some(Chain::Bitcoin), path);
    }
    let address = query::value(&query::parameters(query), QUERY_SEGWIT_ADDRESS).ok_or_else(|| PaymentDecoderError::MissingField("address".to_string()))?;
    bip21::decode(Some(Chain::Bitcoin), &format!("{address}?{query}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetId,
        payment::{PaymentAmount, PaymentRequest},
    };

    const ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

    #[test]
    fn test_decode() {
        let payment = Payment::Request(PaymentRequest {
            address: ADDRESS.to_string(),
            amount: Some(PaymentAmount::ExactValue("0.001".to_string())),
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..PaymentRequest::mock()
        });

        assert_eq!(decode(&format!("{ADDRESS}?amount=0.001")).unwrap(), payment);
        assert_eq!(decode(&format!("?bc={ADDRESS}&amount=0.001")).unwrap(), payment);
        assert_eq!(decode(&format!("{ADDRESS}?bc=bc1qother&amount=0.001")).unwrap(), payment);
        assert_eq!(decode(&format!("{ADDRESS}?sp=sp1qsilentpayment&amount=0.001")).unwrap(), payment);
        assert_eq!(decode(&format!("{ADDRESS}?lightning=lnbc420bogusinvoice&amount=0.001")).unwrap(), payment);
    }

    #[test]
    fn test_decode_refuses_what_it_cannot_sign() {
        assert!(decode("").is_err());
        assert!(decode("?amount=0.001").is_err());
        assert!(decode("?bc=&amount=0.001").is_err());
        assert!(decode("?lightning=lnbc420bogusinvoice").is_err());
        assert!(decode("?lno=lno1bogusoffer&sp=sp1qsilentpayment").is_err());
    }
}
