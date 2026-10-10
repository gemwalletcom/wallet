use std::{
    error::Error,
    future::Future,
    time::{Duration, SystemTime},
};

use futures::future::BoxFuture;
use gem_tracing::warn_with_fields;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{CacheKey, CacherClient};

pub type CacheFuture<'a, T> = BoxFuture<'a, Result<T, Box<dyn Error + Send + Sync>>>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cached<T> {
    pub(crate) value: T,
    updated_at: SystemTime,
}

impl<T> Cached<T> {
    fn is_fresh(&self, duration: Duration) -> bool {
        self.updated_at.elapsed().is_ok_and(|elapsed| elapsed < duration)
    }
}

impl CacherClient {
    pub(crate) async fn get_or_fetch<T, F>(&self, key: CacheKey<'_>, duration: Duration, fetch: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Serialize,
        F: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        if let Some(cached) = self.get::<Cached<T>>(key).await?
            && cached.is_fresh(duration)
        {
            return Ok(cached.value);
        }
        let cached = Cached {
            value: fetch.await?,
            updated_at: SystemTime::now(),
        };
        if let Err(error) = self.set(key, &cached).await {
            warn_with_fields!("cache write failed", error = error.as_ref());
        }
        Ok(cached.value)
    }

    pub(crate) async fn get_or_fetch_with_fallback<T, F>(&self, key: CacheKey<'_>, duration: Duration, fetch: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Serialize,
        F: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        match self.get_or_fetch(key, duration, fetch).await {
            Ok(value) => Ok(value),
            Err(error) => {
                if let Some(cached) = self.get::<Cached<T>>(key).await? {
                    warn_with_fields!("cache refresh failed, serving previous value", error = error.as_ref());
                    return Ok(cached.value);
                }
                Err(error)
            }
        }
    }
}
