use borsh::BorshDeserialize;
use hex_lit::hex;
use num_bigint::{BigUint, Sign};
use primitives::{AssetId, Chain, Transaction};

use crate::{
    RELAY_DEPOSITORY_PROGRAM_ID, RELAY_FORWARDER_PROGRAM_ID,
    models::{BlockTransaction, Instruction, TokenBalanceChange},
};

use super::{ParseContext, ParseContextExt, TransactionParser};

const DEPOSIT_NATIVE_DISCRIMINATOR: [u8; 8] = hex!("0d9e0ddf5fd51c06");
const DEPOSIT_TOKEN_DISCRIMINATOR: [u8; 8] = hex!("0b9c60da27a3b413");
const FORWARD_TOKEN_DISCRIMINATOR: [u8; 8] = hex!("9d537021bf32ab25");
const SENDER_ACCOUNT_INDEX: usize = 1;
const FORWARDER_SENDER_ACCOUNT_INDEX: usize = 0;
const TOKEN_MINT_ACCOUNT_INDEX: usize = 4;
const RELAY_PROGRAMS: [&str; 2] = [RELAY_DEPOSITORY_PROGRAM_ID, RELAY_FORWARDER_PROGRAM_ID];

pub(super) struct RelayParser;

struct DecodedDeposit {
    sender: String,
    program_id: &'static str,
    asset_id: AssetId,
    value: BigUint,
    data: String,
}

#[derive(BorshDeserialize)]
struct DepositArguments {
    amount: u64,
    _id: [u8; 32],
}

impl TransactionParser<ParseContext<'_>, Transaction> for RelayParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context.transaction.transaction.message.instructions.iter().any(|instruction| is_relay_instruction(context, instruction))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<Transaction> {
        let deposit = context.transaction.transaction.message.instructions.iter().find_map(|instruction| decode_instruction(context, instruction))?;

        Some(context.make_swap_transaction(deposit.sender, deposit.program_id, deposit.asset_id, deposit.value)?.with_data(Some(deposit.data)))
    }
}

fn is_relay_instruction(context: &ParseContext<'_>, instruction: &Instruction) -> bool {
    context.transaction.account_key(instruction.program_id_index).is_some_and(|key| RELAY_PROGRAMS.contains(&key.as_str()))
}

fn decode_instruction(context: &ParseContext<'_>, instruction: &Instruction) -> Option<DecodedDeposit> {
    match context.transaction.account_key(instruction.program_id_index)?.as_str() {
        RELAY_DEPOSITORY_PROGRAM_ID => decode_deposit(context, instruction),
        RELAY_FORWARDER_PROGRAM_ID => decode_forward(context, instruction),
        _ => None,
    }
}

fn decode_deposit(context: &ParseContext<'_>, instruction: &Instruction) -> Option<DecodedDeposit> {
    let data = bs58::decode(&instruction.data).into_vec().ok()?;
    let discriminator = data.get(..DEPOSIT_NATIVE_DISCRIMINATOR.len())?;
    let arguments = DepositArguments::try_from_slice(data.get(DEPOSIT_NATIVE_DISCRIMINATOR.len()..)?).ok()?;
    let account = |position: usize| context.transaction.account_key(*instruction.accounts.get(position)? as usize).cloned();
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
        program_id: RELAY_DEPOSITORY_PROGRAM_ID,
        asset_id,
        value: BigUint::from(arguments.amount),
        data: format!("0x{}", hex::encode(&data)),
    })
}

fn decode_forward(context: &ParseContext<'_>, instruction: &Instruction) -> Option<DecodedDeposit> {
    let data = bs58::decode(&instruction.data).into_vec().ok()?;
    let (discriminator, id) = data.split_first_chunk::<8>()?;
    if *discriminator != FORWARD_TOKEN_DISCRIMINATOR || id.len() != 32 {
        return None;
    }
    let sender = context.transaction.account_key(*instruction.accounts.get(FORWARDER_SENDER_ACCOUNT_INDEX)? as usize)?.clone();
    let spent = spent_balance_change(context.transaction, &sender)?;

    Some(DecodedDeposit {
        sender,
        program_id: RELAY_FORWARDER_PROGRAM_ID,
        asset_id: spent.asset_id,
        value: spent.amount.magnitude().clone(),
        data: format!("0x{}", hex::encode(&data)),
    })
}

fn spent_balance_change(transaction: &BlockTransaction, owner: &str) -> Option<TokenBalanceChange> {
    let change = transaction.meta.get_token_balance_changes_by_owner(owner).into_iter().find(|change| change.amount.sign() == Sign::Minus).unwrap_or_else(|| {
        let native = transaction.get_balance_changes_by_owner(owner);
        TokenBalanceChange {
            amount: native.amount + transaction.get_created_token_accounts_rent_by_owner(owner),
            ..native
        }
    });
    (change.amount.sign() == Sign::Minus).then_some(change)
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
        assert_eq!(transaction.data.as_deref(), Some("0x0b9c60da27a3b41390da90000000000047f5f46034fc442123ac53e7381dcf30ec2b9adf66a1269ee9ebe7232d3069d3"));
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
        assert_eq!(transaction.data.as_deref(), Some("0x0d9e0ddf5fd51c06802b530b00000000a1e4ed561538867bf66097ce6d2d87842bd1549ac3dc0a9e7ed5cfabcf2575a7"));
    }

    #[test]
    fn test_parse_forward_token() {
        let transaction = parse_transaction(include_str!("../../../testdata/relay_forward_token.json"));

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Solana, "EPeUFDgHRxs9xxEPVaL6kfGQvCon7jmAWKVUHuux1Tpz"));
        assert_eq!(transaction.from, "CFG2EQVPWobE3jTpbBEzh7V3ucQ7fYSAP2hpJUuqqJHY");
        assert_eq!(transaction.to, RELAY_FORWARDER_PROGRAM_ID);
        assert_eq!(transaction.value, BigUint::from(13_002_173_634u64));
        assert!(transaction.metadata.is_none());
        assert_eq!(transaction.data.as_deref(), Some("0x9d537021bf32ab25f9acf2b30950256def67cea790e16eaaa70a9f91bf4b4d64f33e9a217f8a42f6"));
    }
}
