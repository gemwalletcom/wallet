use super::bip21;
use super::error::Result;
use super::query;
use crate::{
    Chain,
    payment::{Payment, PaymentRequest},
};

const QUERY_DESTINATION_TAG: &str = "dt";

pub fn decode(path: &str) -> Result<Payment> {
    let request = bip21::get_request(Some(Chain::Xrp), path)?;
    let query = path.split_once('?').map_or("", |(_, query)| query);
    Ok(Payment::Request(PaymentRequest {
        memo: request.memo.or_else(|| query::value(&query::parameters(query), QUERY_DESTINATION_TAG)),
        ..request
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, payment::PaymentAmount};

    const ADDRESS: &str = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh";

    #[test]
    fn test_decode() {
        let payment = Payment::Request(PaymentRequest {
            address: ADDRESS.to_string(),
            amount: Some(PaymentAmount::ExactValue("10".to_string())),
            memo: Some("12345".to_string()),
            references: None,
            asset_id: Some(AssetId::from_chain(Chain::Xrp)),
        });

        assert_eq!(decode(&format!("{ADDRESS}?dt=12345&amount=10")).unwrap(), payment);
        assert_eq!(decode(&format!("{ADDRESS}?memo=12345&amount=10")).unwrap(), payment);
        assert_eq!(decode(&format!("{ADDRESS}?memo=12345&dt=99999&amount=10")).unwrap(), payment);
        assert_eq!(
            decode(ADDRESS).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Xrp)),
                ..PaymentRequest::mock()
            })
        );
    }
}
