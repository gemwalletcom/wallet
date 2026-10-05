use std::sync::Mutex;

use async_trait::async_trait;
use gem_client::ClientError;
use push_notification::{GorushNotification, PushErrorLog};
use pusher::{PushProvider, PushResult, Response};

pub(crate) struct RecordingPushProvider {
    pushed: Mutex<Vec<GorushNotification>>,
    logs: Vec<PushErrorLog>,
    failure: Option<String>,
}

impl RecordingPushProvider {
    pub(crate) fn new(logs: Vec<PushErrorLog>) -> Self {
        Self {
            pushed: Mutex::new(Vec::new()),
            logs,
            failure: None,
        }
    }

    pub(crate) fn failing(message: &str) -> Self {
        Self {
            pushed: Mutex::new(Vec::new()),
            logs: Vec::new(),
            failure: Some(message.to_string()),
        }
    }

    pub(crate) fn pushed(&self) -> Vec<GorushNotification> {
        self.pushed.lock().unwrap().clone()
    }
}

#[async_trait]
impl PushProvider for RecordingPushProvider {
    async fn push_notifications(&self, notifications: Vec<GorushNotification>) -> Result<PushResult, ClientError> {
        if let Some(message) = &self.failure {
            return Err(ClientError::Network(message.clone()));
        }
        self.pushed.lock().unwrap().extend(notifications.clone());
        Ok(PushResult {
            response: Response {
                counts: notifications.len() as i32,
                logs: self.logs.clone(),
                success: "ok".to_string(),
            },
            notifications,
        })
    }

    async fn is_device_token_valid(&self, _token: &str, _platform: i32) -> Result<bool, ClientError> {
        if let Some(message) = &self.failure {
            return Err(ClientError::Network(message.clone()));
        }
        Ok(!self.logs.iter().any(PushErrorLog::is_device_invalid))
    }
}
