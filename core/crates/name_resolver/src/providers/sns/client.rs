use std::error::Error;

use gem_client::{Client, ClientError, ClientExt};
use serde::de::DeserializeOwned;

use super::model::{RecordResult, Response};
use super::target::SnsTarget;

const STATUS_OK: &str = "ok";

pub struct SnsClient<C> {
    client: C,
}

impl<C: Client> SnsClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_address(&self, domain: &str) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        self.get_result(SnsTarget::Resolve { domain: domain.to_string() }).await
    }

    pub async fn get_record(&self, domain: &str, record: &str) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let result: Option<RecordResult> = self
            .get_result(SnsTarget::Record {
                domain: domain.to_string(),
                record: record.to_string(),
            })
            .await?;
        Ok(result.map(|result| result.deserialized))
    }

    async fn get_result<T: DeserializeOwned + Send>(&self, target: SnsTarget) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let response: Response<T> = match self.client.get(target).await {
            Ok(response) => response,
            Err(ClientError::Http { status: 404, .. }) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if response.s != STATUS_OK {
            return Err(format!("SNS request failed with status: {}", response.s).into());
        }
        Ok(Some(response.result))
    }
}
