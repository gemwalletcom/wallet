use std::error::Error;
use std::sync::Arc;

use gem_tracing::info_with_fields;
use settings::Settings;

use super::production::seed_database;
use crate::Services;
use crate::rewards::username_rules;

const DEV_USERNAME: &str = "gemcoder";

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let services = Services::new(Arc::new(settings.clone()))?;
    let repository = services.setup_repository();
    repository.update_schema().await?;
    info_with_fields!("setup_dev", step = "postgres migrations complete");
    seed_database(repository.as_ref()).await?;
    let wallet_id = repository.set_dev_seed().await?;

    info_with_fields!("setup_dev", step = "add rewards");
    let username_rules = username_rules(&services.config()).await?;
    match services.rewards_repository().add_username(wallet_id, DEV_USERNAME.to_string(), username_rules).await {
        Ok(Ok(_)) => info_with_fields!("setup_dev", step = "rewards added", username = DEV_USERNAME),
        Ok(Err(error)) => info_with_fields!("setup_dev", step = "rewards skipped (may already exist)", error = error.to_string()),
        Err(error) => info_with_fields!("setup_dev", step = "rewards skipped (may already exist)", error = error.to_string()),
    }

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}
