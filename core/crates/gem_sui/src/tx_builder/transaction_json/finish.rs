use std::collections::HashSet;

use futures::try_join;
use num_traits::ToPrimitive;
use sui_transaction_builder::ObjectInput;
use sui_types::Address;

use super::{
    builder::digest,
    model::{GasData, ObjectRef, TransactionBuilderJson, TransactionInput},
    replay::{TransactionJsonReplay, parse_transaction_json, prepare_replay},
};
use crate::{
    ESTIMATION_GAS_BUDGET, SUI_COIN_TYPE, SuiClient, SuiError,
    address::SuiAddress,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    models::{Coin, TxOutput},
    tx_builder::{TransactionBuilderInput, finish_transaction},
};

pub async fn finish_transaction_json(client: &SuiClient, transaction_json: &str, sender: &str) -> Result<TxOutput, SuiError> {
    let transaction = parse_transaction_json(transaction_json)?;
    validate_sender(&transaction, sender)?;
    finish_parsed(client, transaction, sender).await
}

pub async fn finish_transaction_json_from_sender(client: &SuiClient, transaction_json: &str) -> Result<TxOutput, SuiError> {
    let transaction = parse_transaction_json(transaction_json)?;
    let sender = transaction.sender.clone().ok_or_else(|| SuiError::invalid_input("Missing Sui transaction sender"))?;
    finish_parsed(client, transaction, &sender).await
}

async fn finish_parsed(client: &SuiClient, transaction: TransactionBuilderJson, sender: &str) -> Result<TxOutput, SuiError> {
    validate_expiration(&transaction)?;

    let gas_data = transaction.gas_data.clone().unwrap_or_default();
    let replay = prepare_replay(client, transaction).await?;
    let (gas_price, gas_objects) = try_join!(gas_price(client, &gas_data), gas_objects(client, &replay, &gas_data, sender))?;

    let builder_input = TransactionBuilderInput::new(sender, gas_price, ESTIMATION_GAS_BUDGET, gas_objects);
    let budget = match gas_data.budget {
        Some(budget) => budget,
        None => estimate_gas_budget(client, &replay, &builder_input).await?,
    };

    finish_transaction(replay.replay()?.txb, builder_input.with_gas_budget(budget))
}

fn validate_sender(transaction: &TransactionBuilderJson, sender: &str) -> Result<(), SuiError> {
    let Some(transaction_sender) = transaction.sender.as_deref() else {
        return Ok(());
    };
    if Address::from(SuiAddress::parse(transaction_sender)?) != Address::from(SuiAddress::parse(sender)?) {
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
        None => client
            .get_gas_price()
            .await
            .map_err(SuiError::from_display)?
            .to_u64()
            .ok_or_else(|| SuiError::invalid_input("Sui gas price overflow")),
    }
}

async fn gas_objects(client: &SuiClient, replay: &TransactionJsonReplay, gas_data: &GasData, sender: &str) -> Result<Vec<ObjectInput>, SuiError> {
    if let Some(payment) = gas_data.payment.as_deref()
        && !payment.is_empty()
    {
        return payment.iter().map(payment_object).collect();
    }

    let coins = client.get_coins(sender, SUI_COIN_TYPE).await.map_err(SuiError::from_display)?;
    let input_objects = input_object_ids(&replay.transaction.inputs)?;
    let gas_objects = coins
        .coins
        .iter()
        .filter(|coin| !input_objects.contains(&coin.object.object_id))
        .map(Coin::to_input)
        .collect::<Vec<_>>();
    if gas_objects.is_empty() {
        return Err(SuiError::NoGasCoins);
    }
    Ok(gas_objects)
}

fn payment_object(object: &ObjectRef) -> Result<ObjectInput, SuiError> {
    Ok(ObjectInput::owned(SuiAddress::parse(&object.object_id)?.into(), object.version, digest(&object.digest)?))
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
    let dry_run = client
        .dry_run(estimate.base64_encoded())
        .await
        .map_err(|err| SuiError::invalid_input(format!("Sui gas estimation failed: {err}")))?;
    let fee = dry_run.effects.gas_used.calculate_gas_budget().map_err(SuiError::from_display)?;
    Ok(fee * GAS_BUDGET_MULTIPLIER / 100)
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
    use crate::provider::testkit::{TEST_ADDRESS, create_sui_test_client};
    use gem_encoding::encode_base64;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_finish_transaction_json() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let recipient = encode_base64(&bcs::to_bytes(&Address::from_str(TEST_ADDRESS)?)?);
        let transaction_json = serde_json::json!({
            "version": 2,
            "sender": TEST_ADDRESS,
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

        let output = finish_transaction_json(&client, &transaction_json, TEST_ADDRESS).await?;

        let transaction: sui_types::Transaction = bcs::from_bytes(&output.tx_data)?;
        assert_eq!(transaction.sender.to_string(), TEST_ADDRESS);
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

        let transaction: sui_types::Transaction = bcs::from_bytes(&output.tx_data)?;
        assert_eq!(transaction.sender.to_string(), sender);

        // Dry run depends on the sender's live address balance, so only resolution and build are asserted.
        let dry_run = client.dry_run(output.base64_encoded()).await;
        println!("dry run result: {dry_run:?}");

        Ok(())
    }
}
