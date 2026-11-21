use std::collections::HashMap;
use std::sync::Arc;

use primitives::{Chain, Transaction, TransactionType};

use crate::metrics::parser::ParserMetrics;

pub struct ParserReporter {
    chain: Chain,
    metrics: Arc<ParserMetrics>,
}

impl ParserReporter {
    pub fn new(chain: Chain, metrics: Arc<ParserMetrics>) -> Self {
        Self { chain, metrics }
    }

    pub fn update_state(&self, current_block: i64, latest_block: i64, is_enabled: bool) {
        self.metrics.update_state(self.chain.as_ref(), current_block, latest_block, is_enabled);
    }

    pub fn record_transactions(&self, transactions: &[Transaction]) {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for transaction in transactions {
            let transaction_type = if transaction.transaction_type == TransactionType::Transfer && !transaction.asset_id.is_native() {
                "token_transfer"
            } else {
                transaction.transaction_type.as_ref()
            };
            *counts.entry(transaction_type.to_string()).or_default() += 1;
        }
        let entries: Vec<_> = counts.into_iter().collect();
        self.metrics.record_transactions(self.chain.as_ref(), &entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricsProvider;
    use metrics::MetricsRegistry;
    use num_bigint::BigUint;
    use primitives::AssetId;

    #[test]
    fn test_state_changes_are_visible_on_next_scrape() {
        let metrics = Arc::new(ParserMetrics::new());
        let reporter = ParserReporter::new(Chain::Robinhood, metrics.clone());
        for (current_block, latest_block, is_enabled) in [(100, 200, true), (105, 200, true), (105, 205, false), (110, 205, true)] {
            reporter.update_state(current_block, latest_block, is_enabled);
            let mut registry = MetricsRegistry::new();
            metrics.register(&mut registry);
            let output = registry.encode();
            let state = output
                .lines()
                .filter(|line| line.starts_with("parser_state_current_block{") || line.starts_with("parser_state_latest_block{") || line.starts_with("parser_state_is_enabled{"))
                .collect::<Vec<_>>();
            assert_eq!(
                state,
                vec![
                    format!(r#"parser_state_latest_block{{chain="robinhood"}} {latest_block}"#),
                    format!(r#"parser_state_current_block{{chain="robinhood"}} {current_block}"#),
                    format!(r#"parser_state_is_enabled{{chain="robinhood"}} {}"#, i64::from(is_enabled)),
                ]
            );
        }
    }

    #[test]
    fn records_native_and_token_transfers_separately() {
        let metrics = Arc::new(ParserMetrics::new());
        let reporter = ParserReporter::new(Chain::Ethereum, metrics.clone());
        reporter.record_transactions(&[
            Transaction::mock_with_params(AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, BigUint::from(1u64)),
            Transaction::mock_with_params(AssetId::token(Chain::Ethereum, "0x123"), TransactionType::Transfer, BigUint::from(2u64)),
            Transaction::mock_with_params(AssetId::token(Chain::Ethereum, "0x456"), TransactionType::Swap, BigUint::from(3u64)),
        ]);

        let mut registry = MetricsRegistry::new();
        metrics.register(&mut registry);
        let output = registry.encode();

        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="transfer"} 1"#));
        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="token_transfer"} 1"#));
        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="swap"} 1"#));
    }
}
