use gem_tracing::info_with_fields;
use storage::{Database, MigrationsRepository};

pub fn run_migrations(database: &Database, log_target: &'static str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    database.migrations()?.run_migrations()?;
    info_with_fields!(log_target, step = "postgres migrations complete");
    Ok(())
}
