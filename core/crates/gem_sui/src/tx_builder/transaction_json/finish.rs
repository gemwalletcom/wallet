use std::collections::HashSet;

use futures::try_join;

use super::{
    builder::digest,
    is_transaction_json,
    model::{GasData, ObjectRef, TransactionBuilderJson, TransactionInput},
    replay::{TransactionJsonReplay, parse_transaction_json, prepare_replay},
};
use crate::{
    ESTIMATION_GAS_BUDGET, SuiClient, SuiError,
    address::SuiAddress,
    models::{Coin, TxOutput},
    tx_builder::{TransactionBuilderInput, decode_transaction, finish_transaction},
    types::{Address, ObjectInput, Transaction},
};

pub async fn finish_transaction_json(client: &SuiClient, transaction_json: &str, sender: &str) -> Result<TxOutput, SuiError> {
    let transaction = parse_transaction_json(transaction_json)?;
    validate_sender(&transaction, sender)?;
    validate_expiration(&transaction)?;

    let gas_data = transaction.gas_data.clone().unwrap_or_default();
    let replay = prepare_replay(client, transaction).await?;
    let (gas_price, (gas_objects, gas_coin_total)) = try_join!(gas_price(client, &gas_data), gas_payment(client, &replay, &gas_data, sender))?;

    let builder_input = TransactionBuilderInput {
        gas_coin_total,
        ..TransactionBuilderInput::new(sender, gas_price, ESTIMATION_GAS_BUDGET, gas_objects)
    };
    let budget = match gas_data.budget {
        Some(budget) => budget,
        None => estimate_gas_budget(client, &replay, &builder_input).await?,
    };

    finish_transaction(replay.replay()?.txb, builder_input.with_gas_budget(budget))
}

pub fn validate_wallet_connect_sender(transaction: &str, sender: &str) -> Result<(), SuiError> {
    if is_transaction_json(transaction.as_bytes()) {
        return validate_sender(&parse_transaction_json(transaction)?, sender);
    }
    let transaction: Transaction = decode_transaction(transaction).map_err(SuiError::from_display)?;
    validate_sender_address(transaction.sender, sender)
}

fn validate_sender(transaction: &TransactionBuilderJson, sender: &str) -> Result<(), SuiError> {
    let Some(transaction_sender) = transaction.sender.as_deref() else {
        return Ok(());
    };
    validate_sender_address(SuiAddress::parse(transaction_sender)?.into(), sender)
}

fn validate_sender_address(transaction_sender: Address, sender: &str) -> Result<(), SuiError> {
    if transaction_sender != Address::from(SuiAddress::parse(sender)?) {
        return Err(SuiError::invalid_input(format!("Sui transaction sender mismatch: {transaction_sender}")));
    }
    Ok(())
}

fn validate_expiration(transaction: &TransactionBuilderJson) -> Result<(), SuiError> {
    match &transaction.expiration {
        None | Some(serde_json::Value::Null) => Ok(()),
        Some(expiration) => Err(SuiError::invalid_input(format!("Unsupported Sui transaction expiration: {expiration}"))),
    }
}

async fn gas_price(client: &SuiClient, gas_data: &GasData) -> Result<u64, SuiError> {
    match gas_data.price {
        Some(price) => Ok(price),
        None => client.get_gas_price().await.map_err(SuiError::from_display),
    }
}

async fn gas_payment(client: &SuiClient, replay: &TransactionJsonReplay, gas_data: &GasData, sender: &str) -> Result<(Vec<ObjectInput>, Option<u64>), SuiError> {
    if let Some(payment) = gas_data.payment.as_deref()
        && !payment.is_empty()
    {
        return Ok((payment.iter().map(payment_object).collect::<Result<_, _>>()?, None));
    }

    let mut coins = client.get_gas_coins(sender).await.map_err(SuiError::from_display)?;
    let input_objects = input_object_ids(&replay.transaction.inputs)?;
    coins.coins.retain(|coin| !input_objects.contains(&coin.object.object_id));
    if coins.coins.is_empty() {
        return Err(SuiError::NoGasCoins);
    }
    Ok((coins.coins.iter().map(Coin::to_input).collect(), Some(coins.coin_total())))
}

fn payment_object(object: &ObjectRef) -> Result<ObjectInput, SuiError> {
    Ok(ObjectInput::immutable_or_owned(SuiAddress::parse(&object.object_id)?.into(), object.version, digest(&object.digest)?))
}

/// Coin objects already consumed as transaction inputs cannot double as gas payment.
fn input_object_ids(inputs: &[TransactionInput]) -> Result<HashSet<Address>, SuiError> {
    inputs
        .iter()
        .filter_map(|input| match input {
            TransactionInput::Object { object } => Some(object.object_id()),
            TransactionInput::UnresolvedObject { object } => Some(object.object_id.as_str()),
            TransactionInput::Pure { .. } | TransactionInput::FundsWithdrawal { .. } | TransactionInput::UnresolvedPure { .. } => None,
        })
        .map(|object_id| Ok(SuiAddress::parse(object_id)?.into()))
        .collect()
}

async fn estimate_gas_budget(client: &SuiClient, replay: &TransactionJsonReplay, builder_input: &TransactionBuilderInput) -> Result<u64, SuiError> {
    let estimate = finish_transaction(replay.replay()?.txb, builder_input.clone())?;
    let gas = client
        .estimate_gas(&estimate.base64_encoded(), builder_input.gas_coin_total)
        .await
        .map_err(|err| SuiError::invalid_input(format!("Sui gas estimation failed: {err}")))?;
    Ok(gas.budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sender() {
        let transaction = parse_transaction_json(include_str!("../../../testdata/wc_sign_transaction_cetus.json")).unwrap();
        let sender = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1";

        assert!(validate_sender(&transaction, sender).is_ok());
        assert!(validate_sender(&transaction, "0x2cd8382c19e6994f16df204e9b8cddd04bdc486c251de75ac66ac4e48e3e7081").is_err());

        let senderless = TransactionBuilderJson { sender: None, ..transaction };
        assert!(validate_sender(&senderless, sender).is_ok());
    }

    #[test]
    fn test_input_object_ids_excludes_input_coins() {
        let transaction = parse_transaction_json(include_str!("../../../testdata/wc_sign_transaction_cetus.json")).unwrap();

        let input_objects = input_object_ids(&transaction.inputs).unwrap();

        assert_eq!(input_objects.len(), 10);
        let input_coin: Address = SuiAddress::parse("0xee23d0cd34718145602307f4dccb3228334afbefb2a89919b98e97c420361d5a").unwrap().into();
        assert!(input_objects.contains(&input_coin));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_ADDRESS_SMALL_GAS_BALANCE, create_sui_test_client};
    use gem_encoding::encode_base64;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_finish_transaction_json() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for sender in [TEST_ADDRESS, TEST_ADDRESS_SMALL_GAS_BALANCE] {
            assert_finish_transaction_json(sender).await?;
        }
        Ok(())
    }

    async fn assert_finish_transaction_json(sender: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let recipient = encode_base64(&bcs::to_bytes(&Address::from_str(sender)?)?);
        let transaction_json = serde_json::json!({
            "version": 2,
            "sender": sender,
            "expiration": null,
            "gasData": { "budget": null, "price": null, "owner": null, "payment": null },
            "inputs": [
                { "Pure": { "bytes": "6AMAAAAAAAA=" } },
                { "Pure": { "bytes": recipient } },
                { "UnresolvedObject": { "objectId": "0x0000000000000000000000000000000000000000000000000000000000000006" } }
            ],
            "commands": [
                { "SplitCoins": { "coin": { "GasCoin": true }, "amounts": [{ "Input": 0 }] } },
                { "MoveCall": { "package": "0x2", "module": "clock", "function": "timestamp_ms", "typeArguments": [], "arguments": [{ "Input": 2 }] } },
                { "TransferObjects": { "objects": [{ "NestedResult": [0, 0] }], "address": { "Input": 1 } } }
            ]
        })
        .to_string();

        let output = finish_transaction_json(&client, &transaction_json, sender).await?;

        let transaction: Transaction = bcs::from_bytes(&output.tx_data)?;
        assert_eq!(transaction.sender.to_string(), sender);
        assert!(!transaction.gas_payment.objects.is_empty());
        assert!(transaction.gas_payment.budget > 0);
        assert!(transaction.gas_payment.price > 0);

        let dry_run = client.dry_run(output.base64_encoded()).await?;
        assert_eq!(dry_run.effects.status.status, "success", "dry run failed: {:?}", dry_run.effects.status.error);

        Ok(())
    }

    #[tokio::test]
    async fn test_finish_transaction_json_funds_withdrawal() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let transaction_json = include_str!("../../../testdata/wc_sign_transaction_cetus_funds_withdrawal.json");
        let sender = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1";

        let output = finish_transaction_json(&client, transaction_json, sender).await?;

        let transaction: Transaction = bcs::from_bytes(&output.tx_data)?;
        assert_eq!(transaction.sender.to_string(), sender);

        // Dry run depends on the sender's live address balance, so only resolution and build are asserted.
        let dry_run = client.dry_run(output.base64_encoded()).await;
        println!("dry run result: {dry_run:?}");

        Ok(())
    }
}
