use crate::SwapperError;
use alloy_primitives::{B256, Bytes, LogData, U256, hex::decode as HexDecode};
use alloy_sol_types::SolEvent;
use gem_evm::{
    across::contracts::V3SpokePoolInterface::{FundsDeposited, V3FundsDeposited},
    rpc::model::Log,
};
use gem_tron::models::TransactionReceiptData;

pub(super) struct FillStatusRelayData {
    pub(super) depositor: B256,
    pub(super) recipient: B256,
    pub(super) exclusive_relayer: B256,
    pub(super) input_token: B256,
    pub(super) output_token: B256,
    pub(super) input_amount: U256,
    pub(super) output_amount: U256,
    pub(super) origin_chain_id: U256,
    pub(super) deposit_id: U256,
    pub(super) fill_deadline: u32,
    pub(super) exclusivity_deadline: u32,
    pub(super) message: Bytes,
}

pub(super) struct ParsedDeposit {
    pub(super) destination_chain_id: u64,
    pub(super) relay_data: FillStatusRelayData,
}

fn parse_topic(topic: &str) -> Result<B256, SwapperError> {
    topic
        .parse()
        .or_else(|_| format!("0x{topic}").parse())
        .map_err(|error| SwapperError::TransactionError(format!("invalid event topic: {error}")))
}

fn alloy_log_data(topics: &[String], data: &str) -> Result<LogData, SwapperError> {
    let topics = topics.iter().map(|topic| parse_topic(topic)).collect::<Result<Vec<B256>, _>>()?;
    let data = Bytes::from(HexDecode(data)?);
    LogData::new(topics, data).ok_or_else(|| SwapperError::TransactionError("invalid event topics".into()))
}

fn relay_data_from_v3(event: &V3FundsDeposited, origin_chain_id: u64) -> FillStatusRelayData {
    FillStatusRelayData {
        depositor: event.depositor.into_word(),
        recipient: event.recipient.into_word(),
        exclusive_relayer: event.exclusiveRelayer.into_word(),
        input_token: event.inputToken.into_word(),
        output_token: event.outputToken.into_word(),
        input_amount: event.inputAmount,
        output_amount: event.outputAmount,
        origin_chain_id: U256::from(origin_chain_id),
        deposit_id: U256::from(event.depositId),
        fill_deadline: event.fillDeadline,
        exclusivity_deadline: event.exclusivityDeadline,
        message: event.message.clone(),
    }
}

fn relay_data_from_funds(event: &FundsDeposited, origin_chain_id: u64) -> FillStatusRelayData {
    FillStatusRelayData {
        depositor: event.depositor,
        recipient: event.recipient,
        exclusive_relayer: event.exclusiveRelayer,
        input_token: event.inputToken,
        output_token: event.outputToken,
        input_amount: event.inputAmount,
        output_amount: event.outputAmount,
        origin_chain_id: U256::from(origin_chain_id),
        deposit_id: event.depositId,
        fill_deadline: event.fillDeadline,
        exclusivity_deadline: event.exclusivityDeadline,
        message: event.message.clone(),
    }
}

fn parse_deposit_log(topics: &[String], data: &str, receipt_chain_id: u64) -> Result<Option<ParsedDeposit>, SwapperError> {
    let Some(topic) = topics.first().map(|topic| parse_topic(topic)).transpose()? else {
        return Ok(None);
    };

    if topic == V3FundsDeposited::SIGNATURE_HASH {
        let log_data = alloy_log_data(topics, data)?;
        let event = V3FundsDeposited::decode_log_data(&log_data)?;
        return Ok(Some(ParsedDeposit {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: relay_data_from_v3(&event, receipt_chain_id),
        }));
    }

    if topic == FundsDeposited::SIGNATURE_HASH {
        let log_data = alloy_log_data(topics, data)?;
        let event = FundsDeposited::decode_log_data(&log_data)?;
        return Ok(Some(ParsedDeposit {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: relay_data_from_funds(&event, receipt_chain_id),
        }));
    }

    Ok(None)
}

fn first_parsed_deposit<'a>(logs: impl Iterator<Item = (&'a [String], &'a str)>, receipt_chain_id: u64) -> Result<Option<ParsedDeposit>, SwapperError> {
    for (topics, data) in logs {
        if let Some(deposit) = parse_deposit_log(topics, data, receipt_chain_id)? {
            return Ok(Some(deposit));
        }
    }

    Ok(None)
}

pub(super) fn parse_deposit_from_logs(logs: &[Log], receipt_chain_id: u64) -> Result<Option<ParsedDeposit>, SwapperError> {
    first_parsed_deposit(logs.iter().map(|log| (log.topics.as_slice(), log.data.as_str())), receipt_chain_id)
}

pub(super) fn parse_deposit_from_tron_receipt(receipt: &TransactionReceiptData, receipt_chain_id: u64) -> Result<Option<ParsedDeposit>, SwapperError> {
    let logs = receipt.log.as_deref().unwrap_or_default();
    let entries = logs.iter().filter_map(|log| Some((log.topics.as_deref()?, log.data.as_deref()?)));
    first_parsed_deposit(entries, receipt_chain_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::across::{DEFAULT_FILL_TIMEOUT, status};
    use alloy_primitives::{Address, B256, Bytes, FixedBytes, hex::encode_prefixed as HexEncode};
    use alloy_sol_types::SolEvent;
    use gem_evm::{across::deployment::AcrossDeployment, rpc::model::Log};
    use gem_tron::models::{TransactionReceipt, TransactionReceiptData, TronLog};
    use primitives::{Chain, asset_constants::*};
    use std::str::FromStr;

    const TEST_QUOTE_TIMESTAMP: u32 = 1_700_000_000;
    const TEST_FILL_DEADLINE: u32 = TEST_QUOTE_TIMESTAMP + DEFAULT_FILL_TIMEOUT;

    #[test]
    fn test_parse_v3_funds_deposited() {
        let input_amount = U256::from(1_000_000_000_000_000_000u64);
        let output_amount = U256::from(999_000_000_000_000_000u64);
        let log = build_event_log(
            V3FundsDeposited::SIGNATURE_HASH,
            &[42161, 12345],
            ETHEREUM_WETH_TOKEN_ID,
            ARBITRUM_WETH_TOKEN_ID,
            input_amount,
            output_amount,
        );

        let result = parse_deposit_from_logs(&[log], 1).unwrap().unwrap();

        assert_eq!(result.destination_chain_id, 42161);
        let relay_data = result.relay_data;
        assert_eq!(relay_data.origin_chain_id, U256::from(1));
        assert_eq!(relay_data.deposit_id, U256::from(12345));
        assert_eq!(format!("{:#x}", Address::from_word(relay_data.input_token)), ETHEREUM_WETH_TOKEN_ID.to_ascii_lowercase());
        assert_eq!(format!("{:#x}", Address::from_word(relay_data.output_token)), ARBITRUM_WETH_TOKEN_ID.to_ascii_lowercase());
        assert_eq!(relay_data.input_amount, input_amount);
        assert_eq!(relay_data.output_amount, output_amount);
        assert_eq!(relay_data.fill_deadline, TEST_FILL_DEADLINE);
    }

    #[test]
    fn test_parse_new_funds_deposited() {
        let input_amount = U256::from(20_000_000_000_000_000u64);
        let output_amount = U256::from(19_900_000_000_000_000u64);
        let log = build_event_log(
            FundsDeposited::SIGNATURE_HASH,
            &[8453, 5452553],
            ETHEREUM_WETH_TOKEN_ID,
            BASE_WETH_TOKEN_ID,
            input_amount,
            output_amount,
        );

        let result = parse_deposit_from_logs(&[log], 1).unwrap().unwrap();

        assert_eq!(result.relay_data.deposit_id, U256::from(5452553));
        assert_eq!(result.destination_chain_id, 8453);
        assert_eq!(
            format!("{:#x}", Address::from_word(result.relay_data.input_token)),
            ETHEREUM_WETH_TOKEN_ID.to_ascii_lowercase()
        );
        assert_eq!(
            format!("{:#x}", Address::from_word(result.relay_data.output_token)),
            BASE_WETH_TOKEN_ID.to_ascii_lowercase()
        );
        assert_eq!(result.relay_data.input_amount, input_amount);
        assert_eq!(result.relay_data.output_amount, output_amount);
    }

    #[test]
    fn test_parse_no_matching_event() {
        let log = Log {
            address: String::new(),
            topics: vec![format!("{:#x}", B256::ZERO)],
            data: "0x".into(),
            transaction_hash: None,
        };

        assert!(parse_deposit_from_logs(&[log], 1).unwrap().is_none());
        assert!(parse_deposit_from_logs(&[], 1).unwrap().is_none());
    }

    #[test]
    fn test_parse_tron_v3_funds_deposited() {
        let receipt = TransactionReceiptData {
            id: "deed535ac0e95f35b6f26afdfbe437be6ac2e68891ce01bde77a0782f8df87af".into(),
            fee: Some(13_340_800),
            block_number: 83_883_661,
            block_time_stamp: 1_782_320_547_000,
            receipt: TransactionReceipt { result: Some("SUCCESS".into()) },
            log: Some(vec![TronLog {
                topics: Some(vec![
                    "32ed1a409ef04c7b0227189c3a103dc5ac10e775a15b785dcc510201f7c25ad3".into(),
                    "000000000000000000000000000000000000000000000000000000000000a4b1".into(),
                    "00000000000000000000000000000000000000000000000000000000000000a9".into(),
                    "0000000000000000000000003e1451cdb84d440345de6195b0384d1b77aa4eaa".into(),
                ]),
                data: Some("000000000000000000000000a614f803b6fd780986a42c78ec9c7f77e6ded13c000000000000000000000000fd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb900000000000000000000000000000000000000000000000000000000008339c00000000000000000000000000000000000000000000000000000000000831a60000000000000000000000000000000000000000000000000000000006a3c0d93000000000000000000000000000000000000000000000000000000006a3c61f30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000924a9f036260ddd5808007e1aa95f08ed08aa569000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000009edcf9ff72088db8130c2512e5b4d3b5f34ceaf4000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000120000000000000000000000000fd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044a9059cbb0000000000000000000000009edcf9ff72088db8130c2512e5b4d3b5f34ceaf4000000000000000000000000000000000000000000000000000000000082729100000000000000000000000000000000000000000000000000000000000000000000000000000000fd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044a9059cbb0000000000000000000000000d9dab1a248f63b0a48965ba8435e4de7497a3dc000000000000000000000000000000000000000000000000000000000000a7cf00000000000000000000000000000000000000000000000000000000".into()),
            }]),
        };

        let result = parse_deposit_from_tron_receipt(&receipt, u64::from(AcrossDeployment::deployment_by_chain(&Chain::Tron).unwrap().chain_id))
            .unwrap()
            .unwrap();

        assert_eq!(result.destination_chain_id, 42161);
        assert_eq!(result.relay_data.deposit_id, U256::from(169));
        assert_eq!(result.relay_data.input_amount, U256::from(8_600_000));
        assert_eq!(result.relay_data.output_amount, U256::from(8_591_968));
        assert!(!result.relay_data.message.is_empty());

        let metadata = status::swap_metadata(&result).unwrap();
        assert_eq!(metadata.from_asset, TRON_USDT_ASSET_ID.clone());
        assert_eq!(metadata.to_asset, ARBITRUM_USDT_ASSET_ID.clone());
    }

    fn build_event_log(signature: FixedBytes<32>, indexed: &[u64], input_token: &str, output_token: &str, input_amount: U256, output_amount: U256) -> Log {
        let input_token = Address::from_str(input_token).unwrap();
        let output_token = Address::from_str(output_token).unwrap();

        if signature == V3FundsDeposited::SIGNATURE_HASH {
            return rpc_log(V3FundsDeposited {
                inputToken: input_token,
                outputToken: output_token,
                inputAmount: input_amount,
                outputAmount: output_amount,
                destinationChainId: U256::from(indexed[0]),
                depositId: indexed[1] as u32,
                quoteTimestamp: TEST_QUOTE_TIMESTAMP,
                fillDeadline: TEST_FILL_DEADLINE,
                exclusivityDeadline: 0,
                depositor: Address::ZERO,
                recipient: Address::ZERO,
                exclusiveRelayer: Address::ZERO,
                message: Bytes::new(),
            });
        }

        rpc_log(FundsDeposited {
            inputToken: input_token.into_word(),
            outputToken: output_token.into_word(),
            inputAmount: input_amount,
            outputAmount: output_amount,
            destinationChainId: U256::from(indexed[0]),
            depositId: U256::from(indexed[1]),
            quoteTimestamp: TEST_QUOTE_TIMESTAMP,
            fillDeadline: TEST_FILL_DEADLINE,
            exclusivityDeadline: 0,
            depositor: B256::ZERO,
            recipient: B256::ZERO,
            exclusiveRelayer: B256::ZERO,
            message: Bytes::new(),
        })
    }

    fn rpc_log<E: SolEvent>(event: E) -> Log {
        let data = event.encode_log_data();
        let (topics, data) = data.split();

        Log {
            address: String::new(),
            topics: topics.into_iter().map(|topic| format!("{topic:#x}")).collect(),
            data: HexEncode(data.as_ref()),
            transaction_hash: None,
        }
    }
}
