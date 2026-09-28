use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime};
use config_keys::{ConfigKey, ConfigParamKey, RateLimit, RateLimitKey, RateLimitWindow};
use serde::de::DeserializeOwned;
use storage::{ConfigRepository, Database, DatabaseError};

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

    pub async fn get(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        let cache_key = key.as_ref().to_string();
        if let Some(value) = self.get_cached(&cache_key) {
            return Ok(value);
        }
        let value = self.database.run(move |client| client.get_config(key)).await?;
        self.set_cached(cache_key, value.clone());
        Ok(value)
    }

    pub async fn get_i64(&self, key: ConfigKey) -> Result<i64, DatabaseError> {
        Ok(self.get(key).await?.parse()?)
    }

    pub async fn get_usize(&self, key: ConfigKey) -> Result<usize, DatabaseError> {
        Ok(self.get(key).await?.parse()?)
    }

    pub async fn get_f64(&self, key: ConfigKey) -> Result<f64, DatabaseError> {
        Ok(self.get(key).await?.parse()?)
    }

    pub async fn get_bool(&self, key: ConfigKey) -> Result<bool, DatabaseError> {
        Ok(self.get(key).await?.parse()?)
    }

    pub async fn get_duration(&self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        parse_duration(&self.get(key).await?)
    }

    pub async fn get_param_duration(&self, param: &ConfigParamKey) -> Result<Duration, DatabaseError> {
        parse_duration(&self.get_param_value(param).await)
    }

    pub async fn get_param_durations<T>(&self, values: impl IntoIterator<Item = T>, key: impl Fn(T) -> ConfigParamKey) -> Result<HashMap<T, Duration>, DatabaseError>
    where
        T: Copy + Eq + Hash,
    {
        let mut durations = HashMap::new();
        for value in values {
            durations.insert(value, self.get_param_duration(&key(value)).await?);
        }
        Ok(durations)
    }

    pub async fn get_param_bool(&self, param: &ConfigParamKey) -> Result<bool, DatabaseError> {
        Ok(self.get_param_value(param).await.parse()?)
    }

    pub async fn get_param_usize(&self, param: &ConfigParamKey) -> Result<usize, DatabaseError> {
        Ok(self.get_param_value(param).await.parse()?)
    }

    pub async fn get_rate_limit(&self, key: RateLimitKey) -> Result<RateLimit, DatabaseError> {
        Ok(RateLimit::new(
            self.get_rate_limit_window(key, RateLimitWindow::Minute).await?,
            self.get_rate_limit_window(key, RateLimitWindow::Hour).await?,
            self.get_rate_limit_window(key, RateLimitWindow::Day).await?,
            self.get_rate_limit_window(key, RateLimitWindow::Week).await?,
        ))
    }

    async fn get_rate_limit_window(&self, key: RateLimitKey, window: RateLimitWindow) -> Result<i64, DatabaseError> {
        Ok(self.get_param_usize(&ConfigParamKey::RateLimit(key, window)).await? as i64)
    }

    pub async fn get_datetime(&self, key: ConfigKey) -> Result<NaiveDateTime, DatabaseError> {
        let ts = self.get_i64(key).await?;
        DateTime::from_timestamp(ts, 0).map(|dt| dt.naive_utc()).ok_or_else(|| DatabaseError::Error(format!("Invalid timestamp: {}", ts)))
    }

    pub async fn set_datetime(&self, key: ConfigKey, time: NaiveDateTime) -> Result<usize, DatabaseError> {
        let ts = time.and_utc().timestamp();
        self.set(key, &ts.to_string()).await
    }

    pub async fn get_vec_string(&self, key: ConfigKey) -> Result<Vec<String>, DatabaseError> {
        self.get_vec(key).await
    }

    pub async fn get_vec<T: DeserializeOwned>(&self, key: ConfigKey) -> Result<Vec<T>, DatabaseError> {
        self.get_json(key).await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: ConfigKey) -> Result<T, DatabaseError> {
        Ok(serde_json::from_str(&self.get(key).await?)?)
    }

    pub async fn set(&self, key: ConfigKey, value: &str) -> Result<usize, DatabaseError> {
        self.invalidate(&key);
        let value = value.to_string();
        self.database.run(move |client| client.set_config(key, &value)).await
    }

    fn invalidate(&self, key: &ConfigKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key.as_ref());
        }
    }

    async fn get_param_value(&self, param: &ConfigParamKey) -> String {
        let key = param.key();
        if let Some(value) = self.get_cached(&key) {
            return value;
        }
        let param = *param;
        let stored = self.database.run(move |client| Ok::<_, DatabaseError>(client.get_config_param(param).ok())).await.ok().flatten();
        let value = stored.unwrap_or_else(|| param.default_value());
        self.set_cached(key, value.clone());
        value
    }
}
