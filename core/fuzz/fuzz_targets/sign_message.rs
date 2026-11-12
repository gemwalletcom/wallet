#![no_main]

use gem_wallet_connect::{SignDigestType, decode_sign_message};
use gemstone::message::signer::MessageSigner;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u8, u8, &str)| {
    let (sign_type, chain, data) = input;
    let sign_type = match sign_type % 9 {
        0 => SignDigestType::Eip191,
        1 => SignDigestType::Eip712,
        2 => SignDigestType::Base58,
        3 => SignDigestType::SuiPersonal,
        4 => SignDigestType::Siwe,
        5 => SignDigestType::TonPersonal,
        6 => SignDigestType::TronPersonal,
        7 => SignDigestType::BitcoinEcdsa,
        _ => SignDigestType::BitcoinBip322,
    };
    let Ok(message) = decode_sign_message(core_fuzz::chain(chain), sign_type, data.to_string()) else {
        return;
    };
    let signer = MessageSigner::new(message);
    let _ = signer.preview();
    let _ = signer.plain_preview();
});
