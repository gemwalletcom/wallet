use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

use async_trait::async_trait;
use cacher::CheckScheduleCacher;

#[derive(Default)]
pub(crate) struct MemoryCheckScheduleCacher {
    next_checks: Mutex<HashMap<(String, String), f64>>,
    checked_ids: Mutex<Vec<String>>,
}

impl MemoryCheckScheduleCacher {
    pub(crate) fn checked_ids(&self) -> Vec<String> {
        self.checked_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl CheckScheduleCacher for MemoryCheckScheduleCacher {
    async fn next_checks(&self, queue: &str, ids: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>> {
        self.checked_ids.lock().unwrap().extend_from_slice(ids);
        let checks = self.next_checks.lock().unwrap();
        Ok(ids.iter().map(|id| checks.get(&(queue.to_string(), id.clone())).copied()).collect())
    }

    async fn set_next_check(&self, queue: &str, id: String, next_check_at: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.next_checks.lock().unwrap().insert((queue.to_string(), id), next_check_at);
        Ok(())
    }

    async fn remove_check(&self, queue: &str, id: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.next_checks.lock().unwrap().remove(&(queue.to_string(), id));
        Ok(())
    }
}
