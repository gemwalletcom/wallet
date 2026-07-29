use pricer::PriceClient;
use rocket::State;
use rocket_ws::{Channel, WebSocket};

use crate::devices::auth_config::AuthConfig;
use crate::devices::guard::AuthenticatedDevice;

mod client;
mod price_handler;
mod stream;

#[rocket::get("/health")]
pub fn ws_health(_config: &State<AuthConfig>) -> rocket::http::Status {
    rocket::http::Status::Ok
}

pub use client::StreamObserverConfig;

#[rocket::get("/stream")]
pub async fn ws_stream(ws: WebSocket, auth: AuthenticatedDevice, price_client: &State<PriceClient>, config: &State<StreamObserverConfig>) -> Channel<'static> {
    let price_client = price_client.inner().clone();
    let redis_url = config.redis_url.clone();
    let cacher_client = config.cacher_client.clone();
    let retention = config.retention;
    let history_limit = config.history_limit;
    let device_id = auth.device_row.device_id.clone();

    ws.channel(move |ws_stream| {
        Box::pin(async move {
            let mut observer = client::StreamObserverClient::new(device_id, price_client);
            stream::new_stream(&redis_url, &cacher_client, retention, history_limit, &mut observer, ws_stream).await;
            Ok::<(), rocket_ws::result::Error>(())
        })
    })
}
