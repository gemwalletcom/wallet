pub(crate) mod repository;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime};
use config_keys::{ConfigKey, ConfigParamKey, RateLimit, RateLimitKey, RateLimitWindow};
use serde::de::DeserializeOwned;
use storage::DatabaseError;

use self::repository::Repository;

const DEFAULT_TTL_SECONDS: u64 = 60;

fn parse_duration(value: &str) -> Result<Duration, DatabaseError> {
    primitives::parse_duration(value).ok_or_else(|| DatabaseError::Error(format!("Failed to parse duration: {value}")))
}

struct CachedValue {
    value: String,
    expires_at: Instant,
}

pub struct ConfigCacher {
    repository: Arc<dyn Repository>,
    cache: RwLock<HashMap<String, CachedValue>>,
    ttl: Duration,
}

impl ConfigCacher {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self {
            repository,
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
        let value = self.repository.config_value(key).await?;
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
        parse_duration(&self.get_param_value(param).await?)
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
        Ok(self.get_param_value(param).await?.parse()?)
    }

    pub async fn get_param_usize(&self, param: &ConfigParamKey) -> Result<usize, DatabaseError> {
        Ok(self.get_param_value(param).await?.parse()?)
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
        self.repository.set_config_value(key, value.to_string()).await
    }

    fn invalidate(&self, key: &ConfigKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key.as_ref());
        }
    }

    async fn get_param_value(&self, param: &ConfigParamKey) -> Result<String, DatabaseError> {
        let key = param.key();
        if let Some(value) = self.get_cached(&key) {
            return Ok(value);
        }
        let param = *param;
        let value = param_value_or_default(self.repository.config_param_value(param).await, &param)?;
        self.set_cached(key, value.clone());
        Ok(value)
    }
}

fn param_value_or_default(stored: Result<String, DatabaseError>, param: &ConfigParamKey) -> Result<String, DatabaseError> {
    match stored {
        Ok(value) => Ok(value),
        Err(error) if error.is_not_found() => Ok(param.default_value()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use config_keys::{ConfigKey, ConfigParamKey, RateLimitKey, RateLimitWindow};
    use storage::DatabaseError;

    use super::*;
    use crate::testkit::MemoryConfigRepository;

    const PARAM: ConfigParamKey = ConfigParamKey::RateLimit(RateLimitKey::ReferralPerUserLimit, RateLimitWindow::Day);

    fn config(repository: MemoryConfigRepository) -> ConfigCacher {
        ConfigCacher::new(Arc::new(repository))
    }

    #[tokio::test]
    async fn test_get_param_missing_row_uses_default() {
        let config = config(MemoryConfigRepository::new());

        assert_eq!(config.get_param_usize(&PARAM).await.unwrap(), PARAM.default_value().parse::<usize>().unwrap());
    }

    #[tokio::test]
    async fn test_get_param_stored_row() {
        let config = config(MemoryConfigRepository::new().with_value(&PARAM.key(), "7"));

        assert_eq!(config.get_param_usize(&PARAM).await.unwrap(), 7);
    }

    #[tokio::test]
    async fn test_failed_reads_propagate() {
        let config = config(MemoryConfigRepository::unavailable());

        assert!(matches!(config.get_param_usize(&PARAM).await, Err(DatabaseError::ConnectionPool)));
        assert!(matches!(config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await, Err(DatabaseError::ConnectionPool)));
    }

    #[tokio::test]
    async fn test_missing_key_is_not_found() {
        let config = config(MemoryConfigRepository::new().without(ConfigKey::ReferralVerifiedMultiplier.as_ref()));

        assert!(config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await.unwrap_err().is_not_found());
    }

    #[tokio::test]
    async fn test_malformed_value() {
        let config = config(MemoryConfigRepository::new().with_value(ConfigKey::ReferralVerifiedMultiplier.as_ref(), "two"));

        assert!(config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await.is_err());
    }

    #[tokio::test]
    async fn test_set_refreshes_cached_value() {
        let config = config(MemoryConfigRepository::new());

        assert_eq!(config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await.unwrap(), 2);
        config.set(ConfigKey::ReferralVerifiedMultiplier, "4").await.unwrap();
        assert_eq!(config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await.unwrap(), 4);
    }
}
