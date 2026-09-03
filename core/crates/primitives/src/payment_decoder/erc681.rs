use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain, ChainType,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

const PAY_PREFIX: &str = "pay-";
const HEXADECIMAL_PREFIX: &str = "0x";
const TRANSFER_FUNCTION: &str = "transfer";

const QUERY_ADDRESS: &str = "address";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const QUERY_UINT256: &str = "uint256";
const QUERY_VALUE: &str = "value";

pub fn decode(path: &str) -> Result<Payment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let (target, function) = path.split_once('/').map_or((path, None), |(target, function)| (target, Some(function)));
    let (target, chain) = match target.split_once('@') {
        Some((target, chain_id)) => (
            target,
            chain(chain_id).ok_or_else(|| PaymentDecoderError::InvalidFormat(format!("Unsupported chain id: {chain_id}")))?,
        ),
        None => (target, Chain::Ethereum),
    };
    let target = target.strip_prefix(PAY_PREFIX).unwrap_or(target);
    let parameters = query::parameters(query);
    let memo = query::value(&parameters, QUERY_MEMO);

    match function {
        Some(TRANSFER_FUNCTION) => Ok(Payment::Request(PaymentRequest {
            address: query::value(&parameters, QUERY_ADDRESS).ok_or_else(|| PaymentDecoderError::MissingField(QUERY_ADDRESS.to_string()))?,
            amount: query::value(&parameters, QUERY_UINT256)
                .and_then(|value| amount::atomic(&value))
                .map(PaymentAmount::AtomicValue),
            memo,
            references: None,
            asset_id: Some(AssetId::from(chain, Some(target.to_string()))),
        })),
        Some(function) => Err(PaymentDecoderError::InvalidFormat(format!("Unsupported function: {function}"))),
        None if target.is_empty() => Err(PaymentDecoderError::MissingField(QUERY_ADDRESS.to_string())),
        None => Ok(Payment::Request(PaymentRequest {
            address: target.to_string(),
            amount: query::value(&parameters, QUERY_VALUE)
                .and_then(|value| amount::exact_from_atomic(&value, chain))
                .or_else(|| query::value(&parameters, QUERY_AMOUNT).and_then(|value| amount::exact(&value, chain)))
                .map(PaymentAmount::ExactValue),
            memo,
            references: None,
            asset_id: Some(AssetId::from_chain(chain)),
        })),
    }
}

fn chain(chain_id: &str) -> Option<Chain> {
    let chain_id = match chain_id.strip_prefix(HEXADECIMAL_PREFIX) {
        Some(hexadecimal) => u64::from_str_radix(hexadecimal, 16).ok()?,
        None => chain_id.parse().ok()?,
    };

    Chain::from_chain_id(chain_id).filter(|chain| chain.chain_type() == ChainType::Ethereum)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use super::*;

    const ADDRESS: &str = "0xcB3028d6120802148f03d6c884D6AD6A210Df62A";
    const TOKEN: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    #[test]
    fn test_decode() {
        assert_eq!(
            decode(&format!("{ADDRESS}@1")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(&format!("{ADDRESS}@0x38?amount=1.23")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("1.23".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::SmartChain)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(&format!("{ADDRESS}?value=2.014e18")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("2.014".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(&format!("pay-{ADDRESS}?value=1e6")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("0.000000000001".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode("0x32Be343B94f860124dC4fEe278FDCBD38C102D88?value=10&gas=200000&gasPrice=20000000000").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0x32Be343B94f860124dC4fEe278FDCBD38C102D88".to_string(),
                amount: Some(PaymentAmount::ExactValue("0.00000000000000001".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );

        assert_eq!(
            decode("my-wallet.eth").unwrap(),
            Payment::Request(PaymentRequest {
                address: "my-wallet.eth".to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode("pay-gemwallet.eth@1").unwrap(),
            Payment::Request(PaymentRequest {
                address: "gemwallet.eth".to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                ..PaymentRequest::mock()
            })
        );
    }

    #[test]
    fn test_decode_token_transfer() {
        let token = Some(AssetId::from(Chain::Ethereum, Some(TOKEN.to_string())));

        let one_and_a_half_usdc = Payment::Request(PaymentRequest {
            address: ADDRESS.to_string(),
            amount: Some(PaymentAmount::AtomicValue(BigUint::from(1_500_000u32))),
            asset_id: token.clone(),
            ..PaymentRequest::mock()
        });

        assert_eq!(decode(&format!("{TOKEN}@1/transfer?address={ADDRESS}&uint256=1500000")).unwrap(), one_and_a_half_usdc);
        assert_eq!(decode(&format!("{TOKEN}@1/transfer?address={ADDRESS}&uint256=1.5e6")).unwrap(), one_and_a_half_usdc);
        assert_eq!(
            decode(&format!("{TOKEN}/transfer?address={ADDRESS}&uint256=1.5")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                asset_id: token,
                ..PaymentRequest::mock()
            })
        );
    }

    #[test]
    fn test_decode_refuses_what_it_cannot_sign() {
        assert_eq!(
            decode(&format!("{ADDRESS}/approve?value=1000000000000000000")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported function: approve".to_string()))
        );
        assert_eq!(
            decode(&format!("{TOKEN}/transfer?uint256=1")),
            Err(PaymentDecoderError::MissingField(QUERY_ADDRESS.to_string()))
        );

        assert_eq!(
            decode(&format!("{ADDRESS}@999999?amount=1")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported chain id: 999999".to_string()))
        );
        assert_eq!(
            decode(&format!("{ADDRESS}@1337?amount=1")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported chain id: 1337".to_string()))
        );
        assert_eq!(
            decode(&format!("{ADDRESS}@0x?amount=1")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported chain id: 0x".to_string()))
        );
        assert_eq!(
            decode(&format!("{ADDRESS}@chain?amount=1")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported chain id: chain".to_string()))
        );
    }
}
