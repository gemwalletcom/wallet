use std::error::Error;

use gem_client::ReqwestClient;
use gem_encoding::decode_base64;
use gem_jsonrpc::JsonRpcClient;
use gem_solana::models::{AccountData, ValueResult};
use gem_solana::{Pubkey, SolanaAccountEncoding, SolanaRpc};

use super::model::NameRecord;

pub struct AllDomainsClient {
    client: JsonRpcClient<ReqwestClient>,
}

impl AllDomainsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: JsonRpcClient::new(client),
        }
    }

    pub async fn get_name_record(&self, name_account: &Pubkey) -> Result<Option<NameRecord>, Box<dyn Error + Send + Sync>> {
        let response: ValueResult<Option<AccountData>> = self
            .client
            .request(SolanaRpc::GetAccountInfo(name_account.to_string(), SolanaAccountEncoding::Base64))
            .await?;
        let Some(account) = response.value else {
            return Ok(None);
        };
        let data = account.data.first().ok_or("name account has no data")?;
        Ok(Some(NameRecord::from_account_data(&decode_base64(data)?)?))
    }
}
