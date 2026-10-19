use async_trait::async_trait;
use chain_traits::ChainAccount;
use std::error::Error;

use gem_client::Client;

use crate::provider::accounts_mapper::{GET_PUBLIC_KEY_METHOD, map_public_key};
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainAccount for TonClient<C> {
    async fn get_public_key(&self, address: String) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        map_public_key(&self.run_get_method(&address, GET_PUBLIC_KEY_METHOD, vec![]).await?)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_public_key() {
        let client = TonClient::new(MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/api/v3/runGetMethod");
            assert_eq!(body, br#"{"address":"account","method":"get_public_key","stack":[]}"#);
            Ok(br#"{"gas_used":1021,"exit_code":0,"stack":[{"type":"num","value":"0x7952e94118f34607c75e23258dd9220d66ccac5a3ee074125c25068e8107bfbf"}]}"#.to_vec())
        }));
        assert_eq!(client.get_public_key("account".to_string()).await.unwrap().unwrap(), "7952e94118f34607c75e23258dd9220d66ccac5a3ee074125c25068e8107bfbf");
    }
}
