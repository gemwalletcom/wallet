use std::error::Error;

use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde::Serialize;
use serde::de::DeserializeOwned;

mod cachers;
mod error;
mod keys;
pub use cachers::*;
pub use error::*;
pub(crate) use keys::CacheKey;

#[derive(Clone)]
pub struct CacherClient {
    connection: ConnectionManager,
}

#[derive(Clone, Copy)]
enum WriteMode {
    Set,
    SetIfAbsent,
    SetAndPublish,
}

impl CacherClient {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self { connection })
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, key: CacheKey<'_>) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let value: Option<String> = self.connection.clone().get(key.key()).await?;
        Ok(value.map(|value| serde_json::from_str(&value)).transpose()?)
    }

    pub(crate) async fn get_many<T: DeserializeOwned>(&self, keys: &[CacheKey<'_>]) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_many_optional(keys).await?.into_iter().flatten().collect())
    }

    pub(crate) async fn get_many_optional<T: DeserializeOwned>(&self, keys: &[CacheKey<'_>]) -> Result<Vec<Option<T>>, Box<dyn Error + Send + Sync>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let values: Vec<Option<String>> = self.connection.clone().mget(keys.iter().map(CacheKey::key).collect::<Vec<_>>()).await?;
        Ok(values.into_iter().map(|value| value.map(|value| serde_json::from_str(&value)).transpose()).collect::<Result<_, _>>()?)
    }

    pub(crate) async fn take<T: DeserializeOwned>(&self, key: CacheKey<'_>) -> Result<T, Box<dyn Error + Send + Sync>> {
        let key = key.key();
        let value: Option<String> = self.connection.clone().get_del(&key).await?;
        match value {
            Some(value) => Ok(serde_json::from_str(&value)?),
            None => Err(Box::new(CacheError::KeyNotFound(key))),
        }
    }

    pub(crate) async fn set<T: Serialize>(&self, key: CacheKey<'_>, value: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_many(&[(key, value)]).await?;
        Ok(())
    }

    pub(crate) async fn set_many<T: Serialize>(&self, entries: &[(CacheKey<'_>, &T)]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = entries.iter().map(|(key, value)| Ok((key.key(), serde_json::to_string(value)?, key.ttl()))).collect::<Result<Vec<_>, serde_json::Error>>()?;
        self.write_values(values, WriteMode::Set).await
    }

    pub(crate) async fn set_many_if_absent<T: Serialize>(&self, entries: &[(CacheKey<'_>, &T)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let values = entries.iter().map(|(key, value)| Ok((key.key(), serde_json::to_string(value)?, key.ttl()))).collect::<Result<Vec<_>, serde_json::Error>>()?;
        self.write_values(values, WriteMode::SetIfAbsent).await?;
        Ok(())
    }

    pub(crate) async fn set_many_and_publish<T: Serialize>(&self, entries: &[(CacheKey<'_>, &T)], ttl_seconds: u64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = entries.iter().map(|(key, value)| Ok((key.key(), serde_json::to_string(value)?, ttl_seconds))).collect::<Result<Vec<_>, serde_json::Error>>()?;
        self.write_values(values, WriteMode::SetAndPublish).await
    }

    pub(crate) async fn set_if_absent(&self, key: CacheKey<'_>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let result: Option<String> = redis::cmd("SET").arg(key.key()).arg(1).arg("NX").arg("EX").arg(key.ttl()).query_async(&mut self.connection.clone()).await?;
        Ok(result.is_some())
    }

    pub(crate) async fn delete(&self, keys: &[CacheKey<'_>]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if keys.is_empty() {
            return Ok(0);
        }
        Ok(self.connection.clone().del(keys.iter().map(CacheKey::key).collect::<Vec<_>>()).await?)
    }

    pub(crate) async fn increment(&self, key: CacheKey<'_>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let name = key.key();
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("INCR").arg(&name);
        pipe.cmd("EXPIRE").arg(&name).arg(key.ttl()).arg("NX");
        let (count, _): (i64, i64) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(count)
    }

    pub(crate) async fn add_to_set(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }
        let name = key.key();
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.cmd("SADD").arg(&name).arg(members);
        pipe.cmd("EXPIRE").arg(&name).arg(key.ttl());
        pipe.cmd("SCARD").arg(&name);
        let (_, _, count): (usize, bool, usize) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(count)
    }

    pub(crate) async fn remove_from_set(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }
        Ok(self.connection.clone().srem(key.key(), members).await?)
    }

    pub(crate) async fn set_members(&self, keys: &[CacheKey<'_>]) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let mut pipe = redis::pipe();
        for key in keys {
            pipe.cmd("SMEMBERS").arg(key.key());
        }
        Ok(pipe.query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn add_to_sorted_set(&self, key: CacheKey<'_>, members: &[(String, f64)]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }
        let name = key.key();
        let mut pipe = redis::pipe();
        pipe.atomic();
        for (member, score) in members {
            pipe.cmd("ZADD").arg(&name).arg(score).arg(member).ignore();
        }
        pipe.cmd("EXPIRE").arg(&name).arg(key.ttl()).ignore();
        pipe.cmd("ZCARD").arg(&name);
        let (count,): (usize,) = pipe.query_async(&mut self.connection.clone()).await?;
        Ok(count)
    }

    pub(crate) async fn remove_from_sorted_set(&self, key: CacheKey<'_>, members: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(0);
        }
        Ok(redis::cmd("ZREM").arg(key.key()).arg(members).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn remove_from_sorted_set_up_to(&self, key: CacheKey<'_>, max_score: f64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZREMRANGEBYSCORE").arg(key.key()).arg("-inf").arg(max_score).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn increment_in_sorted_set(&self, key: CacheKey<'_>, members: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(());
        }
        let name = key.key();
        let mut pipe = redis::pipe();
        for member in members {
            pipe.cmd("ZINCRBY").arg(&name).arg(1).arg(member).ignore();
        }
        pipe.cmd("EXPIRE").arg(&name).arg(key.ttl()).ignore();
        pipe.query_async::<()>(&mut self.connection.clone()).await?;
        Ok(())
    }

    pub(crate) async fn sorted_set(&self, key: CacheKey<'_>) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZRANGE").arg(key.key()).arg(0).arg(-1).arg("WITHSCORES").query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn sorted_set_scores(&self, key: CacheKey<'_>, members: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>> {
        if members.is_empty() {
            return Ok(vec![]);
        }
        Ok(redis::cmd("ZMSCORE").arg(key.key()).arg(members).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn sorted_set_range_by_score(&self, key: CacheKey<'_>, min: f64, max: f64, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZRANGEBYSCORE").arg(key.key()).arg(min).arg(max).arg("LIMIT").arg(0).arg(limit).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn sorted_set_count(&self, key: CacheKey<'_>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(redis::cmd("ZCARD").arg(key.key()).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn take_sorted_set(&self, key: CacheKey<'_>) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        let name = key.key();
        let count: usize = redis::cmd("ZCARD").arg(&name).query_async(&mut self.connection.clone()).await?;
        if count == 0 {
            return Ok(vec![]);
        }
        Ok(redis::cmd("ZPOPMIN").arg(&name).arg(count).query_async(&mut self.connection.clone()).await?)
    }

    pub(crate) async fn publish<T: Serialize>(&self, channel: &str, value: &T) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.connection.clone().publish(channel, serde_json::to_string(value)?).await?)
    }

    async fn write_values(&self, values: Vec<(String, String, u64)>, mode: WriteMode) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if values.is_empty() {
            return Ok(0);
        }
        let mut pipe = redis::pipe();
        for (key, value, ttl_seconds) in &values {
            let command = pipe.cmd("SET");
            command.arg(key).arg(value).arg("EX").arg(ttl_seconds);
            match mode {
                WriteMode::Set => {
                    command.ignore();
                }
                WriteMode::SetIfAbsent => {
                    command.arg("NX").ignore();
                }
                WriteMode::SetAndPublish => {
                    command.ignore();
                    pipe.cmd("PUBLISH").arg(key).arg(value).ignore();
                }
            }
        }
        pipe.query_async::<()>(&mut self.connection.clone()).await?;
        Ok(values.len())
    }
}
