use std::error::Error;

use async_trait::async_trait;
use gem_tracing::{error_with_fields, info_with_fields};
use streamer::SupportWebhookPayload;
use streamer::consumer::MessageConsumer;

use support::{ChatwootWebhookPayload, SupportClient};

pub struct SupportWebhookConsumer {
    support_client: SupportClient,
}

impl SupportWebhookConsumer {
    pub fn new(support_client: SupportClient) -> Self {
        Self { support_client }
    }
}

#[async_trait]
impl MessageConsumer<SupportWebhookPayload, bool> for SupportWebhookConsumer {
    async fn should_process(&self, _payload: &SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let webhook = match serde_json::from_value::<ChatwootWebhookPayload>(payload.data.clone()) {
            Ok(w) => w,
            Err(e) => {
                error_with_fields!("support webhook parsing failed", &e, payload = payload.data.to_string());
                return Ok(true);
            }
        };

        let Some(device_id) = webhook.get_device_id() else {
            info_with_fields!("support webhook missing device_id", event = webhook.event);
            return Ok(true);
        };

        let Some(device) = self.support_client.get_device(&device_id)? else {
            info_with_fields!("support webhook device not found", device_id = device_id);
            return Ok(true);
        };

        match self.support_client.process_webhook(&device, &webhook).await {
            Ok(result) => {
                info_with_fields!(
                    "support webhook processed",
                    device_id = device_id,
                    event = webhook.event,
                    notifications = result.notifications,
                    stream_events = result.stream_events
                );
                Ok(true)
            }
            Err(error) => {
                error_with_fields!("support webhook failed", &*error, device_id = device_id, payload = payload.data.to_string());
                Err(error)
            }
        }
    }
}
