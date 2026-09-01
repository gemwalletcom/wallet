use std::error::Error;

use redis::{AsyncCommands, Client, aio::ConnectionManager};

mod access_token;
mod error;
mod keys;
mod rate_limiter;
pub use access_token::*;
pub use error::*;
pub use keys::*;
pub use rate_limiter::*;

#[derive(Clone)]
pub struct CacherClient {
    connection: ConnectionManager,
}

impl CacherClient {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self { connection })
    }

    pub async fn set_values_with_publish(&self, values: Vec<(String, String)>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = values.into_iter().map(|(key, value)| (key, value, ttl_seconds)).collect();
        self.set_serialized_values_with_ttl_and_publish(values, true).await
    }

    pub async fn set_value_with_ttl(&self, key: &str, value: String, seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_serialized_value(key, value, Some(seconds)).await
    }

    pub async fn set_values_with_ttl<T: serde::Serialize>(&self, values: Vec<(&str, &T)>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = values
            .into_iter()
            .map(|(key, value)| serde_json::to_string(value).map(|serialized| (key.to_string(), serialized, ttl_seconds)))
            .collect::<Result<Vec<_>, _>>()?;
        self.set_serialized_values_with_ttl(values).await
    }

    pub async fn set_value<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_serialized_value(key, serde_json::to_string(value)?, None).await
    }

    pub async fn get_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let value: Option<String> = self.connection.clone().get(key).await?;
        match value {
            Some(s) => Ok(serde_json::from_str(&s)?),
            None => Err(Box::new(CacheError::KeyNotFound(key.to_string()))),
        }
    }

    pub async fn get_and_delete_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let value: Option<String> = self.connection.clone().get_del(key).await?;
        match value {
            Some(serialized) => Ok(serde_json::from_str(&serialized)?),
            None => Err(Box::new(CacheError::KeyNotFound(key.to_string()))),
        }
    }

    pub async fn get_value_optional<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let value: Option<String> = self.connection.clone().get(key).await?;
        match value {
            Some(serialized) => Ok(Some(serde_json::from_str(&serialized)?)),
            None => Ok(None),
        }
    }

    pub async fn get_values<T, I>(&self, keys: Vec<String>) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        I: serde::de::DeserializeOwned,
        T: FromIterator<I>,
    {
        if keys.is_empty() {
            return Ok(std::iter::empty::<I>().collect());
        }
        let result: Vec<Option<String>> = self.connection.clone().mget(keys).await?;
        let values = result.into_iter().flatten().map(|value| serde_json::from_str::<I>(&value)).collect::<Result<T, _>>()?;
        Ok(values)
    }

    pub async fn get_or_set_value<T, F, Fut>(&self, key: &str, fetch_fn: F, ttl_seconds: Option<u64>) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        if let Ok(cached_value) = self.get_value::<T>(key).await {
            return Ok(cached_value);
        }

        let fresh_value = fetch_fn().await?;

        let serialized = serde_json::to_string(&fresh_value)?;
        self.set_serialized_value(key, serialized, ttl_seconds).await?;

        Ok(fresh_value)
    }

    pub async fn can_process_now(&self, key: &str, ttl_seconds: u64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(1)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut self.connection.clone())
            .await?;
        Ok(result.is_some())
    }

    pub async fn delete(&self, key: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.connection.clone().del::<&str, i64>(key).await? > 0)
    }

    pub async fn delete_keys(&self, keys: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if keys.is_empty() {
            return Ok(0);
        }
        Ok(self.connection.clone().del(keys).await?)
    }

    pub async fn increment_with_ttl(&self, key: &str, ttl: i64) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("INCR").arg(key);
        pipe.cmd("EXPIRE").arg(key).arg(ttl).arg("NX");
        let results: (i64, i64) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(results.0)
    }

    // CacheKey-aware methods
    pub async fn set_cached<T: serde::Serialize>(&self, key: CacheKey<'_>, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_values_with_ttl(vec![(&key.key(), value)], key.ttl() as i64).await?;
        Ok(())
    }

    pub async fn get_cached_optional<T: serde::de::DeserializeOwned>(&self, key: CacheKey<'_>) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        self.get_value_optional(&key.key()).await
    }

    pub async fn set_values_cached<T: serde::Serialize>(&self, entries: &[(CacheKey<'_>, &T)]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = entries
            .iter()
            .map(|(key, value)| serde_json::to_string(value).map(|serialized| (key.key(), serialized, key.ttl() as i64)))
            .collect::<Result<Vec<_>, _>>()?;
        self.set_serialized_values_with_ttl(values).await
    }

    pub async fn get_or_set_cached<T, F, Fut>(&self, key: CacheKey<'_>, fetch_fn: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        self.get_or_set_value(&key.key(), fetch_fn, Some(key.ttl())).await
    }

    pub async fn increment_cached(&self, key: CacheKey<'_>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.increment_with_ttl(&key.key(), key.ttl() as i64).await
    }

    pub async fn add_to_set_cached(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }
        let key_str = key.key();
        let ttl = key.ttl() as i64;
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("SADD").arg(&key_str).arg(members);
        pipe.cmd("EXPIRE").arg(&key_str).arg(ttl);
        pipe.cmd("SCARD").arg(&key_str);
        let (_, _, count): (usize, bool, usize) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(count)
    }

    pub async fn remove_from_set_cached(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }

        Ok(self.connection.clone().srem(key.key(), members).await?)
    }

    pub async fn add_to_sorted_set_cached(&self, key: CacheKey<'_>, members: &[(String, f64)]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }

        let key_str = key.key();
        let ttl = key.ttl() as i64;
        let mut pipe = redis::pipe();
        pipe.atomic();
        for (member, score) in members {
            pipe.cmd("ZADD").arg(&key_str).arg(score).arg(member).ignore();
        }
        pipe.cmd("EXPIRE").arg(&key_str).arg(ttl).ignore();
        pipe.cmd("ZCARD").arg(&key_str);
        let (count,): (usize,) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(count)
    }

    pub async fn remove_from_sorted_set_cached(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }

        Ok(redis::cmd("ZREM").arg(key.key()).arg(members).query_async(&mut self.connection.clone()).await?)
    }

    pub async fn get_set_members_cached(&self, keys: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_set_members_grouped(keys).await?.into_iter().flatten().collect())
    }

    pub async fn get_set_members_grouped(&self, keys: Vec<String>) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let mut pipe = redis::pipe();
        for key in &keys {
            pipe.cmd("SMEMBERS").arg(key);
        }
        Ok(pipe.query_async(&mut self.connection.clone()).await?)
    }

    pub async fn can_process_cached(&self, key: CacheKey<'_>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.can_process_now(&key.key(), key.ttl()).await
    }

    pub async fn sorted_set_incr_with_expire(&self, key: &str, members: &[String], ttl: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(());
        }
        let mut pipe = redis::pipe();
        for member in members {
            pipe.cmd("ZINCRBY").arg(key).arg(1).arg(member).ignore();
        }
        pipe.cmd("EXPIRE").arg(key).arg(ttl).ignore();
        pipe.query_async::<()>(&mut self.connection.clone()).await?;
        Ok(())
    }

    pub async fn publish<T: serde::Serialize, R: redis::FromRedisValue>(&self, channel: &str, value: &T) -> Result<R, Box<dyn Error + Send + Sync>> {
        let message = serde_json::to_string(value)?;
        Ok(self.connection.clone().publish(channel, &message).await?)
    }

    pub async fn sorted_set_range_by_score(&self, key: &str, min: f64, max: f64, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZRANGEBYSCORE")
            .arg(key)
            .arg(min)
            .arg(max)
            .arg("LIMIT")
            .arg(0)
            .arg(limit)
            .query_async(&mut self.connection.clone())
            .await?)
    }

    pub async fn sorted_set_card(&self, key: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZCARD").arg(key).query_async(&mut self.connection.clone()).await?)
    }

    pub async fn sorted_set_range_with_scores(&self, key: &str, start: isize, stop: isize) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut self.connection.clone())
            .await?)
    }

    pub async fn take_sorted_set_with_scores(&self, key: &str) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        let count = self.sorted_set_card(key).await?;
        if count == 0 {
            return Ok(vec![]);
        }
        Ok(redis::cmd("ZPOPMIN").arg(key).arg(count).query_async(&mut self.connection.clone()).await?)
    }

    async fn set_serialized_values_with_ttl(&self, values: Vec<(String, String, i64)>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.set_serialized_values_with_ttl_and_publish(values, false).await
    }

    async fn set_serialized_values_with_ttl_and_publish(&self, values: Vec<(String, String, i64)>, publish: bool) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if values.is_empty() {
            return Ok(0);
        }
        let mut pipe = redis::pipe();
        for (key, serialized, ttl_seconds) in &values {
            pipe.cmd("SET").arg(key).arg(serialized).arg("EX").arg(ttl_seconds).ignore();
            if publish {
                pipe.cmd("PUBLISH").arg(key).arg(serialized).ignore();
            }
        }
        pipe.query_async::<()>(&mut self.connection.clone()).await?;
        Ok(values.len())
    }

    async fn set_serialized_value(&self, key: &str, serialized: String, ttl_seconds: Option<u64>) -> Result<(), Box<dyn Error + Send + Sync>> {
        match ttl_seconds {
            Some(ttl) => {
                self.connection.clone().set_ex::<&str, String, ()>(key, serialized, ttl).await?;
            }
            None => {
                self.connection.clone().set::<&str, String, ()>(key, serialized).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    struct TestValue {
        nonce: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires REDIS_URL pointing at a disposable Redis instance"]
    async fn get_and_delete_value_is_atomic_under_concurrency() {
        let redis_url = std::env::var("REDIS_URL").expect("set REDIS_URL to run Redis-backed cache tests");
        let client = CacherClient::new(&redis_url).await.unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let key = format!("test:get-and-delete:{}:{timestamp}", std::process::id());
        let value = TestValue { nonce: "single-use".to_string() };

        let _ = client.delete(&key).await;
        client.set_value(&key, &value).await.unwrap();

        let handles = (0..32)
            .map(|_| {
                let client = client.clone();
                let key = key.clone();
                tokio::spawn(async move { client.get_and_delete_value::<TestValue>(&key).await.ok() })
            })
            .collect::<Vec<_>>();

        let mut consumed_values = Vec::new();
        for handle in handles {
            if let Some(consumed_value) = handle.await.unwrap() {
                consumed_values.push(consumed_value);
            }
        }

        assert_eq!(consumed_values, vec![value]);
        assert!(client.get_value_optional::<TestValue>(&key).await.unwrap().is_none());
    }
}
