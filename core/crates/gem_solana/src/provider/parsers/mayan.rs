use hex_lit::hex;
use num_bigint::{BigInt, Sign};
use primitives::{AssetId, Chain, Transaction};

use crate::{MAYAN_CPI_PROXY_PROGRAM_ID, MAYAN_SWIFT_V2_PROGRAM_ID, WSOL_TOKEN_ADDRESS, models::Instruction};

use super::{ParseContext, ParseContextExt, TransactionParser};

const INIT_ORDER_DISCRIMINATOR: [u8; 8] = hex!("204c290c27a284db");
const SWIFT_PROGRAM_ACCOUNT_INDEX: usize = 0;
const TRADER_ACCOUNT_INDEX: usize = 1;
const ORDER_STATE_ACCOUNT_INDEX: usize = 3;
const ORDER_TOKEN_ACCOUNT_INDEX: usize = 4;

pub(super) struct MayanParser;

impl TransactionParser<ParseContext<'_>, Transaction> for MayanParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context.transaction.transaction.message.instructions.iter().any(|instruction| is_init_order(context, instruction))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<Transaction> {
        let instruction = context.transaction.transaction.message.instructions.iter().find(|instruction| is_init_order(context, instruction))?;
        let trader = instruction.accounts.get(TRADER_ACCOUNT_INDEX).and_then(|index| context.transaction.account_key(*index as usize))?.clone();
        let (asset_id, value) = source_debit(context, instruction, &trader)?;

        context.make_swap_transaction(trader, MAYAN_CPI_PROXY_PROGRAM_ID, asset_id, value)
    }
}

fn is_init_order(context: &ParseContext<'_>, instruction: &Instruction) -> bool {
    if context.transaction.account_key(instruction.program_id_index).map(String::as_str) != Some(MAYAN_CPI_PROXY_PROGRAM_ID) {
        return false;
    }
    if instruction.accounts.get(SWIFT_PROGRAM_ACCOUNT_INDEX).and_then(|index| context.transaction.account_key(*index as usize)).map(String::as_str) != Some(MAYAN_SWIFT_V2_PROGRAM_ID) {
        return false;
    }

    bs58::decode(&instruction.data).into_vec().is_ok_and(|data| data.starts_with(&INIT_ORDER_DISCRIMINATOR))
}

fn source_debit(context: &ParseContext<'_>, instruction: &Instruction, trader: &str) -> Option<(AssetId, num_bigint::BigUint)> {
    let token_debits = context
        .transaction
        .meta
        .get_token_balance_changes_by_owner(trader)
        .into_iter()
        .filter(|change| change.amount.sign() == Sign::Minus)
        .collect::<Vec<_>>();
    match token_debits.as_slice() {
        [debit] => return Some((debit.asset_id.clone(), debit.amount.magnitude().clone())),
        [] => {}
        _ => return None,
    }

    let native = context.transaction.get_balance_changes_by_owner(trader);
    let amount = native.amount + created_order_accounts_rent(context, instruction);
    (amount.sign() == Sign::Minus).then(|| (Chain::Solana.as_asset_id(), amount.magnitude().clone()))
}

fn created_order_accounts_rent(context: &ParseContext<'_>, instruction: &Instruction) -> BigInt {
    [ORDER_STATE_ACCOUNT_INDEX, ORDER_TOKEN_ACCOUNT_INDEX]
        .into_iter()
        .filter_map(|account_position| instruction.accounts.get(account_position).copied())
        .map(|account_index| {
            let index = account_index as usize;
            let pre = context.transaction.meta.pre_balances.get(index).copied().unwrap_or(0);
            let post = context.transaction.meta.post_balances.get(index).copied().unwrap_or(0);
            if pre != 0 || post <= pre {
                return BigInt::from(0);
            }

            let wrapped_sol = context
                .transaction
                .meta
                .get_post_token_balance(i64::from(account_index))
                .filter(|balance| balance.mint == WSOL_TOKEN_ADDRESS)
                .map(|balance| BigInt::from(balance.get_amount()))
                .unwrap_or_default();
            BigInt::from(post - pre) - wrapped_sol
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain, JsonRpcResult, TransactionState, TransactionType};

    use crate::models::{BlockTransaction, SingleTransaction};

    use super::*;
    use crate::provider::parsers::ProtocolParsers;

    fn parse_transaction(payload: &str) -> Transaction {
        let result: JsonRpcResult<SingleTransaction> = serde_json::from_str(payload).unwrap();
        let created_at = DateTime::from_timestamp(result.result.block_time, 0).unwrap();
        let transaction = BlockTransaction {
            meta: result.result.meta,
            transaction: result.result.transaction,
        };
        ProtocolParsers::map_transaction(&transaction, created_at, None).unwrap()
    }

    #[test]
    fn test_parse_token_deposit() {
        let transaction = parse_transaction(include_str!("../../../testdata/mayan_swift_deposit_token.json"));

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
        assert_eq!(transaction.from, "AVuw5YptnLN5KJM2L52CbGYu4nzjiDYrCF1XZ4E9oTun");
        assert_eq!(transaction.to, MAYAN_CPI_PROXY_PROGRAM_ID);
        assert_eq!(transaction.contract.as_deref(), Some(MAYAN_CPI_PROXY_PROGRAM_ID));
        assert_eq!(transaction.value, BigUint::from(1_000_000_000u64));
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_parse_routed_native_deposit() {
        let transaction = parse_transaction(include_str!("../../../testdata/mayan_deposit_jupiter_route.json"));

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.asset_id, Chain::Solana.as_asset_id());
        assert_eq!(transaction.from, "BnPkG7QaDZQ4k4m2sPYb5mVEA2GgpphY9kgzWAHxjtvC");
        assert_eq!(transaction.to, MAYAN_CPI_PROXY_PROGRAM_ID);
        assert_eq!(transaction.value, BigUint::from(10_000_000u64));
    }
}
