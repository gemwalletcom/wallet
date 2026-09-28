use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::PriceProvider;

use crate::models::PriceProviderConfigRow;
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceProviderConfig {
    pub provider: PriceProvider,
    pub enabled: bool,
    pub priority: i32,
}

pub trait PricesProvidersRepository {
    fn add_prices_providers(&mut self, providers: Vec<PriceProvider>) -> Result<usize, DatabaseError>;
    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfig>, DatabaseError>;
}

pub(crate) fn price_provider_rows(client: &mut DatabaseClient) -> Result<Vec<PriceProviderConfigRow>, diesel::result::Error> {
    use crate::schema::prices_providers::dsl::*;
    prices_providers.order(priority.asc()).select(PriceProviderConfigRow::as_select()).load(&mut client.connection)
}

impl PricesProvidersRepository for DatabaseClient {
    fn add_prices_providers(&mut self, providers: Vec<PriceProvider>) -> Result<usize, DatabaseError> {
        use crate::schema::prices_providers::dsl::*;
        let values: Vec<PriceProviderConfigRow> = providers.into_iter().map(|provider| PriceProviderConfigRow::new(provider, true)).collect();
        Ok(diesel::insert_into(prices_providers)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((priority.eq(excluded(priority)),))
            .execute(&mut self.connection)?)
    }

    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfig>, DatabaseError> {
        Ok(price_provider_rows(self)?
            .into_iter()
            .map(|row| PriceProviderConfig {
                provider: row.id.0,
                enabled: row.enabled,
                priority: row.priority,
            })
            .collect())
    }
}
