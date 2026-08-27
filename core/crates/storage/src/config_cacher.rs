use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime};
use primitives::{ConfigKey, ConfigParamKey, RateLimit, RateLimitKey, RateLimitWindow};
use serde::de::DeserializeOwned;
use std::hash::Hash;

use crate::database::config::ConfigStore;
use crate::repositories::config_repository::ConfigRepository;
use crate::{Database, DatabaseError};

const DEFAULT_TTL_SECONDS: u64 = 60;

fn parse_duration(value: &str) -> Result<Duration, DatabaseError> {
    primitives::parse_duration(value).ok_or_else(|| DatabaseError::Error(format!("Failed to parse duration: {value}")))
}

struct CachedValue {
    value: String,
    expires_at: Instant,
}

pub struct ConfigCacher {
    database: Database,
    cache: RwLock<HashMap<String, CachedValue>>,
    ttl: Duration,
}

impl ConfigCacher {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            cache: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(DEFAULT_TTL_SECONDS),
        }
    }

    fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().ok()?;
        let cached = cache.get(key)?;
        if cached.expires_at > Instant::now() { Some(cached.value.clone()) } else { None }
    }

    fn set_cached(&self, key: String, value: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                key,
                CachedValue {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
        }
    }

    pub fn get(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        let cache_key = key.as_ref().to_string();
        if let Some(value) = self.get_cached(&cache_key) {
            return Ok(value);
        }
        let value = self.database.client().map_err(|e| DatabaseError::Error(e.to_string()))?.get_config(key)?;
        self.set_cached(cache_key, value.clone());
        Ok(value)
    }

    pub fn get_i64(&self, key: ConfigKey) -> Result<i64, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_usize(&self, key: ConfigKey) -> Result<usize, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_f64(&self, key: ConfigKey) -> Result<f64, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_bool(&self, key: ConfigKey) -> Result<bool, DatabaseError> {
        Ok(self.get(key)?.parse()?)
    }

    pub fn get_duration(&self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        parse_duration(&self.get(key)?)
    }

    pub fn get_param_duration(&self, param: &ConfigParamKey) -> Result<Duration, DatabaseError> {
        parse_duration(&self.get_param_value(param))
    }

    pub fn get_param_durations<T>(&self, values: impl IntoIterator<Item = T>, key: impl Fn(T) -> ConfigParamKey) -> Result<HashMap<T, Duration>, DatabaseError>
    where
        T: Copy + Eq + Hash,
    {
        values
            .into_iter()
            .map(|value| self.get_param_duration(&key(value)).map(|duration| (value, duration)))
            .collect()
    }

    pub fn get_param_usize(&self, param: &ConfigParamKey) -> Result<usize, DatabaseError> {
        Ok(self.get_param_value(param).parse()?)
    }

    pub fn get_rate_limit(&self, key: RateLimitKey) -> Result<RateLimit, DatabaseError> {
        let [minute, hour, day, week] = RateLimitWindow::ALL.map(|window| self.get_param_usize(&ConfigParamKey::RateLimit(key, window)).map(|limit| limit as i64));
        Ok(RateLimit::new(minute?, hour?, day?, week?))
    }

    pub fn get_datetime(&self, key: ConfigKey) -> Result<NaiveDateTime, DatabaseError> {
        let ts = self.get_i64(key)?;
        DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.naive_utc())
            .ok_or_else(|| DatabaseError::Error(format!("Invalid timestamp: {}", ts)))
    }

    pub fn set_datetime(&self, key: ConfigKey, time: NaiveDateTime) -> Result<usize, DatabaseError> {
        let ts = time.and_utc().timestamp();
        self.set(key, &ts.to_string())
    }

    pub fn get_vec_string(&self, key: ConfigKey) -> Result<Vec<String>, DatabaseError> {
        self.get_vec(key)
    }

    pub fn get_vec<T: DeserializeOwned>(&self, key: ConfigKey) -> Result<Vec<T>, DatabaseError> {
        self.get_json(key)
    }

    pub fn get_json<T: DeserializeOwned>(&self, key: ConfigKey) -> Result<T, DatabaseError> {
        Ok(serde_json::from_str(&self.get(key)?)?)
    }

    pub fn set(&self, key: ConfigKey, value: &str) -> Result<usize, DatabaseError> {
        self.invalidate(&key);
        ConfigRepository::set_config(&mut self.database.client().map_err(|e| DatabaseError::Error(e.to_string()))?, key, value)
    }

    pub fn invalidate(&self, key: &ConfigKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key.as_ref());
        }
    }

    fn get_param_value(&self, param: &ConfigParamKey) -> String {
        let key = param.key();
        if let Some(value) = self.get_cached(&key) {
            return value;
        }
        let value = self
            .database
            .client()
            .ok()
            .and_then(|mut client| ConfigStore::get_config_key(&mut client, &key).ok())
            .map_or_else(|| param.default_value(), |row| row.value);
        self.set_cached(key, value.clone());
        value
    }
}
