use borsh::BorshDeserialize;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, Transaction};

use crate::{RELAY_DEPOSITORY_PROGRAM_ID, models::Instruction};

use super::{ParseContext, ParseContextExt, TransactionParser};

const DEPOSIT_NATIVE_DISCRIMINATOR: [u8; 8] = [13, 158, 13, 223, 95, 213, 28, 6];
const DEPOSIT_TOKEN_DISCRIMINATOR: [u8; 8] = [11, 156, 96, 218, 39, 163, 180, 19];
const SENDER_ACCOUNT_INDEX: usize = 1;
const TOKEN_MINT_ACCOUNT_INDEX: usize = 4;

pub(super) struct RelayParser;

struct DecodedDeposit {
    sender: String,
    asset_id: AssetId,
    value: BigUint,
}

#[derive(BorshDeserialize)]
struct DepositArguments {
    amount: u64,
    _id: [u8; 32],
}

impl TransactionParser<ParseContext<'_>, Transaction> for RelayParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context
            .transaction
            .transaction
            .message
            .instructions
            .iter()
            .any(|instruction| is_relay_instruction(context, instruction))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<Transaction> {
        let deposit = context
            .transaction
            .transaction
            .message
            .instructions
            .iter()
            .filter(|instruction| is_relay_instruction(context, instruction))
            .find_map(|instruction| decode_deposit(context, instruction))?;

        context.make_swap_transaction(deposit.sender, RELAY_DEPOSITORY_PROGRAM_ID, deposit.asset_id, deposit.value)
    }
}

fn is_relay_instruction(context: &ParseContext<'_>, instruction: &Instruction) -> bool {
    context.transaction.transaction.message.account_keys.get(instruction.program_id_index).map(String::as_str) == Some(RELAY_DEPOSITORY_PROGRAM_ID)
}

fn decode_deposit(context: &ParseContext<'_>, instruction: &Instruction) -> Option<DecodedDeposit> {
    let data = bs58::decode(&instruction.data).into_vec().ok()?;
    let discriminator = data.get(..DEPOSIT_NATIVE_DISCRIMINATOR.len())?;
    let arguments = DepositArguments::try_from_slice(data.get(DEPOSIT_NATIVE_DISCRIMINATOR.len()..)?).ok()?;
    let account_keys = &context.transaction.transaction.message.account_keys;
    let account = |position: usize| account_keys.get(*instruction.accounts.get(position)? as usize).cloned();
    let sender = account(SENDER_ACCOUNT_INDEX)?;
    let asset_id = if discriminator == DEPOSIT_NATIVE_DISCRIMINATOR {
        Chain::Solana.as_asset_id()
    } else if discriminator == DEPOSIT_TOKEN_DISCRIMINATOR {
        let mint = account(TOKEN_MINT_ACCOUNT_INDEX)?;
        AssetId::from_token(Chain::Solana, &mint)
    } else {
        return None;
    };

    Some(DecodedDeposit {
        sender,
        asset_id,
        value: BigUint::from(arguments.amount),
    })
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
        let transaction = parse_transaction(include_str!("../../../testdata/relay_deposit_token.json"));

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Solana, "CASHx9KJUStyftLFWGvEVf59SGeG9sh5FfcnZMVPCASH"));
        assert_eq!(transaction.from, "HGsJPZ2sb5e31hnyvzt8uAiNKWZm4zp8AmzJRmDfKAhy");
        assert_eq!(transaction.to, RELAY_DEPOSITORY_PROGRAM_ID);
        assert_eq!(transaction.contract, Some(RELAY_DEPOSITORY_PROGRAM_ID.to_string()));
        assert_eq!(transaction.value, BigUint::from(9_493_136u64));
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_parse_native_deposit() {
        let transaction = parse_transaction(include_str!("../../../testdata/relay_deposit_native.json"));

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, Chain::Solana.as_asset_id());
        assert_eq!(transaction.from, "6mW9UoanWQNFcmUnzzNCFyk6KCemATDEMEWeQ938jj6E");
        assert_eq!(transaction.to, RELAY_DEPOSITORY_PROGRAM_ID);
        assert_eq!(transaction.contract, Some(RELAY_DEPOSITORY_PROGRAM_ID.to_string()));
        assert_eq!(transaction.value, BigUint::from(190_000_000u64));
        assert!(transaction.metadata.is_none());
    }
}
