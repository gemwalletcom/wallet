use std::error::Error;

use gem_tracing::info_with_fields;
use storage::{Database, MigrationsRepository};

pub async fn run_migrations(database: &Database, log_target: &'static str) -> Result<(), Box<dyn Error + Send + Sync>> {
    database.run(MigrationsRepository::run_migrations).await?;
    info_with_fields!(log_target, step = "postgres migrations complete");
    Ok(())
}
