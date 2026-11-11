#![no_main]

use gem_wallet_connect::{WalletConnectRequestHandler, WalletConnectTransactionType, validate_send_transaction};
use libfuzzer_sys::fuzz_target;
use primitives::{Chain, TransferDataOutputType};

fuzz_target!(|input: (u8, &str)| {
    let (transaction_type, data) = input;
    let output_type = if transaction_type % 2 == 0 { TransferDataOutputType::EncodedTransaction } else { TransferDataOutputType::Signature };
    let (chain, transaction_type) = match transaction_type / 2 % 5 {
        0 => (Chain::Ethereum, WalletConnectTransactionType::Ethereum),
        1 => (Chain::Solana, WalletConnectTransactionType::Solana { output_type }),
        2 => (Chain::Sui, WalletConnectTransactionType::Sui { output_type }),
        3 => (Chain::Ton, WalletConnectTransactionType::Ton { output_type }),
        _ => (Chain::Tron, WalletConnectTransactionType::Tron { output_type }),
    };
    let _ = validate_send_transaction(&transaction_type, data);
    let _ = WalletConnectRequestHandler::decode_send_transaction(chain, transaction_type, data.to_string());
    let _ = gem_tron::decode_wallet_connect_contract(data);
    let _ = gem_evm::transaction::decode_nft_collection_approval_operator(data.as_bytes());
    let _ = gem_sui::tx_builder::validate_wallet_connect_sender(data, "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
});
