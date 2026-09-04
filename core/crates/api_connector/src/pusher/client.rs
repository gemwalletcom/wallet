use super::model::{PushResult, Response};
use super::target::PusherTarget;
use gem_client::{ClientError, ClientExt, ReqwestClient};
use primitives::{GorushNotification, GorushNotifications};

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

    pub async fn push_notifications(&self, notifications: Vec<GorushNotification>) -> Result<PushResult, ClientError> {
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

        let payload = GorushNotifications {
            notifications: notifications.clone(),
        };
        let response: Response = self.client.post(PusherTarget::Push, &payload).await?;
        Ok(PushResult { response, notifications })
    }

    pub async fn is_device_token_valid(&self, token: &str, platform: i32) -> Result<bool, ClientError> {
        let notification = GorushNotification::for_token_validation(token.to_string(), platform);
        let result = self.push_notifications(vec![notification]).await?;

        let has_invalid_token = result.response.logs.iter().any(|log| log.is_device_invalid());
        Ok(!has_invalid_token)
    }

    //Remove in the future
    fn get_topic(&self, platform: i32) -> Option<String> {
        match platform {
            1 => Some(self.topic.clone()), // ios
            2 => None,
            _ => None,
        }
    }
}
