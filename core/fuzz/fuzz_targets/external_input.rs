#![no_main]

use gem_evm::{eip712::parse_eip712_json, siwe::SiweMessage};
use gem_solana::siws::SiwsMessage;
use gem_wallet_connect::{SignDigestType, WalletConnectRequestHandler, decode_sign_message};
use libfuzzer_sys::fuzz_target;
use payment::PaymentURLDecoder;
use primitives::{Chain, ChainType, WalletConnectCAIP2, WalletConnectCAIP19, WalletConnectLink, WalletConnectRequest, WalletConnectionMethods};

fuzz_target!(|data: &[u8]| {
    let [selector, variant, chain_index, rest @ ..] = data else {
        return;
    };
    let Ok(input) = std::str::from_utf8(rest) else {
        return;
    };
    let chains = Chain::all();
    let chain = chains[*chain_index as usize % chains.len()];
    match selector % 7 {
        0 => {
            let _ = PaymentURLDecoder::decode(input);
        }
        1 => {
            let _ = WalletConnectLink::from_url(input);
            let _ = WalletConnectCAIP2::parse_account(input.to_string());
            let _ = WalletConnectCAIP2::parse_chain_id(input.to_string());
            let _ = WalletConnectCAIP19::get_asset_id(input);
        }
        2 => wallet_connect_request(*variant, chain, input),
        3 => {
            let _ = decode_sign_message(chain, sign_type(*variant), input.to_string());
        }
        4 => {
            let _ = SiwsMessage::parse(input);
            let _ = SiwsMessage::decode_base58(input);
            let _ = SiweMessage::try_parse(input);
        }
        5 => {
            if let Ok(value) = serde_json::from_str(input) {
                let _ = parse_eip712_json(&value);
            }
        }
        _ => {
            let _ = validate_address(input, chain);
        }
    }
});

fn wallet_connect_request(variant: u8, chain: Chain, params: &str) {
    let methods = WalletConnectionMethods::all();
    let method = &methods[variant as usize % methods.len()];
    let Some(method) = serde_json::to_value(method).ok().and_then(|value| value.as_str().map(str::to_string)) else {
        return;
    };
    let chain_id = WalletConnectCAIP2::get_namespace(chain)
        .zip(WalletConnectCAIP2::get_reference(chain))
        .map(|(namespace, reference)| format!("{namespace}:{reference}"));
    let request = WalletConnectRequest {
        topic: String::new(),
        method,
        params: params.to_string(),
        chain_id,
        domain: "example.com".to_string(),
    };
    let _ = WalletConnectRequestHandler::parse_request(request);
}

fn sign_type(variant: u8) -> SignDigestType {
    match variant % 7 {
        0 => SignDigestType::Eip191,
        1 => SignDigestType::Eip712,
        2 => SignDigestType::Base58,
        3 => SignDigestType::SuiPersonal,
        4 => SignDigestType::Siwe,
        5 => SignDigestType::TonPersonal,
        _ => SignDigestType::TronPersonal,
    }
}

fn validate_address(address: &str, chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => gem_evm::validate_address(address),
        ChainType::Solana => gem_solana::validate_address(address),
        ChainType::Cosmos => gem_cosmos::validate_address(address, chain),
        ChainType::Ton => gem_ton::validate_address(address),
        ChainType::Tron => gem_tron::validate_address(address),
        ChainType::Aptos => gem_aptos::validate_address(address),
        ChainType::Sui => gem_sui::validate_address(address),
        ChainType::Near => gem_near::is_valid_address(address),
        ChainType::Stellar => gem_stellar::validate_address(address),
        ChainType::Algorand => gem_algorand::validate_address(address),
        ChainType::Xrp => gem_xrp::validate_address(address),
        ChainType::Polkadot => gem_polkadot::validate_address(address),
        ChainType::Bitcoin => gem_bitcoin::validate_address(address, chain),
        ChainType::Cardano => gem_cardano::validate_address(address),
    }
}
