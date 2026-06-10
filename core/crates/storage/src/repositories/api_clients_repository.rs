use crate::database::api_clients::ApiClientsStore;
use crate::models::{ApiClientGrant, ApiClientResource, ApiClientRow, ApiClientScope};
use crate::{DatabaseClient, DatabaseError};

pub trait ApiClientsRepository {
    fn add_api_client_grants(&mut self, values: Vec<ApiClientGrant>) -> Result<usize, DatabaseError>;
    fn get_enabled_api_client(&mut self, secret: &str, scope: ApiClientScope, resource: ApiClientResource) -> Result<Option<ApiClientRow>, DatabaseError>;
}

impl ApiClientsRepository for DatabaseClient {
    fn add_api_client_grants(&mut self, values: Vec<ApiClientGrant>) -> Result<usize, DatabaseError> {
        Ok(ApiClientsStore::add_api_client_grants(self, values)?)
    }

    fn get_enabled_api_client(&mut self, secret: &str, scope: ApiClientScope, resource: ApiClientResource) -> Result<Option<ApiClientRow>, DatabaseError> {
        Ok(ApiClientsStore::get_enabled_api_client(self, secret, scope, resource)?)
    }
}
