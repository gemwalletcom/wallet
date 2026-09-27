use primitives::Chain;

use super::model::{GemAssetFilter, GemSelectAssetFlow};
use crate::services::transactions::model::{GemChainsFilterSummary, chains_filter_summary};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetsFilterSession {
    pub filters: Vec<GemAssetFilter>,
    pub balance_filter: bool,
    pub selected_chains: Vec<Chain>,
    pub has_balance: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetsFilterView {
    pub selected_chains: Vec<Chain>,
    pub chains_summary: GemChainsFilterSummary,
    pub has_balance: bool,
    pub shows_balance_toggle: bool,
    pub is_filtered: bool,
    pub filters: Vec<GemAssetFilter>,
}

#[uniffi::export]
impl GemSelectAssetFlow {
    pub fn filter_session(&self, chains: Vec<Chain>) -> GemAssetsFilterSession {
        GemAssetsFilterSession {
            filters: self.filters.clone(),
            balance_filter: self.balance_filter,
            selected_chains: chains,
            has_balance: false,
        }
    }
}

#[uniffi::export]
impl GemAssetsFilterSession {
    pub fn on_chains(&self, chains: Vec<Chain>) -> Self {
        Self { selected_chains: chains, ..self.clone() }
    }

    pub fn on_chain_toggled(&self, chain: Chain) -> Self {
        let selected_chains = match self.selected_chains.contains(&chain) {
            true => self.selected_chains.iter().copied().filter(|selected| *selected != chain).collect(),
            false => [self.selected_chains.clone(), vec![chain]].concat(),
        };
        Self { selected_chains, ..self.clone() }
    }

    pub fn on_balance(&self, has_balance: bool) -> Self {
        Self {
            has_balance: has_balance && self.balance_filter,
            ..self.clone()
        }
    }

    pub fn on_clear(&self) -> Self {
        Self {
            selected_chains: vec![],
            has_balance: false,
            ..self.clone()
        }
    }

    pub fn view_state(&self) -> GemAssetsFilterView {
        GemAssetsFilterView {
            selected_chains: self.selected_chains.clone(),
            chains_summary: chains_filter_summary(self.selected_chains.clone()),
            has_balance: self.has_balance,
            shows_balance_toggle: self.balance_filter,
            is_filtered: !self.selected_chains.is_empty() || self.has_balance,
            filters: self.query_filters(),
        }
    }
}

impl GemAssetsFilterSession {
    fn query_filters(&self) -> Vec<GemAssetFilter> {
        let chains = (!self.selected_chains.is_empty()).then(|| GemAssetFilter::Chains { chains: self.selected_chains.clone() });
        let balance = self.has_balance.then_some(GemAssetFilter::HasBalance);
        chains.into_iter().chain(balance).fold(self.filters.clone(), |mut filters, filter| {
            if !filters.contains(&filter) {
                filters.push(filter);
            }
            filters
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::assets::GemSelectAssetType;

    #[test]
    fn test_a_new_session_reads_the_flow_filters_only() {
        let flow = GemSelectAssetType::Send.flow();
        let view = flow.filter_session(vec![]).view_state();

        assert!(!view.is_filtered);
        assert_eq!(view.chains_summary, GemChainsFilterSummary::All);
        assert_eq!(view.filters, flow.filters);
    }

    #[test]
    fn test_the_picked_chains_and_the_balance_reach_the_query() {
        let flow = GemSelectAssetType::Manage.flow();
        let chains = vec![Chain::Ethereum, Chain::Solana];
        let view = flow.filter_session(vec![]).on_chains(chains.clone()).on_balance(true).view_state();

        assert!(view.is_filtered);
        assert!(view.shows_balance_toggle);
        assert_eq!(view.chains_summary, GemChainsFilterSummary::Count { count: 2 });
        assert_eq!(view.filters, [flow.filters, vec![GemAssetFilter::Chains { chains }, GemAssetFilter::HasBalance]].concat());
    }

    #[test]
    fn test_the_balance_only_counts_where_the_flow_offers_it() {
        let flow = GemSelectAssetType::Send.flow();
        let view = flow.filter_session(vec![]).on_balance(true).view_state();

        assert!(!flow.balance_filter);
        assert!(!view.has_balance);
        assert!(!view.is_filtered, "a hidden toggle never marks the list as filtered");
        assert_eq!(
            view.filters.iter().filter(|filter| **filter == GemAssetFilter::HasBalance).count(),
            1,
            "a flow that already filters by balance does not add it twice"
        );
    }

    #[test]
    fn test_toggling_a_chain_twice_and_clearing_restore_the_start() {
        let session = GemSelectAssetType::Manage.flow().filter_session(vec![Chain::Bitcoin]);
        let toggled = session.on_chain_toggled(Chain::Ethereum);

        assert_eq!(toggled.selected_chains, vec![Chain::Bitcoin, Chain::Ethereum]);
        assert_eq!(toggled.on_chain_toggled(Chain::Ethereum), session);
        assert_eq!(toggled.on_balance(true).on_clear().view_state(), GemSelectAssetType::Manage.flow().filter_session(vec![]).view_state());
    }
}
