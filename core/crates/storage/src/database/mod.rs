pub mod api_clients;
pub mod assets;
pub mod assets_addresses;
pub mod assets_associations;
pub mod assets_links;
pub mod assets_usage_ranks;

pub mod chains;
pub mod charts;
pub mod config;
pub mod devices;
pub mod fiat;
pub mod migrations;
pub mod nft;
pub mod notifications;
pub mod parser_state;
pub mod perpetuals;
pub mod price_alerts;
pub mod prices;
pub mod prices_providers;
pub mod referrals;
pub mod releases;
pub mod rewards;
pub mod rewards_redemptions;
pub mod scan_addresses;
pub mod support_sessions;
pub mod tag;
pub mod transactions;
pub mod usernames;
pub mod wallets;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

use crate::{
    DatabaseError,
    repositories::{
    config_repository::ConfigRepository, devices_repository::DevicesRepository, fiat_repository::FiatRepository, nft_repository::NftRepository,
    perpetuals_repository::PerpetualsRepository, rewards_repository::RewardsRepository,
    },
};

pub fn create_pool(database_url: &str, pool_size: u32) -> Result<PgPool, DatabaseError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .map_err(|_| DatabaseError::ConnectionPool)
}

pub struct DatabaseClient {
    connection: PgPooledConnection,
}

impl DatabaseClient {
    pub fn from_pool(pool: &PgPool) -> Result<Self, r2d2::Error> {
        let connection = pool.get()?;
        Ok(Self { connection })
    }

    pub fn config(&mut self) -> &mut dyn ConfigRepository {
        self
    }

    pub fn devices(&mut self) -> &mut dyn DevicesRepository {
        self
    }

    pub fn fiat(&mut self) -> &mut dyn FiatRepository {
        self
    }

    pub fn perpetuals(&mut self) -> &mut dyn PerpetualsRepository {
        self
    }

    pub fn nft(&mut self) -> &mut dyn NftRepository {
        self
    }

    pub fn rewards(&mut self) -> &mut dyn RewardsRepository {
        self
    }
}
