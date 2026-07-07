use std::error::Error;

use async_trait::async_trait;
use gem_tracing::info_with_fields;
use lists::ListsClient;
use streamer::{FetchListPayload, consumer::MessageConsumer};

pub struct FetchListConsumer {
    pub lists_client: ListsClient,
}

#[async_trait]
impl MessageConsumer<FetchListPayload, u32> for FetchListConsumer {
    async fn should_process(&self, _payload: &FetchListPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: FetchListPayload) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let payload_display = payload.to_string();
        let count = match self.lists_client.add_list(payload.id, payload.list_id).await? {
            Some(list) => list.count,
            None => 0,
        };
        info_with_fields!("fetch list", payload = payload_display.as_str(), count = count);
        Ok(count)
    }
}
