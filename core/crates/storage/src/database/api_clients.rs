use diesel::OptionalExtension;
use diesel::prelude::*;
use std::collections::BTreeSet;

use crate::DatabaseClient;
use crate::models::api_client::{NewApiClientRow, NewApiClientScopeRow};
use crate::models::{ApiClientGrant, ApiClientResource, ApiClientRow, ApiClientScope};
use crate::schema::{api_client_scopes, api_clients};

pub trait ApiClientsStore {
    fn add_api_client_grants(&mut self, values: Vec<ApiClientGrant>) -> Result<usize, diesel::result::Error>;
    fn get_enabled_api_client(&mut self, secret: &str, scope: ApiClientScope, resource: ApiClientResource) -> Result<Option<ApiClientRow>, diesel::result::Error>;
}

impl ApiClientsStore for DatabaseClient {
    fn add_api_client_grants(&mut self, values: Vec<ApiClientGrant>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }

        let client_rows = values
            .iter()
            .map(|value| value.client_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|name| NewApiClientRow { name })
            .collect::<Vec<_>>();

        diesel::insert_into(api_clients::table)
            .values(client_rows)
            .on_conflict(api_clients::name)
            .do_nothing()
            .execute(&mut self.connection)?;

        let names = values.iter().map(|value| value.client_name.clone()).collect::<Vec<_>>();
        let clients = api_clients::table
            .filter(api_clients::name.eq_any(names))
            .select(ApiClientRow::as_select())
            .load(&mut self.connection)?;

        let scopes = values
            .into_iter()
            .filter_map(|value| {
                clients.iter().find(|client| client.name == value.client_name).map(|client| NewApiClientScopeRow {
                    client_id: client.id,
                    scope: value.scope,
                    resource: value.resource.as_str().to_string(),
                })
            })
            .collect::<Vec<_>>();

        diesel::insert_into(api_client_scopes::table)
            .values(scopes)
            .on_conflict((api_client_scopes::client_id, api_client_scopes::scope, api_client_scopes::resource))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn get_enabled_api_client(
        &mut self,
        secret_value: &str,
        scope_value: ApiClientScope,
        resource_value: ApiClientResource,
    ) -> Result<Option<ApiClientRow>, diesel::result::Error> {
        api_clients::table
            .inner_join(api_client_scopes::table)
            .filter(api_clients::secret.eq(secret_value))
            .filter(api_clients::enabled.eq(true))
            .filter(api_client_scopes::scope.eq(scope_value))
            .filter(api_client_scopes::resource.eq(resource_value.as_str()))
            .select(ApiClientRow::as_select())
            .first(&mut self.connection)
            .optional()
    }
}
