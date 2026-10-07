use std::cmp::min;

use primitives::{EVMChain, PriorityFeeValue, fee::FeePriority};

use crate::models::fee::EthereumFeeHistory;

use num_bigint::BigInt;
use serde_serializers::bigint_from_hex_str;

pub fn get_fee_history_blocks(chain: EVMChain) -> u64 {
    let block_time = chain.to_chain().block_time();
    min(60 * 1000 / block_time, 15) as u64
}

pub fn get_reward_percentiles(chain: EVMChain) -> [u64; 2] {
    match chain {
        EVMChain::Ethereum => [16, 35],
        _ => [40, 60],
    }
}

pub struct FeeCalculator;

impl Default for FeeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl FeeCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_priority_fees(&self, chain: EVMChain, fee_history: &EthereumFeeHistory, priorities: &[FeePriority]) -> Result<Vec<PriorityFeeValue>, Box<dyn std::error::Error + Sync + Send>> {
        if fee_history.reward.is_empty() {
            return Err("fee_history.reward is empty".into());
        }

        if priorities.len() != fee_history.reward[0].len() {
            return Err("priorities.len() != fee_history.reward[0].len()".into());
        }

        let rewards = &fee_history.reward;
        let min_priority_fee = BigInt::from(chain.min_priority_fee());

        let mut columns: Vec<Vec<BigInt>> = vec![Vec::new(); priorities.len()];
        for row in rewards {
            for (i, hex_fee) in row.iter().enumerate().take(priorities.len()) {
                if let Ok(bn) = bigint_from_hex_str(hex_fee) {
                    columns[i].push(bn);
                }
            }
        }

        let mut result: Vec<PriorityFeeValue> = priorities
            .iter()
            .zip(columns.iter())
            .map(|(&priority, fees)| {
                let value = match chain {
                    EVMChain::Ethereum => median(fees),
                    _ => mean(fees),
                }
                .map_or(min_priority_fee.clone(), |value| value.max(min_priority_fee.clone()));

                PriorityFeeValue { priority, value }
            })
            .collect();

        result.sort_unstable_by(|a, b| a.value.cmp(&b.value));
        let base_fee = fee_history.base_fee_per_gas.last().ok_or("No base fee available")?;
        let normal_fee = result[0].value.clone();
        result.iter_mut().zip(priorities.iter()).for_each(|(fee, &priority)| {
            fee.priority = priority;
            match (chain, priority) {
                (_, FeePriority::Normal) => {}
                (EVMChain::Ethereum, FeePriority::Fast) => fee.value = fee.value.clone().max(&normal_fee + (base_fee + &normal_fee) / 10),
                (_, FeePriority::Fast) => fee.value *= BigInt::from(2),
            }
        });

        Ok(result)
    }
}

fn mean(values: &[BigInt]) -> Option<BigInt> {
    (!values.is_empty()).then(|| values.iter().sum::<BigInt>() / BigInt::from(values.len()))
}

fn median(values: &[BigInt]) -> Option<BigInt> {
    let mut values = values.to_vec();
    values.sort_unstable();
    let middle = values.len() / 2;
    match values.len() {
        0 => None,
        len if len % 2 == 0 => Some((&values[middle - 1] + &values[middle]) / 2),
        _ => Some(values[middle].clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::fee::FeePriority;

    #[test]
    fn test_get_fee_history_blocks() {
        assert!(get_fee_history_blocks(EVMChain::Ethereum) > 0);
        assert!(get_fee_history_blocks(EVMChain::Arbitrum) > 0);
    }

    #[test]
    fn test_get_reward_percentiles() {
        assert_eq!(get_reward_percentiles(EVMChain::Ethereum), [16, 35]);
        assert_eq!(get_reward_percentiles(EVMChain::Arbitrum), [40, 60]);
    }

    #[test]
    fn test_calculate_priority_fees() {
        let calculator = FeeCalculator::new();
        let fee_history = EthereumFeeHistory::mock_ethereum();
        let priorities = [FeePriority::Normal, FeePriority::Fast];

        let result = calculator.calculate_priority_fees(EVMChain::SmartChain, &fee_history, &priorities).unwrap();

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].priority, FeePriority::Normal);
        assert_eq!(result[0].value, BigInt::from(584926205));

        assert_eq!(result[1].priority, FeePriority::Fast);
        assert_eq!(result[1].value, BigInt::from(1924038520));
    }

    #[test]
    fn test_calculate_priority_fees_ethereum() {
        let calculator = FeeCalculator::new();
        let priorities = [FeePriority::Normal, FeePriority::Fast];

        let result = calculator.calculate_priority_fees(EVMChain::Ethereum, &EthereumFeeHistory::mock_ethereum(), &priorities).unwrap();

        assert_eq!(result[0].value, BigInt::from(700_000_000));
        assert_eq!(result[1].value, BigInt::from(1_024_105_347));

        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["0x0".to_string(), "0x0".to_string()]],
            base_fee_per_gas: vec![BigInt::from(100_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0,
        };
        let result = calculator.calculate_priority_fees(EVMChain::Ethereum, &fee_history, &priorities).unwrap();

        assert_eq!(result[0].value, BigInt::from(100_000));
        assert_eq!(result[1].value, BigInt::from(10_110_000));
    }

    #[test]
    fn test_median() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[BigInt::from(5), BigInt::from(1), BigInt::from(3)]), Some(BigInt::from(3)));
        assert_eq!(median(&[BigInt::from(4), BigInt::from(1), BigInt::from(2), BigInt::from(100)]), Some(BigInt::from(3)));
    }

    #[test]
    fn test_calculate_priority_fees_sorts_and_relabels() {
        let calculator = FeeCalculator::new();
        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["0xb2d05e00".to_string(), "0x3b9aca00".to_string()]],
            base_fee_per_gas: vec![BigInt::from(100_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0,
        };
        let priorities = [FeePriority::Normal, FeePriority::Fast];

        let result = calculator.calculate_priority_fees(EVMChain::Arbitrum, &fee_history, &priorities).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].priority, FeePriority::Normal);
        assert_eq!(result[1].priority, FeePriority::Fast);
        assert!(result[0].value < result[1].value);
    }

    #[test]
    fn test_calculate_priority_fees_errors() {
        let calculator = FeeCalculator::new();
        let empty_history = EthereumFeeHistory {
            reward: vec![],
            base_fee_per_gas: vec![],
            gas_used_ratio: vec![],
            oldest_block: 0,
        };

        assert!(calculator.calculate_priority_fees(EVMChain::Ethereum, &empty_history, &[FeePriority::Normal]).is_err());
        assert!(calculator.calculate_priority_fees(EVMChain::Ethereum, &EthereumFeeHistory::mock_ethereum(), &[FeePriority::Normal]).is_err());
    }
}
