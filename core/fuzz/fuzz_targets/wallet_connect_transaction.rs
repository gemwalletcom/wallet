#![no_main]

use gem_wallet_connect::{WalletConnectRequestHandler, WalletConnectTransactionType, validate_send_transaction};
use libfuzzer_sys::fuzz_target;
use primitives::TransferDataOutputType;

fuzz_target!(|input: (u8, &str)| {
    let (transaction_type, data) = input;
    let output_type = if transaction_type % 2 == 0 { TransferDataOutputType::EncodedTransaction } else { TransferDataOutputType::Signature };
    let transaction_type = match transaction_type / 2 % 5 {
        0 => WalletConnectTransactionType::Ethereum,
        1 => WalletConnectTransactionType::Solana { output_type },
        2 => WalletConnectTransactionType::Sui { output_type },
        3 => WalletConnectTransactionType::Ton { output_type },
        _ => WalletConnectTransactionType::Tron { output_type },
    };
    let _ = validate_send_transaction(&transaction_type, data);
    let _ = WalletConnectRequestHandler::decode_send_transaction(transaction_type, data.to_string());
});
