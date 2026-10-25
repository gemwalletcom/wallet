use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PATH_PAYMENT_STRICT_SEND, TRANSACTION_TYPE_PAYMENT};
use crate::models::transaction::{Payment, StellarTransactionBroadcast};
use crate::provider::balances_mapper::map_asset_id;
use chrono::DateTime;
use num_bigint::BigUint;
use primitives::{AssetId, SwapProvider, Transaction, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType, chain::Chain, swap::STELLAR_REFERRAL_ADDRESS};
use std::error::Error;
use url::form_urlencoded;

pub fn encode_transaction_data(data: &str) -> String {
    form_urlencoded::Serializer::new(String::new()).append_pair("tx", data).finish()
}

pub fn map_transaction_broadcast(response: &StellarTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(hash) = &response.hash
        && response.tx_status == "PENDING"
    {
        Ok(hash.clone())
    } else if let Some(error) = &response.error_message {
        Err(format!("Broadcast error: {}", error).into())
    } else if let Some(error) = &response.error_result_xdr {
        Err(format!("Broadcast error: {}", error).into())
    } else {
        Err("Unknown broadcast error".into())
    }
}

pub fn map_transactions(chain: Chain, transactions: Vec<Payment>) -> Vec<Transaction> {
    transactions
        .chunk_by(|first, second| first.transaction_hash == second.transaction_hash)
        .flat_map(|payments| {
            let swap = payments
                .iter()
                .find(|payment| payment.payment_type == TRANSACTION_TYPE_PATH_PAYMENT_STRICT_SEND && payment.from.is_some() && payment.from == payment.to);
            let fee = swap.and_then(|swap| {
                payments
                    .iter()
                    .find(|payment| payment.payment_type == TRANSACTION_TYPE_PAYMENT && payment.from == swap.from && payment.to.as_deref() == Some(STELLAR_REFERRAL_ADDRESS))
            });
            let swap = swap.and_then(|swap| map_swap(chain, swap, fee));
            let absorbed_fee = fee.filter(|_| swap.is_some());
            let others = payments.iter().filter(|payment| absorbed_fee.is_none_or(|fee| fee.id != payment.id)).cloned();
            swap.into_iter().chain(others.filter_map(|payment| map_transaction(chain, payment))).collect::<Vec<_>>()
        })
        .collect()
}

pub fn map_transaction_by_hash(chain: Chain, transactions: Vec<Payment>, hash: &str) -> Option<Transaction> {
    map_transactions(chain, transactions).into_iter().find(|transaction| transaction.hash() == hash)
}

fn map_swap(chain: Chain, swap: &Payment, fee: Option<&Payment>) -> Option<Transaction> {
    let from_asset = map_asset_id(chain, swap.source_asset_type.as_deref()?, swap.source_asset_issuer.as_ref(), swap.source_asset_code.as_ref())?;
    let to_asset = map_asset_id(chain, swap.asset_type.as_deref()?, swap.asset_issuer.as_ref(), swap.asset_code.as_ref())?;
    let referral_fee = fee.and_then(|payment| {
        Some(TransactionSwapReferralFee {
            asset_id: map_asset_id(chain, payment.asset_type.as_deref()?, payment.asset_issuer.as_ref(), payment.asset_code.as_ref())?,
            value: payment.amount.clone()?,
        })
    });
    let fee_in = |asset_id: &AssetId| referral_fee.as_ref().filter(|fee| fee.asset_id == *asset_id).map_or(BigUint::ZERO, |fee| fee.value.clone());
    let from_value = swap.source_amount.clone()? + fee_in(&from_asset);
    let (received, to_fee) = (swap.amount.clone()?, fee_in(&to_asset));
    let to_value = (to_fee <= received).then(|| received - to_fee)?;
    let metadata = TransactionSwapMetadata::new(from_asset.clone(), from_value.clone(), to_asset, to_value, SwapProvider::StellarDex).with_referral_fee(referral_fee);
    let address = swap.from.clone()?;

    Some(Transaction::new(
        swap.transaction_hash.clone(),
        from_asset,
        address.clone(),
        address,
        None,
        TransactionType::Swap,
        swap.get_state(),
        swap.fee_charged(),
        chain.as_asset_id(),
        from_value,
        swap.get_memo(),
        serde_json::to_value(metadata).ok(),
        DateTime::parse_from_rfc3339(&swap.created_at).ok()?.into(),
    ))
}

pub fn map_transaction(chain: Chain, transaction: Payment) -> Option<Transaction> {
    match transaction.payment_type.as_str() {
        TRANSACTION_TYPE_PAYMENT | TRANSACTION_TYPE_CREATE_ACCOUNT => {
            if transaction.clone().asset_type.unwrap_or_default() == "native" || transaction.clone().payment_type.as_str() == TRANSACTION_TYPE_CREATE_ACCOUNT {
                let created_at = DateTime::parse_from_rfc3339(&transaction.created_at).ok()?.into();

                return Some(Transaction::new(
                    transaction.clone().transaction_hash,
                    chain.as_asset_id(),
                    transaction.from_address()?,
                    transaction.to_address()?,
                    None,
                    TransactionType::Transfer,
                    transaction.get_state(),
                    transaction.fee_charged(),
                    chain.as_asset_id(),
                    transaction.get_value()?,
                    transaction.clone().get_memo(),
                    None,
                    created_at,
                ));
            }

            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{
            Embedded,
            transaction::{Payment, StellarTransactionBroadcast, StellarTransactionStatus},
        },
        provider::testkit::TEST_TRANSACTION_ID,
    };
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain};

    #[test]
    fn test_encode_transaction_data_variants() {
        assert_eq!(encode_transaction_data("payload"), "tx=payload");

        let special = encode_transaction_data("tx=abc/123?&value=42 +more");
        assert_eq!(special, "tx=tx%3Dabc%2F123%3F%26value%3D42+%2Bmore");
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = StellarTransactionBroadcast {
            hash: Some("test_hash_123".to_string()),
            error_message: None,
            tx_status: "PENDING".to_string(),
            error_result_xdr: None,
        };

        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "test_hash_123");
    }

    #[test]
    fn test_map_transaction_broadcast_with_real_data() {
        let response: StellarTransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_status_success.json")).unwrap();

        let result = map_transaction_broadcast(&StellarTransactionBroadcast {
            hash: Some(response.hash),
            error_message: None,
            tx_status: "PENDING".to_string(),
            error_result_xdr: None,
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dbc69dff72e4ca7ddf47311e12da09ac5952c777d19855f95f13b0ec624f8baf");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let payments: Embedded<Payment> = serde_json::from_str(include_str!("../../testdata/transaction_by_hash.json")).unwrap();
        let transaction = map_transaction_by_hash(Chain::Stellar, payments._embedded.records, TEST_TRANSACTION_ID).unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        assert_eq!(transaction.from, "GFROM");
        assert_eq!(transaction.to, "GTO");
        assert_eq!(transaction.memo, Some("49639518".to_string()));
        assert_eq!(transaction.fee, BigUint::from(1500u32), "the fee comes from the joined transaction, not a constant");
    }

    #[test]
    fn test_a_payment_without_a_joined_transaction_falls_back_to_the_base_fee() {
        let payment: Payment = serde_json::from_str(
            r#"{"id":"1","transaction_successful":true,"transaction_hash":"h","type":"payment","asset_type":"native","from":"GFROM","to":"GTO","amount":"0.0001","created_at":"2026-01-11T05:10:52Z","transaction":null}"#,
        )
        .unwrap();

        assert_eq!(map_transaction(Chain::Stellar, payment).unwrap().fee, BigUint::from(100u32));
    }

    #[test]
    fn test_map_transactions_swap_records_what_left_the_wallet_and_what_it_kept() {
        let records = || serde_json::from_str::<Embedded<Payment>>(include_str!("../../testdata/transaction_swap_path_payment.json")).unwrap()._embedded.records;
        let fee_payment = |asset: &str, amount: &str| -> Payment {
            serde_json::from_str(&format!(
                r#"{{"id":"2","transaction_successful":true,"transaction_hash":"b652bc89d2c3b29573a512d76088d8fc3108298947451197af019709185f2ef2","type":"payment",{asset},"from":"GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL","to":"{STELLAR_REFERRAL_ADDRESS}","amount":"{amount}","created_at":"2026-10-25T04:21:03Z","transaction":{{"memo":null,"fee_charged":"300"}}}}"#
            ))
            .unwrap()
        };
        let usdc = AssetId::from_token(Chain::Stellar, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC");
        let swap = |fee: Option<Payment>| {
            let transactions = map_transactions(Chain::Stellar, [records(), fee.into_iter().collect()].concat());
            assert_eq!(transactions.len(), 1, "Gem's fee payment is part of the swap, not a transfer of its own");
            assert_eq!(transactions[0].transaction_type, TransactionType::Swap);
            serde_json::from_value::<TransactionSwapMetadata>(transactions[0].metadata.clone().unwrap()).unwrap()
        };
        let xlm_fee = swap(Some(fee_payment(r#""asset_type":"native""#, "0.0100170")));
        let usdc_fee = swap(Some(fee_payment(
            r#""asset_type":"credit_alphanum4","asset_code":"USDC","asset_issuer":"GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN""#,
            "0.0017163",
        )));

        assert_eq!(
            swap(None),
            TransactionSwapMetadata::new(Chain::Stellar.as_asset_id(), BigUint::from(20_034_027u32), usdc.clone(), BigUint::from(3_432_692u32), SwapProvider::StellarDex)
        );
        assert_eq!((xlm_fee.from_value, xlm_fee.to_value), (BigUint::from(20_134_197u32), BigUint::from(3_432_692u32)), "the XLM paid includes Gem's fee");
        assert_eq!((usdc_fee.from_value, usdc_fee.to_value), (BigUint::from(20_034_027u32), BigUint::from(3_415_529u32)), "the USDC kept excludes Gem's fee");
        assert_eq!(
            usdc_fee.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: usdc,
                value: BigUint::from(17_163u32),
            })
        );
        assert_eq!(
            map_transaction_by_hash(Chain::Stellar, records(), "b652bc89d2c3b29573a512d76088d8fc3108298947451197af019709185f2ef2").unwrap().transaction_type,
            TransactionType::Swap
        );
    }

    #[test]
    fn test_map_transactions_swap_keeps_a_payment_to_someone_else() {
        let records = serde_json::from_str::<Embedded<Payment>>(include_str!("../../testdata/transaction_swap_path_payment.json")).unwrap()._embedded.records;
        let payment: Payment = serde_json::from_str(
            r#"{"id":"2","transaction_successful":true,"transaction_hash":"b652bc89d2c3b29573a512d76088d8fc3108298947451197af019709185f2ef2","type":"payment","asset_type":"native","from":"GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL","to":"GTHIRD","amount":"1.0000000","created_at":"2026-10-25T04:21:03Z","transaction":{"memo":null,"fee_charged":"200"}}"#,
        )
        .unwrap();

        let transactions = map_transactions(Chain::Stellar, [records, vec![payment]].concat());

        assert_eq!(
            transactions.iter().map(|transaction| transaction.transaction_type.clone()).collect::<Vec<_>>(),
            vec![TransactionType::Swap, TransactionType::Transfer]
        );
        assert_eq!((transactions[1].to.as_str(), transactions[1].value.clone()), ("GTHIRD", BigUint::from(10_000_000u32)));
    }
}
