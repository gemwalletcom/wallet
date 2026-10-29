#![no_main]

use std::sync::{Arc, OnceLock};

use gem_client::testkit::MockClient;
use libfuzzer_sys::fuzz_target;
use name_resolver::{NameQuery, NameResolver, providers::ens::EnsProvider};
use primitives::Chain;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 || data.len() > 65536 {
        return;
    }
    let split = usize::from(u16::from_le_bytes([data[0], data[1]]));
    let Some(response) = data[2..].get(..split) else {
        return;
    };
    let response = Arc::<[u8]>::from(response);
    let ccip_response = Arc::<[u8]>::from(&data[2 + split..]);
    let get_response = ccip_response.clone();
    let client = MockClient::new().with_post(move |_, _| Ok(response.to_vec()));
    let ccip = MockClient::new().with_get(move |_| Ok(get_response.to_vec())).with_post(move |_, _| Ok(ccip_response.to_vec()));
    let provider = EnsProvider::new(client, ccip, "https://ccip-v3.ens.xyz".to_string());

    let _ = RUNTIME.get_or_init(|| Runtime::new().unwrap()).block_on(provider.resolve(&NameQuery::new("fuzz.eth"), Chain::Ethereum));
});
