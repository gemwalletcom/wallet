use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType, NodeSyncStatus, chain_cosmos::CosmosChain};
use std::error::Error;

pub fn calculate_fee_rates(chain: CosmosChain, base_fee: BigInt) -> Vec<FeeRate> {
    match chain {
        CosmosChain::Thorchain | CosmosChain::Mayachain | CosmosChain::Cosmos | CosmosChain::Noble => {
            vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee))]
        }
        CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei | CosmosChain::Injective => {
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee.clone())),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(&base_fee * BigInt::from(2))),
            ]
        }
    }
}

pub fn map_node_status(latest_block: u64) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
    Ok(NodeSyncStatus::synced(latest_block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_chains_that_order_by_gas_price_offer_a_fast_rate() {
        let priorities = |chain| {
            calculate_fee_rates(chain, BigInt::from(1_000))
                .into_iter()
                .map(|rate| (rate.priority, rate.gas_price_type.gas_price()))
                .collect::<Vec<_>>()
        };

        for chain in [CosmosChain::Osmosis, CosmosChain::Celestia, CosmosChain::Sei, CosmosChain::Injective] {
            assert_eq!(
                priorities(chain),
                vec![(FeePriority::Normal, BigInt::from(1_000)), (FeePriority::Fast, BigInt::from(2_000))]
            );
        }

        for chain in [CosmosChain::Cosmos, CosmosChain::Noble, CosmosChain::Thorchain, CosmosChain::Mayachain] {
            assert_eq!(priorities(chain), vec![(FeePriority::Normal, BigInt::from(1_000))]);
        }
    }

    #[test]
    fn test_map_node_status() {
        let latest_block = 12345678u64;
        let mapped = map_node_status(latest_block).unwrap();

        assert!(mapped.in_sync);
        assert_eq!(mapped.latest_block_number, Some(12345678));
        assert_eq!(mapped.current_block_number, Some(12345678));
    }
}
