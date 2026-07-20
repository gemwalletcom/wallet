use std::{error::Error, io};

use alloy_primitives::{B256, Bytes, LogData, U256, hex};
use alloy_sol_types::SolEvent;

use super::contracts::V3SpokePoolInterface::{FundsDeposited, V3FundsDeposited};

pub struct RelayData {
    pub depositor: B256,
    pub recipient: B256,
    pub exclusive_relayer: B256,
    pub input_token: B256,
    pub output_token: B256,
    pub input_amount: U256,
    pub output_amount: U256,
    pub origin_chain_id: U256,
    pub deposit_id: U256,
    pub fill_deadline: u32,
    pub exclusivity_deadline: u32,
    pub message: Bytes,
}

pub struct Deposit {
    pub destination_chain_id: u64,
    pub relay_data: RelayData,
}

impl Deposit {
    fn from_v3(event: V3FundsDeposited, origin_chain_id: u64) -> Self {
        Self {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: RelayData {
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
                message: event.message,
            },
        }
    }

    fn from_universal(event: FundsDeposited, origin_chain_id: u64) -> Self {
        Self {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: RelayData {
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
                message: event.message,
            },
        }
    }
}

pub fn parse_deposit<'a>(logs: impl IntoIterator<Item = (&'a [String], &'a str)>, origin_chain_id: u64) -> Result<Option<Deposit>, Box<dyn Error + Send + Sync>> {
    for (topics, data) in logs {
        let Some(topic) = topics.first().map(|topic| parse_topic(topic)).transpose()? else {
            continue;
        };
        if topic == V3FundsDeposited::SIGNATURE_HASH {
            let event = V3FundsDeposited::decode_log_data(&alloy_log_data(topics, data)?)?;
            return Ok(Some(Deposit::from_v3(event, origin_chain_id)));
        }
        if topic == FundsDeposited::SIGNATURE_HASH {
            let event = FundsDeposited::decode_log_data(&alloy_log_data(topics, data)?)?;
            return Ok(Some(Deposit::from_universal(event, origin_chain_id)));
        }
    }

    Ok(None)
}

fn parse_topic(topic: &str) -> Result<B256, Box<dyn Error + Send + Sync>> {
    Ok(topic.parse().or_else(|_| format!("0x{topic}").parse())?)
}

fn alloy_log_data(topics: &[String], data: &str) -> Result<LogData, Box<dyn Error + Send + Sync>> {
    let topics = topics.iter().map(|topic| parse_topic(topic)).collect::<Result<Vec<_>, _>>()?;
    let data = Bytes::from(hex::decode(data)?);
    LogData::new(topics, data).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid event topics").into())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, Bytes, U256, hex};
    use alloy_sol_types::SolEvent;

    use super::{V3FundsDeposited, parse_deposit};

    #[test]
    fn test_parse_v3_deposit() {
        let event = V3FundsDeposited {
            inputToken: Address::ZERO,
            outputToken: Address::ZERO,
            inputAmount: U256::from(1_000_000),
            outputAmount: U256::from(999_000),
            destinationChainId: U256::from(8453),
            depositId: 123,
            quoteTimestamp: 1_700_000_000,
            fillDeadline: 1_700_003_600,
            exclusivityDeadline: 0,
            depositor: Address::ZERO,
            recipient: Address::ZERO,
            exclusiveRelayer: Address::ZERO,
            message: Bytes::new(),
        };
        let encoded = event.encode_log_data();
        let (topics, data) = encoded.split();
        let topics = topics.into_iter().map(|topic| hex::encode(topic)).collect::<Vec<_>>();
        let data = hex::encode(data);
        let deposit = parse_deposit([(topics.as_slice(), data.as_str())], 137).unwrap().unwrap();

        assert_eq!(deposit.destination_chain_id, 8453);
        assert_eq!(deposit.relay_data.origin_chain_id, U256::from(137));
        assert_eq!(deposit.relay_data.deposit_id, U256::from(123));
        assert_eq!(deposit.relay_data.input_amount, U256::from(1_000_000));
        assert_eq!(deposit.relay_data.output_amount, U256::from(999_000));
        assert!(parse_deposit([], 137).unwrap().is_none());
    }
}
