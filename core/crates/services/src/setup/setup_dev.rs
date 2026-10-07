use std::error::Error;
use std::sync::Arc;

use gem_tracing::info_with_fields;
use settings::Settings;

use super::production::seed_database;
use crate::Services;
use crate::rewards::username_rules;

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let services = Services::new(Arc::new(settings.clone()))?;
    let repository = services.setup_repository();
    repository.run_migrations().await?;
    info_with_fields!("setup_dev", step = "postgres migrations complete");
    seed_database(repository.as_ref()).await?;
    let username_rules = username_rules(&services.config()).await?;
    repository.seed_dev(username_rules).await?;

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}
