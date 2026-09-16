use primitives::Version;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};

use super::AuthenticatedDevice;

pub struct VersionParam(pub Version);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VersionParam {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<AuthenticatedDevice>().await {
            Outcome::Success(auth) => match auth.device_row.version.parse::<Version>() {
                Ok(version) => Outcome::Success(Self(version)),
                Err(error) => Outcome::Error((Status::BadRequest, error.to_string())),
            },
            Outcome::Error(error) => Outcome::Error(error),
            Outcome::Forward(status) => Outcome::Forward(status),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use primitives::Device;
    use rocket::local::asynchronous::Client as AsyncClient;
    use storage::models::DeviceRow;

    use super::*;

    #[rocket::async_test]
    async fn test_version_param() {
        let client = AsyncClient::tracked(rocket::build()).await.unwrap();
        for (version, expected) in [
            ("2.114.32", Outcome::Success(Version::new(2, 114, 32))),
            ("3.invalid.0", Outcome::Error((Status::BadRequest, "Invalid version".to_string()))),
        ] {
            let device = Device::mock();
            let request = client.get("/?version=9.0.0");
            request.inner().local_cache(|| {
                Outcome::<AuthenticatedDevice, String>::Success(AuthenticatedDevice {
                    device_row: DeviceRow {
                        id: 1,
                        device_id: device.id,
                        platform: device.platform.into(),
                        platform_store: device.platform_store.into(),
                        token: device.token,
                        locale: device.locale.into(),
                        currency: device.currency.into(),
                        is_push_enabled: device.is_push_enabled,
                        is_price_alerts_enabled: false,
                        version: version.to_string(),
                        subscriptions_version: device.subscriptions_version,
                        os: device.os,
                        model: device.model,
                        updated_at: DateTime::UNIX_EPOCH.naive_utc(),
                        created_at: DateTime::UNIX_EPOCH.naive_utc(),
                    },
                })
            });
            assert_eq!(request.inner().guard::<VersionParam>().await.map(|version| version.0), expected);
        }

        let request = client.get("/");
        let error = (Status::Unauthorized, "Invalid signature".to_string());
        request.inner().local_cache(|| Outcome::<AuthenticatedDevice, String>::Error(error.clone()));
        assert_eq!(request.inner().guard::<VersionParam>().await.map(|version| version.0), Outcome::Error(error));
    }
}
