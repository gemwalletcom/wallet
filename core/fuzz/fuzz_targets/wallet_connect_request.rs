#![no_main]

use gem_wallet_connect::WalletConnectRequestHandler;
use libfuzzer_sys::fuzz_target;
use primitives::{WalletConnectCAIP2, WalletConnectRequest, WalletConnectionMethods};

fuzz_target!(|input: (u8, u8, &str)| {
    let (method, chain, params) = input;
    let methods = WalletConnectionMethods::all();
    let Some(method) = serde_json::to_value(&methods[usize::from(method) % methods.len()]).ok().and_then(|value| value.as_str().map(str::to_string)) else {
        return;
    };
    let chain = core_fuzz::chain(chain);
    let _ = WalletConnectRequestHandler::parse_request(WalletConnectRequest {
        topic: String::new(),
        method,
        params: params.to_string(),
        chain_id: WalletConnectCAIP2::get_namespace(chain)
            .zip(WalletConnectCAIP2::get_reference(chain))
            .map(|(namespace, reference)| format!("{namespace}:{reference}")),
        domain: String::new(),
    });
});
