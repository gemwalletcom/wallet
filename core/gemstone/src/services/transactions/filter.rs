use primitives::{Chain, TransactionsFilter, WalletType};

use super::model::{GemChainsFilterSummary, GemTransactionFilter, GemTransactionsFilterSummary, chains_filter_summary};
use super::rules;
use crate::services::empty_state::{GemEmptyState, GemEmptyStateAction, GemEmptyStateKind, screen_empty_state};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionsFilterSession {
    pub chains: Vec<Chain>,
    pub selected_chains: Vec<Chain>,
    pub selected_types: Vec<GemTransactionFilter>,
    pub wallet_type: WalletType,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionsFilterView {
    pub chains_summary: GemChainsFilterSummary,
    pub types_summary: GemTransactionsFilterSummary,
    pub types: Vec<GemTransactionFilter>,
    pub is_filtered: bool,
    pub filter: TransactionsFilter,
    pub empty_state: GemEmptyState,
}

#[uniffi::export]
pub fn new_transactions_filter_session(chains: Vec<Chain>, wallet_type: WalletType) -> GemTransactionsFilterSession {
    GemTransactionsFilterSession {
        chains,
        selected_chains: vec![],
        selected_types: vec![],
        wallet_type,
    }
}

#[uniffi::export]
impl GemTransactionsFilterSession {
    pub fn on_chains(&self, chains: Vec<Chain>) -> Self {
        Self { selected_chains: chains, ..self.clone() }
    }

    pub fn on_types(&self, types: Vec<GemTransactionFilter>) -> Self {
        Self { selected_types: types, ..self.clone() }
    }

    pub fn on_clear(&self) -> Self {
        new_transactions_filter_session(self.chains.clone(), self.wallet_type)
    }

    pub fn view_state(&self) -> GemTransactionsFilterView {
        let is_filtered = !self.selected_chains.is_empty() || !self.selected_types.is_empty();
        let kind = match is_filtered {
            true => GemEmptyStateKind::SearchActivity,
            false => GemEmptyStateKind::Activity,
        };
        GemTransactionsFilterView {
            chains_summary: chains_filter_summary(self.selected_chains.clone()),
            types_summary: types_summary(&self.selected_types),
            types: rules::TRANSACTION_FILTERS.to_vec(),
            is_filtered,
            filter: rules::activity_filters(self.selected_chains.clone(), self.selected_types.clone()),
            empty_state: screen_empty_state(kind, self.wallet_type == WalletType::View, &[GemEmptyStateAction::Buy, GemEmptyStateAction::Receive, GemEmptyStateAction::ClearFilters]),
        }
    }
}

fn types_summary(types: &[GemTransactionFilter]) -> GemTransactionsFilterSummary {
    match types {
        [] => GemTransactionsFilterSummary::All,
        [filter] => GemTransactionsFilterSummary::Filter { filter: *filter },
        selected => GemTransactionsFilterSummary::Count { count: selected.len() as u32 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::empty_state::GemEmptyStateText;

    fn session() -> GemTransactionsFilterSession {
        new_transactions_filter_session(vec![Chain::Bitcoin, Chain::Ethereum], WalletType::Multicoin)
    }

    #[test]
    fn test_a_new_session_reads_the_whole_activity() {
        let view = session().view_state();

        assert!(!view.is_filtered);
        assert_eq!((view.chains_summary, view.types_summary), (GemChainsFilterSummary::All, GemTransactionsFilterSummary::All));
        assert_eq!(view.types, rules::TRANSACTION_FILTERS);
        assert_eq!(view.filter, rules::activity_filters(vec![], vec![]));
        assert_eq!(
            (view.empty_state.title, view.empty_state.actions),
            (GemEmptyStateText::ActivityTitle, vec![GemEmptyStateAction::Buy, GemEmptyStateAction::Receive])
        );
    }

    #[test]
    fn test_each_selection_keeps_the_other_and_reaches_the_query() {
        let filtered = session().on_chains(vec![Chain::Ethereum]).on_types(vec![GemTransactionFilter::Swaps, GemTransactionFilter::Stake]);
        let view = filtered.view_state();

        assert_eq!(filtered.selected_chains, vec![Chain::Ethereum]);
        assert!(view.is_filtered);
        assert_eq!(view.chains_summary, GemChainsFilterSummary::Chain { chain: Chain::Ethereum });
        assert_eq!(view.types_summary, GemTransactionsFilterSummary::Count { count: 2 });
        assert_eq!(view.filter, rules::activity_filters(vec![Chain::Ethereum], vec![GemTransactionFilter::Swaps, GemTransactionFilter::Stake]));
        assert_eq!((view.empty_state.title, view.empty_state.actions), (GemEmptyStateText::SearchActivityTitle, vec![GemEmptyStateAction::ClearFilters]));

        assert_eq!(filtered.on_clear(), session(), "clearing keeps the offered chains and drops both selections");
    }

    #[test]
    fn test_a_type_filter_alone_reads_as_its_one_choice_and_hides_activity() {
        let view = session().on_types(vec![GemTransactionFilter::Swaps]).view_state();

        assert_eq!(view.types_summary, GemTransactionsFilterSummary::Filter { filter: GemTransactionFilter::Swaps });
        assert_eq!(view.empty_state.title, GemEmptyStateText::SearchActivityTitle, "a type filter hides activity just as a chain filter does");
    }

    #[test]
    fn test_a_watch_only_wallet_explains_itself_and_offers_nothing() {
        let view = new_transactions_filter_session(vec![], WalletType::View).view_state();

        assert_eq!((view.empty_state.title, view.empty_state.actions), (GemEmptyStateText::WatchWalletTitle, vec![]));
    }
}
