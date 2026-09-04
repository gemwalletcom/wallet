use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};
use serde::de::DeserializeOwned;

use super::model::{RecordResult, Response};
use super::target::SnsTarget;

const STATUS_OK: &str = "ok";

pub struct SnsClient {
    client: ReqwestClient,
}

impl SnsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_address(&self, domain: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.get_result(SnsTarget::Resolve { domain: domain.to_string() }).await
    }

    pub async fn get_record(&self, domain: &str, record: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result: RecordResult = self
            .get_result(SnsTarget::Record {
                domain: domain.to_string(),
                record: record.to_string(),
            })
            .await?;
        Ok(result.deserialized)
    }

    async fn get_result<T: DeserializeOwned + Send>(&self, target: SnsTarget) -> Result<T, Box<dyn Error + Send + Sync>> {
        let response: Response<T> = self.client.get(target).await?;
        if response.s != STATUS_OK {
            return Err(format!("SNS request failed with status: {}", response.s).into());
        }
        Ok(response.result)
    }
}
