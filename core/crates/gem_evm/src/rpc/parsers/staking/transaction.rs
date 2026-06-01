use num_bigint::BigUint;
use primitives::{AssetId, Transaction as PrimitivesTransaction, TransactionType};

use crate::ethereum_address_checksum;

use super::ParseContext;

pub(super) fn make_staking_transaction(context: &ParseContext<'_>, to: &str, transaction_type: TransactionType, value: BigUint) -> Option<PrimitivesTransaction> {
    let from = ethereum_address_checksum(&context.transaction.from).ok()?;
    let contract = context.transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());

    Some(PrimitivesTransaction::new(
        context.transaction.hash.clone(),
        AssetId::from_chain(*context.chain),
        from,
        to.to_string(),
        contract,
        transaction_type,
        context.receipt.get_state(),
        context.receipt.get_fee().to_string(),
        AssetId::from_chain(*context.chain),
        value.to_string(),
        None,
        None,
        context.created_at,
    ))
}
