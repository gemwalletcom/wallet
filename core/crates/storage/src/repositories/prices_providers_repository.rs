use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{PriceProvider, PriceProviderConfig};

use crate::models::PriceProviderConfigRow;
use crate::{DatabaseClient, DatabaseError};

pub trait PricesProvidersRepository {
    fn add_prices_providers(&mut self, providers: Vec<PriceProvider>) -> Result<usize, DatabaseError>;
    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfig>, DatabaseError>;
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
        use crate::schema::prices_providers::dsl::*;
        Ok(prices_providers
            .order(priority.asc())
            .select(PriceProviderConfigRow::as_select())
            .load::<PriceProviderConfigRow>(&mut self.connection)?
            .into_iter()
            .map(|row| PriceProviderConfig {
                provider: row.id.0,
                enabled: row.enabled,
                priority: row.priority,
            })
            .collect())
    }
}
