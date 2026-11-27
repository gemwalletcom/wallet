use std::collections::HashMap;
use std::error::Error;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn observed_assets(&self, min_observers: usize, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let subscribers = self.channel_subscribers(&CacheKey::Price("*").key()).await?;
        Ok(most_observed(subscribers, &CacheKey::Price("").key(), min_observers, limit))
    }
}

fn most_observed(subscribers: HashMap<String, usize>, prefix: &str, min_observers: usize, limit: usize) -> Vec<String> {
    let mut observed: Vec<(usize, String)> = subscribers
        .into_iter()
        .filter(|(_, observers)| *observers >= min_observers)
        .filter_map(|(channel, observers)| Some((observers, channel.strip_prefix(prefix)?.to_string())))
        .collect();
    observed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    observed.into_iter().take(limit).map(|(_, asset_id)| asset_id).collect()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn test_most_observed_takes_the_most_watched_first() {
        let subscribers = HashMap::from([
            ("prices:ethereum".to_string(), 40),
            ("prices:bitcoin".to_string(), 90),
            ("prices:tron".to_string(), 2),
            ("prices:solana".to_string(), 40),
            ("prices:sui".to_string(), 1),
        ]);

        assert_eq!(most_observed(subscribers.clone(), "prices:", 2, 3), vec!["bitcoin", "ethereum", "solana"]);
        assert_eq!(most_observed(subscribers, "prices:", 2, 10), vec!["bitcoin", "ethereum", "solana", "tron"]);
    }

    #[tokio::test]
    #[ignore = "requires REDIS_URL pointing at a disposable Redis instance"]
    async fn test_observed_assets_counts_live_subscribers() {
        let redis_url = std::env::var("REDIS_URL").unwrap();
        let client = CacherClient::new(&redis_url).await.unwrap();
        let tag = format!("observed-test-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let watched = [(format!("{tag}-a"), 3), (format!("{tag}-b"), 2), (format!("{tag}-c"), 1)];

        let mut subscribers = Vec::new();
        for (asset_id, observers) in &watched {
            for _ in 0..*observers {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
                let config = redis::AsyncConnectionConfig::new().set_push_sender(sender);
                let mut connection = redis::Client::open(redis_url.as_str()).unwrap().get_multiplexed_async_connection_with_config(&config).await.unwrap();
                connection.subscribe(CacheKey::Price(asset_id).key()).await.unwrap();
                subscribers.push((connection, receiver));
            }
        }

        let observed: Vec<String> = client.observed_assets(2, usize::MAX).await.unwrap().into_iter().filter(|asset_id| asset_id.starts_with(&tag)).collect();

        assert_eq!(observed, vec![format!("{tag}-a"), format!("{tag}-b")]);
    }
}
