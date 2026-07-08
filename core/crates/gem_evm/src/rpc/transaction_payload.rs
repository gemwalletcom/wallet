use num_bigint::BigUint;
use num_traits::Num;

use crate::{
    address::{ethereum_address_checksum, ethereum_address_from_topic},
    rpc::model::{Log, Transaction, TransactionReceipt},
};

pub(crate) const INPUT_0X: &str = "0x";
const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";
const FUNCTION_ERC721_TRANSFER: &str = "0x23b872dd";
const FUNCTION_ERC721_SAFE_TRANSFER: &str = "0x42842e0e";
const FUNCTION_ERC1155_TRANSFER: &str = "0xf242432a";
pub(crate) const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
const APPROVAL_TOPIC: &str = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
const TRANSFER_SINGLE: &str = "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62";
const TRANSFER_GAS_LIMIT: u64 = 21000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogSearch {
    FirstMatch,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NftTransferPayload {
    pub(super) to: String,
    pub(super) contract_address: String,
    pub(super) token_id: BigUint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Erc20TransferPayload {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) contract_address: String,
    pub(super) value: BigUint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Erc20ApprovalPayload {
    pub(super) spender: String,
    pub(super) contract_address: String,
    pub(super) value: BigUint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TransactionPayload {
    NativeTransfer,
    NativeTransferWithCallData,
    Erc20Approve(Erc20ApprovalPayload),
    Erc20Transfer(Erc20TransferPayload),
    Erc721Transfer(NftTransferPayload),
    Erc1155Transfer(NftTransferPayload),
    SmartContractCall,
    Unknown,
}

impl TransactionPayload {
    pub(super) fn from_transaction(transaction: &Transaction, transaction_receipt: &TransactionReceipt, from: &str, to: &str) -> Self {
        let has_recipient = transaction.to.is_some();
        let has_call_data = Self::has_call_data(transaction);

        match transaction.input.as_str() {
            input if has_recipient && input == INPUT_0X && transaction_receipt.logs.is_empty() && (transaction.value > BigUint::from(0u8) || from == to) => Self::NativeTransfer,
            input if input.starts_with(FUNCTION_ERC20_APPROVE) => match Self::erc20_approval(transaction_receipt) {
                Some(approval) => Self::Erc20Approve(approval),
                None => Self::SmartContractCall,
            },
            input if input.starts_with(FUNCTION_ERC20_TRANSFER) => match Self::erc20_transfer(transaction_receipt) {
                Some(transfer) => Self::Erc20Transfer(transfer),
                None => Self::SmartContractCall,
            },
            input if input.starts_with(FUNCTION_ERC721_TRANSFER) || input.starts_with(FUNCTION_ERC721_SAFE_TRANSFER) => {
                Self::nft_transfer_or_contract_call(Self::erc721_transfer(transaction_receipt), Self::Erc721Transfer, transaction_receipt, from, to)
            }
            input if input.starts_with(FUNCTION_ERC1155_TRANSFER) => {
                Self::nft_transfer_or_contract_call(Self::erc1155_transfer(transaction_receipt), Self::Erc1155Transfer, transaction_receipt, from, to)
            }
            _ => match Self::relevant_erc20_transfer(transaction_receipt, from, to) {
                Some(transfer) => Self::Erc20Transfer(transfer),
                None if has_call_data
                    && transaction.gas > TRANSFER_GAS_LIMIT
                    && data_cost(&transaction.input).is_some_and(|data_cost| transaction_receipt.gas_used <= BigUint::from(TRANSFER_GAS_LIMIT + data_cost)) =>
                {
                    Self::NativeTransferWithCallData
                }
                None if has_call_data => Self::SmartContractCall,
                None => Self::Unknown,
            },
        }
    }

    fn has_call_data(transaction: &Transaction) -> bool {
        transaction.to.is_some() && transaction.input.len() > INPUT_0X.len()
    }

    fn erc20_transfer(transaction_receipt: &TransactionReceipt) -> Option<Erc20TransferPayload> {
        let log = Self::log_with_topic(transaction_receipt, TRANSFER_TOPIC, 3, LogSearch::FirstMatch)?;
        let from = ethereum_address_from_topic(log.topics.get(1)?)?;
        let to = ethereum_address_from_topic(log.topics.get(2)?)?;
        let value = biguint_from_hex(&log.data)?;
        let contract_address = ethereum_address_checksum(&log.address).ok()?;

        Some(Erc20TransferPayload {
            from,
            to,
            contract_address,
            value,
        })
    }

    fn relevant_erc20_transfer(transaction_receipt: &TransactionReceipt, from: &str, to: &str) -> Option<Erc20TransferPayload> {
        let transfer = Self::erc20_transfer(transaction_receipt)?;
        match transaction_receipt.logs.len() <= 2 && (transfer.from == from || transfer.from == to) {
            true => Some(transfer),
            false => None,
        }
    }

    fn nft_transfer_or_contract_call(
        nft_transfer: Option<NftTransferPayload>,
        nft_payload: fn(NftTransferPayload) -> Self,
        transaction_receipt: &TransactionReceipt,
        from: &str,
        to: &str,
    ) -> Self {
        match nft_transfer {
            Some(transfer) => nft_payload(transfer),
            None => match Self::relevant_erc20_transfer(transaction_receipt, from, to) {
                Some(transfer) => Self::Erc20Transfer(transfer),
                None => Self::SmartContractCall,
            },
        }
    }

    fn erc20_approval(transaction_receipt: &TransactionReceipt) -> Option<Erc20ApprovalPayload> {
        let log = Self::log_with_topic(transaction_receipt, APPROVAL_TOPIC, 3, LogSearch::FirstMatch)?;
        let spender = ethereum_address_from_topic(&log.topics[2])?;
        let value = biguint_from_hex(&log.data)?;
        let contract_address = ethereum_address_checksum(&log.address).ok()?;

        Some(Erc20ApprovalPayload { spender, contract_address, value })
    }

    fn erc721_transfer(transaction_receipt: &TransactionReceipt) -> Option<NftTransferPayload> {
        let log = Self::log_with_topic(transaction_receipt, TRANSFER_TOPIC, 4, LogSearch::Last)?;
        let to = ethereum_address_from_topic(&log.topics[2])?;
        let token_id = biguint_from_hex(&log.topics[3])?;
        let contract_address = ethereum_address_checksum(&log.address).ok()?;

        Some(NftTransferPayload { to, contract_address, token_id })
    }

    fn erc1155_transfer(transaction_receipt: &TransactionReceipt) -> Option<NftTransferPayload> {
        let log = Self::log_with_topic(transaction_receipt, TRANSFER_SINGLE, 4, LogSearch::Last)?;
        let to = ethereum_address_from_topic(&log.topics[3])?;
        let data = hex_value(&log.data);
        let token_id = BigUint::from_str_radix(data.get(0..64)?, 16).ok()?;
        let contract_address = ethereum_address_checksum(&log.address).ok()?;

        Some(NftTransferPayload { to, contract_address, token_id })
    }

    fn log_with_topic<'a>(transaction_receipt: &'a TransactionReceipt, topic: &str, topics_len: usize, search: LogSearch) -> Option<&'a Log> {
        match search {
            LogSearch::FirstMatch => transaction_receipt.logs.iter().find(|log| Self::has_topic(log, topic, topics_len)),
            LogSearch::Last => transaction_receipt.logs.last().filter(|log| Self::has_topic(log, topic, topics_len)),
        }
    }

    fn has_topic(log: &Log, topic: &str, topics_len: usize) -> bool {
        log.topics.len() == topics_len && log.topics.first().is_some_and(|log_topic| log_topic == topic)
    }
}

fn data_cost(input: &str) -> Option<u64> {
    let bytes = hex::decode(hex_value(input)).ok()?;
    let data_cost = bytes.iter().map(|byte| if *byte == 0 { 4 } else { 68 }).sum();

    Some(data_cost)
}

fn biguint_from_hex(value: &str) -> Option<BigUint> {
    BigUint::from_str_radix(hex_value(value), 16).ok()
}

fn hex_value(value: &str) -> &str {
    value.strip_prefix("0x").unwrap_or(value)
}
