use gem_encoding::decode_base64;
use gem_hash::sha2::sha256;
use num_bigint::BigUint;
use primitives::chain_cosmos::CosmosChain;
use primitives::hex;
use primitives::{AssetId, Transaction, TransactionState, TransactionType};
use std::error::Error;

use crate::constants::get_base_fee;
use crate::models::BroadcastResponse;
use crate::models::{Message, TransactionResponse, TransactionResponseCoin};

pub fn map_transaction_broadcast(response: &BroadcastResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(tx_response) = &response.tx_response {
        if tx_response.code != 0 { Err(tx_response.raw_log.clone().into()) } else { Ok(tx_response.txhash.clone()) }
    } else if let Some(message) = &response.message {
        Err(format!("Broadcast error: {}", message).into())
    } else {
        Err("Unknown broadcast error".into())
    }
}

pub fn map_transaction_decode(body: &str) -> Option<String> {
    let bytes = decode_base64(body).ok()?;
    let decoded_str = String::from_utf8_lossy(&bytes);
    let has_supported_type = crate::constants::SUPPORTED_MESSAGES.iter().any(|msg_type| decoded_str.contains(msg_type));
    if has_supported_type { Some(get_hash(&bytes)) } else { None }
}

pub fn get_hash(bytes: &[u8]) -> String {
    hex::encode(sha256(bytes)).to_uppercase()
}

pub fn map_transactions(chain: CosmosChain, transactions: Vec<TransactionResponse>) -> Vec<primitives::Transaction> {
    transactions.into_iter().filter_map(|transaction| map_transaction(chain, transaction)).collect()
}

fn asset_id_from_denom(chain: primitives::Chain, denom: &str, default_denom: &str) -> AssetId {
    if denom == default_denom { chain.as_asset_id() } else { AssetId::token(chain, denom) }
}

#[allow(clippy::match_like_matches_macro)]
fn is_reward_withdrawal(message: &Message<TransactionResponseCoin>) -> bool {
    match message {
        Message::MsgWithdrawDelegatorReward(_) => true,
        _ => false,
    }
}

fn select_transaction_message(messages: Vec<Message<TransactionResponseCoin>>) -> Option<Message<TransactionResponseCoin>> {
    let (rewards, actions): (Vec<_>, Vec<_>) = messages.into_iter().partition(is_reward_withdrawal);
    match (actions.as_slice(), rewards.is_empty()) {
        ([], false) => rewards.into_iter().next(),
        ([Message::MsgUndelegate(_) | Message::MsgBeginRedelegate(_)], false) | ([_], true) => actions.into_iter().next(),
        _ => None,
    }
}

pub fn map_transaction(cosmos_chain: CosmosChain, transaction: TransactionResponse) -> Option<Transaction> {
    if transaction.tx_response.code != 0 {
        return None;
    }

    let TransactionResponse { tx, tx_response } = transaction;
    let body = tx.body;
    let auth_info = tx.auth_info;

    let hash = tx_response.txhash.clone();
    let chain = cosmos_chain.as_chain();
    let default_denom = chain.as_denom()?.to_string();
    let native_asset_id = chain.as_asset_id();

    let fee_coin = auth_info.and_then(|info| info.fee.amount.into_iter().next());
    let fee = match fee_coin.as_ref() {
        Some(coin) => BigUint::try_from(coin.amount.clone()).ok()?,
        None => BigUint::from(get_base_fee(cosmos_chain)),
    };
    let fee_asset_id = fee_coin.as_ref().map_or_else(|| native_asset_id.clone(), |coin| asset_id_from_denom(chain, &coin.denom, &default_denom));

    let memo = if body.memo.is_empty() { None } else { Some(body.memo.clone()) };

    let message = select_transaction_message(body.messages)?;
    let asset_id: AssetId;
    let transaction_type: TransactionType;
    let value: BigUint;
    let from_address: String;
    let to_address: String;

    match message {
        Message::MsgSend(message) => {
            let coin = message.amount.first()?;
            asset_id = asset_id_from_denom(chain, &coin.denom, &default_denom);
            transaction_type = TransactionType::Transfer;
            value = BigUint::try_from(coin.amount.clone()).ok()?;
            from_address = message.from_address;
            to_address = message.to_address;
        }
        Message::MsgDelegate(message) => {
            asset_id = native_asset_id;
            transaction_type = TransactionType::StakeDelegate;
            value = BigUint::try_from(message.amount?.amount).ok()?;
            from_address = message.delegator_address;
            to_address = message.validator_address;
        }
        Message::MsgUndelegate(message) => {
            asset_id = native_asset_id;
            transaction_type = TransactionType::StakeUndelegate;
            value = BigUint::try_from(message.amount?.amount).ok()?;
            from_address = message.delegator_address;
            to_address = message.validator_address;
        }
        Message::MsgBeginRedelegate(message) => {
            asset_id = native_asset_id;
            transaction_type = TransactionType::StakeRedelegate;
            value = BigUint::try_from(message.amount?.amount).ok()?;
            from_address = message.delegator_address;
            to_address = message.validator_dst_address;
        }
        Message::MsgWithdrawDelegatorReward(message) => {
            asset_id = native_asset_id;
            value = tx_response.get_rewards_value(&default_denom)?;
            transaction_type = TransactionType::StakeRewards;
            from_address = message.delegator_address;
            to_address = message.validator_address;
        }
        Message::MsgDeposit(message) => {
            let coin = message.coins.first().filter(|coin| coin.asset.to_lowercase().ends_with(&format!(".{default_denom}")))?;
            asset_id = native_asset_id;
            transaction_type = TransactionType::SmartContractCall;
            value = coin.amount.parse().ok()?;
            from_address = message.signer.clone();
            to_address = message.signer;
        }
        _ => return None,
    }

    Some(Transaction::new(
        hash,
        asset_id,
        from_address,
        to_address,
        None,
        transaction_type,
        TransactionState::Confirmed,
        fee,
        fee_asset_id,
        value,
        memo,
        None,
        tx_response.timestamp,
    ))
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, DateTime};

    use super::*;
    use crate::models::transaction::{BroadcastResponse, TransactionResult};
    use crate::provider::testkit::TEST_TRANSACTION_ID;

    #[test]
    fn test_map_transaction_broadcast() {
        let success = BroadcastResponse {
            tx_response: Some(TransactionResult {
                txhash: "ABC123".to_string(),
                code: 0,
                raw_log: String::new(),
            }),
            code: None,
            message: None,
        };
        let failed: BroadcastResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_failed.json")).unwrap();

        assert_eq!(map_transaction_broadcast(&success).unwrap(), "ABC123");
        assert_eq!(
            map_transaction_broadcast(&failed).unwrap_err().to_string(),
            "signature verification failed; please verify account number (1343971) and chain-id (cosmoshub-4): (unable to verify single signer signature): unauthorized"
        );
    }

    #[test]
    fn test_map_transaction_transfer() {
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, serde_json::from_str(include_str!("../../testdata/transfer.json")).unwrap()),
            Some(Transaction::new(
                TEST_TRANSACTION_ID.to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1wev8ptzj27aueu04wgvvl4gvurax6rj5f0v7rw".to_string(),
                "cosmos1hgp84me0lze8t4jfrwsr05aep2kr57zrk4gecx".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(1600u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(50000000u64),
                Some("6439432658467882".to_string()),
                None,
                DateTime::parse_rfc3339("2025-06-20T04:09:19Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, serde_json::from_str(include_str!("../../testdata/transfer_ibc.json")).unwrap()),
            Some(Transaction::new(
                "90BBC25C199B58E6A4C9A2A3448C64E853B4E8DF3E88B1F6E35DE9FBF20400F6".to_string(),
                AssetId::token(Chain::Cosmos, "ibc/915992C8486D299941292A913640167F0BA02DC2F599BFFED0732CE932C2FCC4"),
                "cosmos1dej28rxfh39axghzlcusd98qhpkdarcqqu23ua".to_string(),
                "cosmos1n3wq399w3s6reslvngve5quw85xusf7la5cpfs".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(689u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(20000000000u64),
                None,
                None,
                DateTime::parse_rfc3339("2026-03-03T17:24:03Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Thorchain, serde_json::from_str(include_str!("../../testdata/transfer_thorchain.json")).unwrap()),
            Some(Transaction::new(
                "C4ED43321E89497C96B7084BE2AA2640EFB10A93A82F396B9FC7A8308F9662AE".to_string(),
                Chain::Thorchain.as_asset_id(),
                "thor1rr6rahhd4sy76a7rdxkjaen2q4k4pw2g06w7qp".to_string(),
                "thor1tpr8cqs2uncwfsggevmha4q4tc9eelu9r00cxx".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(2000000u64),
                Chain::Thorchain.as_asset_id(),
                BigUint::from(50000000000u64),
                Some("thankyou".to_string()),
                None,
                DateTime::parse_rfc3339("2025-10-03T00:39:55Z").unwrap(),
            ))
        );
    }

    #[test]
    fn test_map_transaction_deposit() {
        assert_eq!(
            map_transaction(CosmosChain::Thorchain, serde_json::from_str(include_str!("../../testdata/thorchain_msg_deposit.json")).unwrap()),
            Some(Transaction::new(
                "93CE6289CE4DB1FBEFCCB1AF56A86DD9D366AB8E1D12BC59A37F471369096EFC".to_string(),
                Chain::Thorchain.as_asset_id(),
                "thor14k0f875upg7uz6u5xylkay2yrv30x24tycwpwc".to_string(),
                "thor14k0f875upg7uz6u5xylkay2yrv30x24tycwpwc".to_string(),
                None,
                TransactionType::SmartContractCall,
                TransactionState::Confirmed,
                BigUint::from(2000000u64),
                Chain::Thorchain.as_asset_id(),
                BigUint::from(238536268u64),
                Some("=:e:0xc17ecB5049dd80fCca2b75E025F64613f17C67AA:0/1/0:g1:50".to_string()),
                None,
                DateTime::parse_rfc3339("2025-12-01T01:03:29Z").unwrap(),
            ))
        );
    }

    #[test]
    fn test_map_transaction_staking() {
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, TransactionResponse::mock_delegate()),
            Some(Transaction::new(
                "FD334515F2D872B6689D7B52598796BF91C42111C857D6E80E984BC6DB4B0575".to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1z64xeecaqudhe2scx0m4mtvh7d0g5khyakpsmw".to_string(),
                "cosmosvaloper1jlr62guqwrwkdt4m3y00zh2rrsamhjf9num5xr".to_string(),
                None,
                TransactionType::StakeDelegate,
                TransactionState::Confirmed,
                BigUint::from(5194u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(17732657u64),
                None,
                None,
                DateTime::parse_rfc3339("2025-06-21T20:33:42Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, serde_json::from_str(include_str!("../../testdata/rewards.json")).unwrap()),
            Some(Transaction::new(
                "0B615F5DDDB216574DF8AC07B104C3C902B23974C7957DF4275E1572CDDAFCB4".to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj".to_string(),
                "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
                None,
                TransactionType::StakeRewards,
                TransactionState::Confirmed,
                BigUint::from(25000u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(2385518u64),
                None,
                None,
                DateTime::parse_rfc3339("2025-06-21T20:51:28Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, TransactionResponse::mock_unstake()),
            Some(Transaction::new(
                "C11964CF4C025C7300632AE2AE1EF7DEB6E823991183E2B73C31A348402765EF".to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1qhymx7svuag4mzstm9g9qtqskwkzzumxn83a8z".to_string(),
                "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
                None,
                TransactionType::StakeUndelegate,
                TransactionState::Confirmed,
                BigUint::from(10000u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(11700000u64),
                Some("Stake via Gem Wallet".to_string()),
                None,
                DateTime::parse_rfc3339("2026-11-18T16:13:21Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, serde_json::from_str(include_str!("../../testdata/redelegate.json")).unwrap()),
            Some(Transaction::new(
                "CE342F6582FC26F86C4CF881FBE15B12C0A3BF8FDF30DBB423AABEC42CD05551".to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1qhymx7svuag4mzstm9g9qtqskwkzzumxn83a8z".to_string(),
                "cosmosvaloper1fhr7e04ct0zslmkzqt9smakg3sxrdve6ulclj2".to_string(),
                None,
                TransactionType::StakeRedelegate,
                TransactionState::Confirmed,
                BigUint::from(12500u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(326222u64),
                Some("Stake via Gem Wallet".to_string()),
                None,
                DateTime::parse_rfc3339("2026-11-03T20:45:14Z").unwrap(),
            ))
        );
        assert_eq!(
            map_transaction(CosmosChain::Cosmos, serde_json::from_str(include_str!("../../testdata/rewards_multiple_validators.json")).unwrap()),
            Some(Transaction::new(
                "D5CBABF775D5783E130DC06919C7805AC4F47050F78FAA445515426385A52357".to_string(),
                Chain::Cosmos.as_asset_id(),
                "cosmos1qhymx7svuag4mzstm9g9qtqskwkzzumxn83a8z".to_string(),
                "cosmosvaloper1fhr7e04ct0zslmkzqt9smakg3sxrdve6ulclj2".to_string(),
                None,
                TransactionType::StakeRewards,
                TransactionState::Confirmed,
                BigUint::from(7500u64),
                Chain::Cosmos.as_asset_id(),
                BigUint::from(64842u64),
                Some("Stake via Gem Wallet".to_string()),
                None,
                DateTime::parse_rfc3339("2026-11-23T01:31:37Z").unwrap(),
            ))
        );
    }

    #[test]
    fn test_map_transaction_ignored() {
        let mut reverted_delegate = TransactionResponse::mock_delegate();
        reverted_delegate.tx_response.code = 1;
        let mut negative_amount = TransactionResponse::mock_reverted_transfer_spam();
        negative_amount.tx_response.code = 0;

        assert_eq!(map_transaction(CosmosChain::Cosmos, reverted_delegate), None);
        assert_eq!(map_transaction(CosmosChain::Cosmos, TransactionResponse::mock_reverted_transfer_spam()), None);
        assert_eq!(map_transaction(CosmosChain::Cosmos, negative_amount), None);
    }

    #[test]
    fn test_map_transaction_unsupported_messages() {
        let [withdraw, undelegate]: [Message<TransactionResponseCoin>; 2] = TransactionResponse::mock_unstake().tx.body.messages.try_into().unwrap();
        let delegate = TransactionResponse::mock_delegate().tx.body.messages.remove(0);
        let mut transaction = TransactionResponse::mock_unstake();

        for messages in [vec![withdraw.clone(), undelegate.clone(), undelegate.clone()], vec![withdraw.clone(), Message::Unknown], vec![delegate, withdraw, undelegate]] {
            transaction.tx.body.messages = messages;
            assert_eq!(map_transaction(CosmosChain::Cosmos, transaction.clone()), None);
        }
    }

    #[test]
    fn test_map_transaction_decode() {
        assert_eq!(
            map_transaction_decode(
                "CtQBCo8BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEm8KLWNvc21vczF6ODM1Y2p4Zno3MzU5NXd1bnF0bjNmbHg2dGdscnN5MDV5NjczdxItY29zbW9zMXo4MzVjanhmejczNTk1d3VucXRuM2ZseDZ0Z2xyc3kwNXk2NzN3Gg8KBXVhdG9tEgYxMDAwMDASQGFlY2IyY2UwZDU1YTg0NTVhNzc2YTMzOWU2ODY1MDE2NmE2YWE0NTVjMDVlZmRkZjQ5ZTAxMWI0MjAzYTI1YTASZwpRCkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAjhf6Rbk8v7+0NCc4zugr/adpy2yOQikY1pzi6L/SzH2EgQKAggBGOogEhIKDAoFdWF0b20SAzYxOBChxQcaQOl+CCMVx/uR1/yU0RvPKkUADK3LFwo+zsElulf0M34xP00FnS6/51y4FEgn/ewRJokkxy1mPwPvqmK2FBsFVdY="
            ),
            Some("F09F0730AB6C8C60FBD9252F3844184FF8D463ABE4978937AB7166149F5611FD".to_string())
        );
        assert_eq!(
            map_transaction_decode(
                "CooBCocBCiYvc2VpcHJvdG9jb2wuc2VpY2hhaW4ub3JhY2xlLk1zZ0FnZ3JlZ2F0ZUV4Y2hhbmdlUmF0ZVZvdGVdCjQuMjk2ODI3NTE5ODU5MzYzNDIxdWF0b20sMTIwNjgyLjg1MDMyNTUwOTUyMzkwOTIwMnVidGMsNDQ4NS45NDM1MTE1ODEyOTMxMDg2NDZ1ZXRoLDAuMTcxNzU3NzgyMjY4Nzc3Mzg0dW9zbW8sMC4zMDA2MDc3OTA2MDkyMzExNjF1c2VpLDAuOTk5MjM1MTc2MDUxMDgxMDI4dXVzZGMsMS4wMDA0NzA5MTg5NzYyMDM0Njd1dXNkdBIqc2VpMTRxcmN3bXpwZHN6cTBnZWhmcTYzcmFybTdweHF3eG03eGFyY3hqGjFzZWl2YWxvcGVyMTh0cGRldDIya3B2c3d4YXlla3duNTVyeTByNWFjeDRrYWF1dXBrYg=="
            ),
            None
        );
    }
}
