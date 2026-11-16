use crate::{address::ethereum_address_checksum, rpc::transaction_payload::Erc20TransferPayload};
use primitives::{AssetId, Transaction as PrimitivesTransaction, TransactionType, contract_constants::ETHEREUM_YO_PROTOCOL_CONTRACT};

use super::{ParseContext, TransactionParser};

pub(crate) const FUNCTION_YO_DEPOSIT: &str = "0x82b78ba7";
pub(crate) const FUNCTION_YO_WITHDRAW: &str = "0x99519ab8";

pub struct YoParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for YoParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(to) = context.transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok()) else {
            return false;
        };
        let Some(contract) = ethereum_address_checksum(ETHEREUM_YO_PROTOCOL_CONTRACT).ok() else {
            return false;
        };

        to == contract && (context.transaction.input.starts_with(FUNCTION_YO_DEPOSIT) || context.transaction.input.starts_with(FUNCTION_YO_WITHDRAW))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        let to = ethereum_address_checksum(context.transaction.to.as_ref()?).ok()?;
        let mut transfers = context.metadata.receipt.logs.iter().filter_map(Erc20TransferPayload::from_log);
        let (transaction_type, transfer) = if context.transaction.input.starts_with(FUNCTION_YO_DEPOSIT) {
            (TransactionType::EarnDeposit, transfers.find(|transfer| transfer.from == from)?)
        } else if context.transaction.input.starts_with(FUNCTION_YO_WITHDRAW) {
            (TransactionType::EarnWithdraw, transfers.find(|transfer| transfer.to == from)?)
        } else {
            return None;
        };

        Some(PrimitivesTransaction::new(
            context.transaction.hash.clone(),
            AssetId::from_token(*context.metadata.chain, &transfer.contract_address),
            from,
            to,
            None,
            transaction_type,
            context.metadata.receipt.get_state(),
            context.metadata.receipt.get_fee(),
            AssetId::from_chain(*context.metadata.chain),
            transfer.value,
            None,
            None,
            context.created_at,
        ))
    }
}
