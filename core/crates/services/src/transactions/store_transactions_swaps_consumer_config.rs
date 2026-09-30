use std::error::Error;

use config_keys::ConfigKey;

use crate::config::ConfigCacher;

pub struct StoreTransactionsSwapsConsumerConfig {
    pub max_output_to_input_value: f64,
    pub max_input_to_output_value: f64,
}

impl StoreTransactionsSwapsConsumerConfig {
    pub async fn read(config: &ConfigCacher) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            max_output_to_input_value: config.get_f64(ConfigKey::TransactionsSwapMaxOutputToInputValue).await?,
            max_input_to_output_value: config.get_f64(ConfigKey::TransactionsSwapMaxInputToOutputValue).await?,
        })
    }
}
