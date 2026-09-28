use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

use crate::DatabaseError;

pub fn create_pool(database_url: &str, pool_size: u32) -> Result<PgPool, DatabaseError> {
    if pool_size == 0 {
        return Err(DatabaseError::ConnectionPool);
    }
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(pool_size).build(manager).map_err(|_| DatabaseError::ConnectionPool)
}

pub struct DatabaseClient {
    pub(crate) connection: PgPooledConnection,
}

impl DatabaseClient {
    pub fn from_pool(pool: &PgPool) -> Result<Self, DatabaseError> {
        let connection = pool.get().map_err(|_| DatabaseError::ConnectionPool)?;
        Ok(Self { connection })
    }

    pub(crate) fn transaction<T, E: From<DatabaseError>>(&mut self, operation: impl FnOnce(&mut Self) -> Result<T, E>) -> Result<T, E> {
        AnsiTransactionManager::begin_transaction(&mut *self.connection).map_err(DatabaseError::from)?;
        match operation(self) {
            Ok(value) => {
                AnsiTransactionManager::commit_transaction(&mut *self.connection).map_err(DatabaseError::from)?;
                Ok(value)
            }
            Err(error) => {
                AnsiTransactionManager::rollback_transaction(&mut *self.connection).map_err(DatabaseError::from)?;
                Err(error)
            }
        }
    }
}
