use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};
use serde::de::DeserializeOwned;

use super::model::{RecordResult, Response};

const STATUS_OK: &str = "ok";

pub struct SnsClient {
    client: ReqwestClient,
}

impl SnsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_address(&self, domain: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.get_result(&format!("/resolve/{domain}")).await
    }

    pub async fn get_record(&self, domain: &str, record: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result: RecordResult = self.get_result(&format!("/record-v2/{domain}/{record}")).await?;
        Ok(result.deserialized)
    }

    async fn get_result<T: DeserializeOwned + Send>(&self, path: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let response: Response<T> = self.client.get(path).await?;
        if response.s != STATUS_OK {
            return Err(format!("SNS request failed with status: {}", response.s).into());
        }
        Ok(response.result)
    }
}
