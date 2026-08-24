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
        for tx in transactions {
            let transaction_type = if tx.transaction_type == TransactionType::Transfer && !tx.asset_id.is_native() {
                "token_transfer"
            } else {
                tx.transaction_type.as_ref()
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
    use primitives::AssetId;

    #[test]
    fn records_native_and_token_transfers_separately() {
        let metrics = Arc::new(ParserMetrics::new());
        let reporter = ParserReporter::new(Chain::Ethereum, metrics.clone());
        reporter.record_transactions(&[
            Transaction::mock_with_params(AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, "1".into()),
            Transaction::mock_with_params(AssetId::token(Chain::Ethereum, "0x123"), TransactionType::Transfer, "2".into()),
            Transaction::mock_with_params(AssetId::token(Chain::Ethereum, "0x456"), TransactionType::Swap, "3".into()),
        ]);

        let mut registry = MetricsRegistry::new();
        metrics.register(registry.registry_mut());
        let output = registry.encode();

        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="transfer"} 1"#));
        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="token_transfer"} 1"#));
        assert!(output.contains(r#"parser_transactions_total{chain="ethereum",transaction_type="swap"} 1"#));
    }
}
