use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::{AUTHORIZATION_HEADER, build_device_auth_header, device_public_key};
use primitives::Device;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let seed = hex::decode(args.get(1).expect("usage: device_header <seed-hex> header <METHOD> <PATH> [WALLET_ID] | register <BASE_URL> [VERSION]")).unwrap();
    let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    match args.get(2).map(String::as_str) {
        Some("header") => {
            let method = &args[3];
            let path = &args[4];
            let wallet_id = args.get(5).map(String::as_str).unwrap_or("");
            println!("{}", build_device_auth_header(&seed, method, path, wallet_id, &[], timestamp_ms).unwrap());
        }
        Some("register") => {
            let base = &args[3];
            let device = Device {
                id: hex::encode(device_public_key(&seed).unwrap()),
                version: args.get(4).cloned().unwrap_or_else(|| "2.114.60".to_string()),
                ..Device::mock()
            };
            let body = serde_json::to_vec(&device).unwrap();
            let header = build_device_auth_header(&seed, "POST", "/v2/devices", "", &body, timestamp_ms).unwrap();
            let response = reqwest::Client::new()
                .post(format!("{base}/v2/devices"))
                .header(AUTHORIZATION_HEADER, header)
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .unwrap();
            println!("{} {}", response.status(), response.text().await.unwrap());
        }
        _ => panic!("unknown command"),
    }
}
