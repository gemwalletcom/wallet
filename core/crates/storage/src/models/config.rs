use diesel::prelude::*;
use primitives::{ConfigKey, ConfigParamKey};

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::config)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConfigRow {
    pub key: String,
    pub value: String,
    pub default_value: String,
}

impl ConfigRow {
    pub fn from_primitive(key: ConfigKey) -> Self {
        Self {
            key: key.as_ref().to_string(),
            value: key.default_value().to_string(),
            default_value: key.default_value().to_string(),
        }
    }

    pub fn from_param(key: ConfigParamKey) -> Self {
        let default_value = key.default_value();
        Self {
            key: key.key(),
            value: default_value.clone(),
            default_value,
        }
    }
}
