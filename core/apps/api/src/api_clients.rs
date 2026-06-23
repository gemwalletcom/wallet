use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};
use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::{ApiClientResource, ApiClientScope, ApiClientsRepository, Database};

use crate::responders::cache_error;

fn error_outcome<T>(req: &Request<'_>, status: Status, message: &str) -> Outcome<T, String> {
    cache_error(req, message);
    Error((status, message.to_string()))
}

pub struct PermissionAdminWrite;
pub struct PermissionChainRead;
pub struct PermissionDeviceRead;
pub struct PermissionDeviceSubscriptionsRead;
pub struct PermissionDeviceTransactionsRead;
pub struct PermissionFiatQuotesRead;

fn api_client_secret<'r>(req: &'r Request<'_>) -> Result<&'r str, Outcome<(), String>> {
    let Some(auth_value) = req.headers().get_one(AUTHORIZATION_HEADER) else {
        return Err(error_outcome(req, Status::Unauthorized, "Missing Authorization header"));
    };

    auth_value
        .strip_prefix(BEARER_PREFIX)
        .filter(|secret| !secret.is_empty())
        .ok_or_else(|| error_outcome(req, Status::Unauthorized, "Invalid authorization format"))
}

async fn authorize_api_client(req: &Request<'_>, scope: ApiClientScope) -> Outcome<(), String> {
    let secret = match api_client_secret(req) {
        Ok(secret) => secret,
        Err(outcome) => return outcome,
    };

    let Success(database) = req.guard::<&rocket::State<Database>>().await else {
        return error_outcome(req, Status::InternalServerError, "Database not available");
    };

    let client = match database
        .api_clients()
        .and_then(|mut client| Ok(client.get_enabled_api_client(secret, scope, ApiClientResource::Global)?))
    {
        Ok(client) => client,
        Err(_) => return error_outcome(req, Status::InternalServerError, "Failed to load API client"),
    };

    if client.is_none() {
        return error_outcome(req, Status::Unauthorized, "Invalid API client");
    }

    Success(())
}

macro_rules! api_client_guard {
    ($guard:ident, $scope:expr) => {
        #[rocket::async_trait]
        impl<'r> FromRequest<'r> for $guard {
            type Error = String;

            async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
                match authorize_api_client(req, $scope).await {
                    Success(()) => Success($guard),
                    Error(error) => Error(error),
                    Outcome::Forward(status) => Outcome::Forward(status),
                }
            }
        }
    };
}

api_client_guard!(PermissionAdminWrite, ApiClientScope::AdminWrite);
api_client_guard!(PermissionChainRead, ApiClientScope::ChainRead);
api_client_guard!(PermissionDeviceRead, ApiClientScope::DevicesRead);
api_client_guard!(PermissionDeviceSubscriptionsRead, ApiClientScope::DevicesSubscriptionsRead);
api_client_guard!(PermissionDeviceTransactionsRead, ApiClientScope::DevicesTransactionsRead);
api_client_guard!(PermissionFiatQuotesRead, ApiClientScope::FiatQuotesRead);

#[cfg(test)]
mod tests {
    use rocket::http::{Header, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket, get, routes};

    use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};

    use super::PermissionDeviceRead;

    #[get("/protected-client")]
    async fn protected_client(_permission: PermissionDeviceRead) -> &'static str {
        "ok"
    }

    fn rocket() -> Rocket<Build> {
        rocket::build().mount("/", routes![protected_client])
    }

    #[rocket::async_test]
    async fn test_api_client_missing_authorization_returns_unauthorized() {
        let client = Client::tracked(rocket()).await.unwrap();

        let response = client.get("/protected-client").dispatch().await;

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_api_client_invalid_authorization_format_returns_unauthorized() {
        let client = Client::tracked(rocket()).await.unwrap();

        let response = client.get("/protected-client").header(Header::new(AUTHORIZATION_HEADER, "Basic secret")).dispatch().await;

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_api_client_without_database_returns_internal_server_error() {
        let client = Client::tracked(rocket()).await.unwrap();

        let response = client
            .get("/protected-client")
            .header(Header::new(AUTHORIZATION_HEADER, format!("{BEARER_PREFIX}secret")))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::InternalServerError);
    }
}
