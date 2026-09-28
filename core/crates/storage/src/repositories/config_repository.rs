use config_keys::{ConfigKey, ConfigParamKey};
use diesel::prelude::*;

use crate::models::ConfigRow;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

pub trait ConfigRepository {
    fn get_config(&mut self, key: ConfigKey) -> Result<String, DatabaseError>;
    fn get_config_param(&mut self, key: ConfigParamKey) -> Result<String, DatabaseError>;
    fn get_config_keys(&mut self) -> Result<Vec<String>, DatabaseError>;
    fn add_config_keys(&mut self, keys: Vec<ConfigKey>) -> Result<usize, DatabaseError>;
    fn add_config_params(&mut self, keys: Vec<ConfigParamKey>) -> Result<usize, DatabaseError>;
    fn set_config(&mut self, key: ConfigKey, value: &str) -> Result<usize, DatabaseError>;
    fn delete_keys(&mut self, keys: Vec<String>) -> Result<usize, DatabaseError>;
}

fn add_config_rows(client: &mut DatabaseClient, configs: Vec<ConfigRow>) -> Result<usize, diesel::result::Error> {
    use crate::schema::config::dsl::*;
    diesel::insert_into(config)
        .values(&configs)
        .on_conflict(key)
        .do_update()
        .set((
            value.eq(diesel::dsl::case_when(value.eq(default_value), diesel::upsert::excluded(value)).otherwise(value)),
            default_value.eq(diesel::upsert::excluded(default_value)),
        ))
        .execute(&mut client.connection)
}

fn config_row(client: &mut DatabaseClient, config_key: &str) -> Result<ConfigRow, diesel::result::Error> {
    use crate::schema::config::dsl::*;
    config.filter(key.eq(config_key)).select(ConfigRow::as_select()).first(&mut client.connection)
}

impl ConfigRepository for DatabaseClient {
    fn get_config(&mut self, key: ConfigKey) -> Result<String, DatabaseError> {
        let key = key.as_ref().to_string();
        let result = config_row(self, &key).or_not_found(key)?;
        Ok(result.value)
    }

    fn get_config_param(&mut self, key: ConfigParamKey) -> Result<String, DatabaseError> {
        let key = key.key();
        let result = config_row(self, &key).or_not_found(key)?;
        Ok(result.value)
    }

    fn add_config_keys(&mut self, keys: Vec<ConfigKey>) -> Result<usize, DatabaseError> {
        Ok(add_config_rows(self, keys.into_iter().map(ConfigRow::from_primitive).collect())?)
    }

    fn add_config_params(&mut self, keys: Vec<ConfigParamKey>) -> Result<usize, DatabaseError> {
        Ok(add_config_rows(self, keys.into_iter().map(ConfigRow::from_param).collect())?)
    }

    fn set_config(&mut self, config_key: ConfigKey, config_value: &str) -> Result<usize, DatabaseError> {
        use crate::schema::config::dsl::*;
        Ok(diesel::update(config.filter(key.eq(config_key.as_ref()))).set(value.eq(config_value)).execute(&mut self.connection)?)
    }

    fn get_config_keys(&mut self) -> Result<Vec<String>, DatabaseError> {
        use crate::schema::config::dsl::*;
        Ok(config.select(key).load(&mut self.connection)?)
    }

    fn delete_keys(&mut self, keys: Vec<String>) -> Result<usize, DatabaseError> {
        use crate::schema::config::dsl::*;
        Ok(diesel::delete(config.filter(key.eq_any(keys))).execute(&mut self.connection)?)
    }
}
