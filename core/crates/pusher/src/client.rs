use async_trait::async_trait;
use gem_client::{ClientError, ClientExt, ReqwestClient};
use push_notification::{GorushNotification, GorushNotifications};

use crate::PushProvider;
use crate::model::{PushResult, Response};
use crate::target::PusherTarget;

#[derive(Clone, Debug)]
pub struct PusherClient {
    client: ReqwestClient,
    topic: String,
}

impl PusherClient {
    pub fn new(url: String, topic: String) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            topic,
        }
    }

    fn get_topic(&self, platform: i32) -> Option<String> {
        match platform {
            1 => Some(self.topic.clone()),
            2 => None,
            _ => None,
        }
    }
}

#[async_trait]
impl PushProvider for PusherClient {
    async fn push_notifications(&self, notifications: Vec<GorushNotification>) -> Result<PushResult, ClientError> {
        let notifications: Vec<GorushNotification> = notifications
            .into_iter()
            .filter(|n| !n.tokens.is_empty() && n.tokens.iter().all(|t| !t.is_empty()))
            .map(|x| x.clone().with_topic(self.get_topic(x.platform)))
            .collect();

        if notifications.is_empty() {
            return Ok(PushResult {
                response: Response {
                    counts: 0,
                    logs: vec![],
                    success: "ok".to_string(),
                },
                notifications,
            });
        }

        let payload = GorushNotifications { notifications: notifications.clone() };
        let response: Response = self.client.post(PusherTarget::Push, &payload).await?;
        Ok(PushResult { response, notifications })
    }

    async fn is_device_token_valid(&self, token: &str, platform: i32) -> Result<bool, ClientError> {
        let notification = GorushNotification::for_token_validation(token.to_string(), platform);
        let result = self.push_notifications(vec![notification]).await?;

        let has_invalid_token = result.response.logs.iter().any(push_notification::PushErrorLog::is_device_invalid);
        Ok(!has_invalid_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_topic() {
        let client = PusherClient::new("http://localhost".to_string(), "com.gemwallet.ios".to_string());

        assert_eq!(client.get_topic(1), Some("com.gemwallet.ios".to_string()));
        assert_eq!(client.get_topic(2), None);
    }
}
