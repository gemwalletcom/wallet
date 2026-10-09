use diesel_migrations::MigrationHarness;

use crate::database::MIGRATIONS;
use crate::{DatabaseClient, DatabaseError};

pub trait MigrationsRepository {
    fn update_schema(&mut self) -> Result<(), DatabaseError>;
}

impl MigrationsRepository for DatabaseClient {
    fn update_schema(&mut self) -> Result<(), DatabaseError> {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
        Ok(())
    }
}
