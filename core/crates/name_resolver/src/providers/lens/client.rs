use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};
use serde_json::json;

use super::model::{Data, Record};

const NAMESPACE_ADDRESS: &str = "0x1aA55B9042f08f45825dC4b651B64c9F98Af4615";

pub struct LensClient {
    client: ReqwestClient,
}

impl LensClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_username(&self, local_name: &str) -> Result<Record, Box<dyn Error + Send + Sync>> {
        let query = format!("query {{ username(request: {{ username: {{ localName: \"{local_name}\", namespace: \"{NAMESPACE_ADDRESS}\" }} }}) {{ linkedTo }} }}");
        let response: Data<Record> = self.client.post("", &json!({ "query": query })).await?;
        Ok(response.data)
    }
}
