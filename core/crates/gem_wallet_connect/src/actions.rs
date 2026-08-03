use primitives::{Chain, SignDigestType, SignableTransactionType};

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectAction {
    SignMessage {
        chain: Chain,
        sign_type: SignDigestType,
        data: String,
    },
    SignTransaction {
        chain: Chain,
        transaction_type: SignableTransactionType,
        data: String,
    },
    SignAllTransactions {
        chain: Chain,
        transaction_type: SignableTransactionType,
        transactions: Vec<String>,
    },
    SendTransaction {
        chain: Chain,
        transaction_type: SignableTransactionType,
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
pub enum WalletConnectChainOperation {
    AddChain,
    SwitchChain { chain: Chain },
    GetChainId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectResponseType {
    String { value: String },
    Object { json: String },
}
