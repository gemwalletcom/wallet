#![no_main]

use std::sync::{Arc, OnceLock};

use gem_aptos::validate_address;
use gem_client::testkit::MockClient;
use libfuzzer_sys::fuzz_target;
use name_resolver::{NameQuery, NameResolver, providers::aptos::AptosProvider};
use primitives::Chain;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let response = Arc::<[u8]>::from(data);
    let provider = AptosProvider::new(MockClient::new().with_post(move |_, _| Ok(response.to_vec())));

    let resolved = RUNTIME.get_or_init(|| Runtime::new().unwrap()).block_on(provider.resolve(&NameQuery::new("fuzz.apt"), Chain::Aptos)).ok().flatten();
    if let Some(address) = resolved {
        assert!(validate_address(&address), "{address}");
    }
});
