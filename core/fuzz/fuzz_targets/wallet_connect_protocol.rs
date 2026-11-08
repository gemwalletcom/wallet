#![no_main]

use libfuzzer_sys::fuzz_target;
use wallet_connect_protocol::authenticate::{AuthenticateParams, narrowed_payload};
use wallet_connect_protocol::envelope;
use wallet_connect_protocol::relay::parse_frame;
use wallet_connect_protocol::sign::namespaces::{Supported, approved_namespaces};
use wallet_connect_protocol::sign::{Payload, SessionProposal, SessionRequest};
use wallet_connect_protocol::PairingUri;

const KEY: [u8; 32] = [7; 32];

fuzz_target!(|input: &str| {
    let _ = PairingUri::parse(input, 0);
    let _ = parse_frame(input);
    let _ = envelope::decrypt(&KEY, input);
    if let Ok(Payload::Request { params, .. }) = Payload::parse(input.as_bytes()) {
        let _ = serde_json::from_str::<SessionRequest>(params.get());
        if let Ok(proposal) = serde_json::from_str::<SessionProposal>(params.get()) {
            let chains = proposal.optional_chain_ids();
            let accounts: Vec<String> = chains.iter().map(|chain| format!("{chain}:address")).collect();
            let methods = vec!["personal_sign".to_string()];
            let _ = approved_namespaces(&proposal, &Supported { chains: &chains, accounts: &accounts, methods: &methods, events: &[] });
        }
        if let Ok(params) = serde_json::from_str::<AuthenticateParams>(params.get()) {
            let _ = narrowed_payload(&params.auth_payload, &params.auth_payload.chains, &["personal_sign".to_string()]);
        }
    }
});
