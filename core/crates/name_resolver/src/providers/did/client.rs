use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::{AccountRequest, Data, Record, Records};

pub struct DidClient {
    client: ReqwestClient,
}

impl DidClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_records(&self, account: &str) -> Result<Vec<Record>, Box<dyn Error + Send + Sync>> {
        let response: Data<Records> = self.client.post("/v2/account/records", &AccountRequest { account }).await?;
        Ok(response.data.records)
    }
}
