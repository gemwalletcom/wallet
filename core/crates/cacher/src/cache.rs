use std::{
    error::Error,
    future::Future,
    time::{Duration, SystemTime},
};

use gem_tracing::warn_with_fields;
use serde::de::{DeserializeOwned, IgnoredAny};
use serde::{Deserialize, Serialize};

use crate::{CacheKey, CacherClient};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cached<T> {
    value: T,
    updated_at: SystemTime,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StoredValue<T> {
    Current(Cached<T>),
    Outdated(IgnoredAny),
}

impl<T> Cached<T> {
    fn is_fresh(&self, duration: Duration) -> bool {
        self.updated_at.elapsed().is_ok_and(|elapsed| elapsed < duration)
    }
}

impl<T> StoredValue<T> {
    fn cached(self) -> Option<Cached<T>> {
        match self {
            Self::Current(cached) => Some(cached),
            Self::Outdated(_) => None,
        }
    }
}

impl CacherClient {
    pub(crate) async fn get_cached<T: DeserializeOwned>(&self, key: CacheKey<'_>) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        Ok(self.get::<StoredValue<T>>(key).await?.and_then(StoredValue::cached).map(|cached| cached.value))
    }

    pub(crate) async fn set_cached<T: Serialize>(&self, key: CacheKey<'_>, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(key, &Cached { value, updated_at: SystemTime::now() }).await
    }

    pub(crate) async fn get_many_cached<T: DeserializeOwned>(&self, keys: &[CacheKey<'_>]) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_many::<StoredValue<T>>(keys).await?.into_iter().filter_map(StoredValue::cached).map(|cached| cached.value).collect())
    }

    pub(crate) async fn get_or_set_with_fallback<T, F>(&self, key: CacheKey<'_>, duration: Duration, request: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Serialize,
        F: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let previous = match self.get::<StoredValue<T>>(key).await?.and_then(StoredValue::cached) {
            Some(cached) if cached.is_fresh(duration) => return Ok(cached.value),
            previous => previous,
        };
        match request.await {
            Ok(value) => {
                let cached = Cached { value, updated_at: SystemTime::now() };
                if let Err(error) = self.set(key, &cached).await {
                    warn_with_fields!("cache write failed", error = error.as_ref());
                }
                Ok(cached.value)
            }
            Err(error) => match previous {
                Some(previous) => {
                    warn_with_fields!("cache refresh failed, serving previous value", error = error.as_ref());
                    Ok(previous.value)
                }
                None => Err(error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;

    #[test]
    fn test_stored_value_in_an_older_shape_reads_as_missing() {
        let current = serde_json::to_string(&Cached { value: 5u32, updated_at: UNIX_EPOCH }).unwrap();

        assert_eq!(serde_json::from_str::<StoredValue<u32>>(&current).unwrap().cached().map(|cached| cached.value), Some(5));
        assert!(serde_json::from_str::<StoredValue<u32>>("5").unwrap().cached().is_none());
    }
}
