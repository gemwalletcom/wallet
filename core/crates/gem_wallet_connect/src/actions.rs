use crate::sign_type::SignDigestType;
pub use gem_evm::transaction::EvmTransactionKind;
use gem_solana::VersionedTransaction;
use primitives::{Chain, TransferDataOutputType, WCEthereumTransaction};

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectAction {
    SignMessage {
        chain: Chain,
        sign_type: SignDigestType,
        data: String,
    },
    SignTransaction {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
    },
    SignAllTransactions {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        transactions: Vec<String>,
    },
    SendTransaction {
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
    },
    ChainOperation {
        operation: WalletConnectChainOperation,
    },
    GetAccounts {
        chain: Chain,
    },
    Unsupported {
        method: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectTransactionType {
    Ethereum,
    Solana { output_type: TransferDataOutputType },
    Sui { output_type: TransferDataOutputType },
    Ton { output_type: TransferDataOutputType },
    Tron { output_type: TransferDataOutputType },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectChainOperation {
    AddChain { chain_id: u64 },
    SwitchChain { chain_id: u64 },
    GetChainId { chain_id: u64 },
}

#[derive(Debug, Clone)]
pub struct WCSolanaTransactionData {
    pub transaction: String,
    pub decoded: VersionedTransaction,
}

#[derive(Debug, Clone)]
pub struct WCSuiTransactionData {
    pub transaction: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum WalletConnectTransaction {
    Ethereum { data: WCEthereumTransaction, kind: EvmTransactionKind },
    Solana { data: WCSolanaTransactionData, output_type: TransferDataOutputType },
    Sui { data: WCSuiTransactionData, output_type: TransferDataOutputType },
    Ton { data: String, output_type: TransferDataOutputType },
    Tron { data: String, output_type: TransferDataOutputType },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}
