use primitives::unix_timestamp;
use std::collections::HashMap;
use std::sync::Mutex;

use metrics::{Family, Gauge, LabelEncoder, LabelSet, MetricsRegistry};

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ParserLabels {
    chain: String,
}

impl LabelSet for ParserLabels {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        encoder.add("chain", &self.chain);
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct TransactionTypeLabels {
    chain: String,
    transaction_type: String,
}

impl LabelSet for TransactionTypeLabels {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        encoder.add("chain", &self.chain);
        encoder.add("transaction_type", &self.transaction_type);
    }
}

#[derive(Default)]
struct ParserState {
    current_block: i64,
    latest_block: i64,
    is_enabled: bool,
    updated_at: i64,
    transactions: HashMap<String, u64>,
}

pub struct ParserMetrics {
    chains: Mutex<HashMap<String, ParserState>>,
}

impl ParserMetrics {
    pub fn new() -> Self {
        Self { chains: Mutex::new(HashMap::new()) }
    }

    pub fn update_state(&self, chain: &str, current_block: i64, latest_block: i64, is_enabled: bool) {
        let mut chains = super::locked(&self.chains);
        let state = chains.entry(chain.to_string()).or_default();
        state.current_block = current_block;
        state.latest_block = latest_block;
        state.is_enabled = is_enabled;
        state.updated_at = unix_timestamp() as i64;
    }

    pub fn record_transactions(&self, chain: &str, transactions: &[(String, u64)]) {
        let mut chains = super::locked(&self.chains);
        let state = chains.entry(chain.to_string()).or_default();
        for (transaction_type, count) in transactions {
            *state.transactions.entry(transaction_type.clone()).or_default() += count;
        }
    }
}

impl MetricsProvider for ParserMetrics {
    fn register(&self, registry: &mut MetricsRegistry) {
        let latest_block = Family::<ParserLabels, Gauge>::default();
        let current_block = Family::<ParserLabels, Gauge>::default();
        let is_enabled = Family::<ParserLabels, Gauge>::default();
        let updated_at = Family::<ParserLabels, Gauge>::default();
        let transactions = Family::<TransactionTypeLabels, Gauge>::default();

        let chains = super::locked(&self.chains);
        for (chain, state) in chains.iter() {
            let labels = ParserLabels { chain: chain.clone() };

            current_block.get_or_create(&labels).set(state.current_block);
            latest_block.get_or_create(&labels).set(state.latest_block);
            is_enabled.get_or_create(&labels).set(state.is_enabled as i64);
            updated_at.get_or_create(&labels).set(state.updated_at);

            for (transaction_type, count) in &state.transactions {
                let type_labels = TransactionTypeLabels {
                    chain: chain.clone(),
                    transaction_type: transaction_type.clone(),
                };
                transactions.get_or_create(&type_labels).set(*count as i64);
            }
        }

        registry.register("parser_state_latest_block", "Parser latest block", latest_block);
        registry.register("parser_state_current_block", "Parser current block", current_block);
        registry.register("parser_state_is_enabled", "Parser is enabled", is_enabled);
        registry.register("parser_state_updated_at", "Parser updated at", updated_at);
        registry.register("parser_transactions_total", "Transactions parsed total", transactions);
    }
}
