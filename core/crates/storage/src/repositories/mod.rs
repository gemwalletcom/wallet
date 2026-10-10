pub mod api_clients_repository;
pub mod assets_addresses_repository;
pub mod assets_links_repository;
pub mod assets_repository;
pub mod assets_usage_ranks_repository;
pub mod chains_repository;
pub mod charts_repository;
pub mod config_repository;
pub mod devices_repository;
pub mod features_repository;
pub mod fiat_repository;
pub mod migrations_repository;
pub mod nft_repository;
pub mod notifications_repository;
pub mod parser_state_repository;
pub mod perpetuals_repository;
pub mod price_alerts_repository;
pub mod prices_providers_repository;
pub mod prices_repository;
pub mod releases_repository;
pub mod rewards_redemptions_repository;
pub mod rewards_repository;
pub mod risk_signals_repository;
pub mod scan_addresses_repository;
pub mod scan_detections_repository;
pub mod scan_websites_repository;
pub mod support_sessions_repository;
pub mod tag_repository;
pub mod transactions_perpetuals_repository;
pub mod transactions_repository;
pub mod transactions_swaps_repository;
pub mod wallets_repository;

use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::sql_types::Bool;
use diesel::{BoolExpressionMethods, BoxableExpression};

pub(crate) type Condition<T> = Box<dyn BoxableExpression<T, Pg, SqlType = Bool>>;

pub(crate) trait QueryFilter<T> {
    fn condition(self) -> Condition<T>;
}

pub(crate) fn matching<T: 'static, F: QueryFilter<T>>(filters: Vec<F>) -> Condition<T> {
    filters.into_iter().map(QueryFilter::condition).fold(Box::new(sql::<Bool>("TRUE")), |all, condition| Box::new(all.and(condition)))
}
