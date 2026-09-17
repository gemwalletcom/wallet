use chrono::{Duration, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, Transaction, TransactionDirection, TransactionId, TransactionState, TransactionSwapMetadata, TransactionType};

const SOLANA_ADDRESS: &str = "7nVDzZUjrBA3gHs3gNcHidhmR96CH7KpKsU8pyBZGHUr";
const ETHEREUM_ADDRESS: &str = "0xf1158986419F6058231b0Dbd7A78Ff0674ebBc50";
const BITCOIN_ADDRESS: &str = "bc1q4jwwsy7txnzsr7w53j4wnrg6rrnmj86a47e2t9";
const TRON_ADDRESS: &str = "TAw8sw21A3pGDCtHGuB55BGDqLVHQTYwAC";

struct SampleTransaction {
    direction: TransactionDirection,
    from: &'static str,
    to: &'static str,
    chain: Chain,
    transaction_type: TransactionType,
    value: u128,
    swap: Option<(Chain, u128, Chain, u128)>,
    seconds_ago: i64,
}

pub fn sample_transactions() -> Vec<Transaction> {
    let now = Utc::now();
    samples()
        .into_iter()
        .enumerate()
        .map(|(index, sample)| {
            let asset_id = AssetId::from_chain(sample.chain);
            let metadata = sample.swap.map(|(from_chain, from_value, to_chain, to_value)| {
                serde_json::to_value(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(from_chain),
                    from_value: BigUint::from(from_value),
                    to_asset: AssetId::from_chain(to_chain),
                    to_value: BigUint::from(to_value),
                    provider: None,
                })
                .expect("swap metadata is serializable")
            });
            Transaction {
                id: TransactionId::new(sample.chain, index.to_string()),
                asset_id: asset_id.clone(),
                from: sample.from.to_string(),
                to: sample.to.to_string(),
                contract: None,
                transaction_type: sample.transaction_type,
                state: TransactionState::Confirmed,
                block_number: Some("0".to_string()),
                sequence: Some("0".to_string()),
                fee: BigUint::ZERO,
                fee_asset_id: asset_id,
                value: BigUint::from(sample.value),
                memo: None,
                direction: sample.direction,
                utxo_inputs: Some(vec![]),
                utxo_outputs: Some(vec![]),
                metadata,
                data: None,
                created_at: now - Duration::seconds(sample.seconds_ago),
            }
        })
        .collect()
}

fn samples() -> Vec<SampleTransaction> {
    vec![
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: SOLANA_ADDRESS,
            to: "",
            chain: Chain::Solana,
            transaction_type: TransactionType::Transfer,
            value: 111_111_111,
            swap: None,
            seconds_ago: 1,
        },
        SampleTransaction {
            direction: TransactionDirection::Outgoing,
            from: "",
            to: SOLANA_ADDRESS,
            chain: Chain::Solana,
            transaction_type: TransactionType::Transfer,
            value: 3_311_111_111,
            swap: None,
            seconds_ago: 2,
        },
        SampleTransaction {
            direction: TransactionDirection::SelfTransfer,
            from: "",
            to: "",
            chain: Chain::Sui,
            transaction_type: TransactionType::Swap,
            value: 76_767_623_311_111_111,
            swap: Some((Chain::Sui, 2_767_611_111, Chain::Solana, 812_312_312)),
            seconds_ago: 122_223,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: TRON_ADDRESS,
            to: "",
            chain: Chain::Tron,
            transaction_type: TransactionType::Transfer,
            value: 912_312_312,
            swap: None,
            seconds_ago: 122_224,
        },
        SampleTransaction {
            direction: TransactionDirection::Outgoing,
            from: "",
            to: ETHEREUM_ADDRESS,
            chain: Chain::Ethereum,
            transaction_type: TransactionType::Transfer,
            value: 76_767_623_311_111_111,
            swap: None,
            seconds_ago: 1_344_411,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: BITCOIN_ADDRESS,
            to: "",
            chain: Chain::Bitcoin,
            transaction_type: TransactionType::Transfer,
            value: 621_111_111,
            swap: None,
            seconds_ago: 100,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: BITCOIN_ADDRESS,
            to: "",
            chain: Chain::Bitcoin,
            transaction_type: TransactionType::Transfer,
            value: 46_161_111,
            swap: None,
            seconds_ago: 10_000,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: BITCOIN_ADDRESS,
            to: "",
            chain: Chain::Bitcoin,
            transaction_type: TransactionType::Transfer,
            value: 72_312_312,
            swap: None,
            seconds_ago: 1_344_401,
        },
        SampleTransaction {
            direction: TransactionDirection::SelfTransfer,
            from: "",
            to: "",
            chain: Chain::Ethereum,
            transaction_type: TransactionType::Swap,
            value: 76_767_623_311_111_111,
            swap: Some((Chain::Ethereum, 276_767_623_311_111_111, Chain::Bitcoin, 32_312_312)),
            seconds_ago: 1_344_411,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: "",
            to: "",
            chain: Chain::SmartChain,
            transaction_type: TransactionType::StakeRewards,
            value: 464_222_222_272_312_312,
            swap: None,
            seconds_ago: 1_444_401,
        },
        SampleTransaction {
            direction: TransactionDirection::Incoming,
            from: "",
            to: "NodeReal",
            chain: Chain::SmartChain,
            transaction_type: TransactionType::StakeDelegate,
            value: 54_213_322_222_272_312_312,
            swap: None,
            seconds_ago: 1_464_401,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_transactions_cover_every_row_the_screen_draws() {
        let transactions = sample_transactions();
        assert_eq!(transactions.len(), 11);
        assert_eq!(transactions.iter().filter(|transaction| transaction.metadata.is_some()).count(), 2);
        assert!(transactions.iter().all(|transaction| transaction.state == TransactionState::Confirmed));
        assert_eq!(
            transactions.iter().map(|transaction| transaction.id.hash.clone()).collect::<Vec<_>>(),
            (0..11).map(|index| index.to_string()).collect::<Vec<_>>()
        );
    }
}
