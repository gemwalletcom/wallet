use gem_encoding::decode_base58;
use num_bigint::{BigUint, Sign};

use crate::{
    COMPUTE_BUDGET_PROGRAM_ID, JUPITER_PROGRAM_ID, MEMO_PROGRAM_ID, METAPLEX_CORE_PROGRAM, METAPLEX_PROGRAM, METEORA_DAMM_V2_PROGRAM_ID, METEORA_DLMM_PROGRAM_ID, OKX_DEX_V2_PROGRAM_ID, ORCA_WHIRLPOOL_PROGRAM_ID, PUMP_AMM_PROGRAM_ID,
    RAYDIUM_AMM_PROGRAM_ID, RAYDIUM_CLMM_PROGRAM_ID, RAYDIUM_CPMM_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSTEM_PROGRAMS, TOKEN_PROGRAM, TOKEN_PROGRAM_2022, WSOL_TOKEN_ADDRESS,
    instructions::system,
    models::{BlockTransaction, BlockTransactions, SingleTransaction, TokenBalanceChange, TransactionSignature},
};
use primitives::{
    AssetId, Chain, DateTime, NFTAssetId, OptionStringExt, SwapProvider, Transaction, TransactionNFTTransferMetadata, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType, swap::SOLANA_REFERRAL_ADDRESS,
};

use super::parsers::ProtocolParsers;
use chain_traits::{TransactionIdRequest, TransactionsResult};

const CHAIN: Chain = Chain::Solana;
const SWAP_PROGRAMS: &[(SwapProvider, &str)] = &[
    (SwapProvider::Jupiter, JUPITER_PROGRAM_ID),
    (SwapProvider::Okx, OKX_DEX_V2_PROGRAM_ID),
    (SwapProvider::Orca, ORCA_WHIRLPOOL_PROGRAM_ID),
    (SwapProvider::RaydiumClmm, RAYDIUM_CLMM_PROGRAM_ID),
    (SwapProvider::RaydiumClmm, RAYDIUM_CPMM_PROGRAM_ID),
    (SwapProvider::RaydiumClmm, RAYDIUM_AMM_PROGRAM_ID),
    (SwapProvider::PumpSwap, PUMP_AMM_PROGRAM_ID),
    (SwapProvider::MeteoraDammV2, METEORA_DAMM_V2_PROGRAM_ID),
    (SwapProvider::MeteoraDammV2, METEORA_DLMM_PROGRAM_ID),
];
const MPL_CORE_TRANSFER_V1: u8 = 14;
const MPL_TOKEN_METADATA_TRANSFER_V1: u8 = 49;
const MPL_TOKEN_METADATA_MINT_ACCOUNT_INDEX: usize = 4;
const SPL_TRANSFER_CHECKED: u8 = 12;
const SPL_TRANSFER_CHECKED_MINT_ACCOUNT_INDEX: usize = 1;

struct NativeTransfer {
    sender: String,
    recipient: String,
    lamports: u64,
}

struct MetaplexCoreNftTransfer {
    sender: String,
    new_owner: String,
    asset: String,
    collection: String,
}

fn map_memo(transaction: &BlockTransaction) -> Option<String> {
    transaction.transaction.message.instructions.iter().find_map(|instruction| {
        if transaction.instruction_program(instruction) != Some(MEMO_PROGRAM_ID) {
            return None;
        }
        String::from_utf8(decode_base58(&instruction.data).ok()?).ok().non_empty()
    })
}

fn map_native_transfer(transaction: &BlockTransaction) -> Option<NativeTransfer> {
    let mut transfer = None;
    for instruction in &transaction.transaction.message.instructions {
        match transaction.instruction_program(instruction)? {
            COMPUTE_BUDGET_PROGRAM_ID | MEMO_PROGRAM_ID => {}
            SYSTEM_PROGRAM_ID => {
                let data = decode_base58(&instruction.data).ok()?;
                if system::is_advance_nonce_account_data(&data) {
                    continue;
                }
                let next = NativeTransfer {
                    sender: transaction.instruction_account(instruction, 0)?.clone(),
                    recipient: transaction.instruction_account(instruction, 1)?.clone(),
                    lamports: system::transfer_lamports(&data)?,
                };
                if transfer.replace(next).is_some() {
                    return None;
                }
            }
            _ => return None,
        }
    }
    transfer
}

fn map_metaplex_core_nft_transfer(transaction: &BlockTransaction, account_keys: &[String]) -> Option<MetaplexCoreNftTransfer> {
    if !account_keys.iter().any(|key| key == METAPLEX_CORE_PROGRAM) {
        return None;
    }
    let instruction = transaction
        .transaction
        .message
        .instructions
        .iter()
        .find(|instruction| transaction.instruction_program(instruction) == Some(METAPLEX_CORE_PROGRAM))?;
    let data = decode_base58(&instruction.data).ok()?;
    if data.first().copied() != Some(MPL_CORE_TRANSFER_V1) {
        return None;
    }
    let resolve = |position: usize| transaction.instruction_account(instruction, position).cloned();
    Some(MetaplexCoreNftTransfer {
        asset: resolve(0)?,
        collection: resolve(1)?,
        sender: resolve(2)?,
        new_owner: resolve(4)?,
    })
}

fn is_nft_token_transfer(transaction: &BlockTransaction, token_id: &str, value: &BigUint) -> bool {
    if value != &BigUint::from(1u8) {
        return false;
    }
    if !transaction
        .meta
        .pre_token_balances
        .iter()
        .chain(transaction.meta.post_token_balances.iter())
        .all(|balance| balance.ui_token_amount.decimals == 0)
    {
        return false;
    }

    transaction.transaction.message.instructions.iter().any(|instruction| {
        let Some(program) = transaction.instruction_program(instruction) else {
            return false;
        };
        let mint_account_index = if program == METAPLEX_PROGRAM {
            MPL_TOKEN_METADATA_MINT_ACCOUNT_INDEX
        } else if program == TOKEN_PROGRAM || program == TOKEN_PROGRAM_2022 {
            SPL_TRANSFER_CHECKED_MINT_ACCOUNT_INDEX
        } else {
            return false;
        };
        if transaction.instruction_account(instruction, mint_account_index).map(String::as_str) != Some(token_id) {
            return false;
        }
        let Ok(data) = decode_base58(&instruction.data) else {
            return false;
        };
        if program == METAPLEX_PROGRAM {
            data.first().copied() == Some(MPL_TOKEN_METADATA_TRANSFER_V1)
        } else {
            let [instruction, amount @ .., decimals] = data.as_slice() else {
                return false;
            };
            if *instruction != SPL_TRANSFER_CHECKED || *decimals != 0 {
                return false;
            }
            let Ok(amount) = <[u8; 8]>::try_from(amount) else {
                return false;
            };
            u64::from_le_bytes(amount) == 1
        }
    })
}

fn get_swap_provider(account_keys: &[String]) -> Option<(SwapProvider, &'static str)> {
    SWAP_PROGRAMS.iter().copied().find(|(_, program_id)| account_keys.iter().any(|key| key == program_id))
}

fn map_swap_metadata(transaction: &BlockTransaction, owner: &str, provider: SwapProvider) -> Option<TransactionSwapMetadata> {
    let native_change = transaction.get_balance_changes_by_owner(owner);
    let balance_changes = TokenBalanceChange {
        amount: native_change.amount + transaction.get_created_token_accounts_rent_by_owner(owner),
        ..native_change
    };
    let token_balance_changes = transaction.meta.get_token_balance_changes_by_owner(owner);

    let (from_asset, from_value, to_asset, to_value) = match token_balance_changes.as_slice() {
        [change] => {
            let (from, to) = match change.amount.sign() {
                Sign::Plus => (&balance_changes, change),
                Sign::Minus => (change, &balance_changes),
                Sign::NoSign => return None,
            };
            if to.amount.sign() != Sign::Plus || from.amount.sign() != Sign::Minus {
                return None;
            }
            (from.asset_id.clone(), from.amount.magnitude().clone(), to.asset_id.clone(), to.amount.magnitude().clone())
        }
        [a, b] => {
            let (from, to) = match a.amount.sign() {
                Sign::Plus => (b, a),
                Sign::Minus => (a, b),
                Sign::NoSign => return None,
            };
            (from.asset_id.clone(), from.amount.magnitude().clone(), to.asset_id.clone(), to.amount.magnitude().clone())
        }
        _ => return None,
    };

    Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, provider).with_referral_fee(map_referral_fee(transaction, owner)))
}

fn map_referral_fee(transaction: &BlockTransaction, owner: &str) -> Option<TransactionSwapReferralFee> {
    if owner == SOLANA_REFERRAL_ADDRESS {
        return None;
    }
    let token_changes = transaction.meta.get_token_balance_changes_by_owner(SOLANA_REFERRAL_ADDRESS);
    let native_change = transaction.get_balance_changes_by_owner(SOLANA_REFERRAL_ADDRESS);
    token_changes.into_iter().chain([native_change]).find(|change| change.amount.sign() == Sign::Plus).map(|change| TransactionSwapReferralFee {
        asset_id: match change.asset_id.token_id.as_deref() {
            Some(WSOL_TOKEN_ADDRESS) => CHAIN.as_asset_id(),
            _ => change.asset_id,
        },
        value: change.amount.magnitude().clone(),
    })
}

pub fn map_transaction_requests(signatures: Vec<TransactionSignature>) -> TransactionsResult {
    TransactionsResult::TransactionRequests(signatures.into_iter().map(|signature| TransactionIdRequest::new(CHAIN, signature.signature, None)).collect())
}

pub fn map_block_transactions(transactions: &BlockTransactions) -> Vec<primitives::Transaction> {
    transactions.transactions.iter().filter_map(|transaction| map_transaction(transaction, transactions.block_time)).collect()
}

pub fn map_single_transaction(transaction: SingleTransaction) -> Option<primitives::Transaction> {
    let block_transaction = BlockTransaction {
        meta: transaction.meta,
        transaction: transaction.transaction,
    };
    map_transaction(&block_transaction, transaction.block_time)
}

pub fn map_transaction(transaction: &BlockTransaction, created_at: DateTime) -> Option<primitives::Transaction> {
    if transaction.transaction.signatures.len() > 2 {
        return None;
    }

    let chain = CHAIN;
    let account_keys = &transaction.transaction.message.account_keys;
    let hash = transaction.transaction.signatures.first()?.clone();
    let fee = transaction.meta.fee;
    let state = if transaction.meta.has_error() { TransactionState::Reverted } else { TransactionState::Confirmed };
    let fee_asset_id = chain.as_asset_id();
    let memo = map_memo(transaction);

    if let Some(transaction) = ProtocolParsers::map_transaction(transaction, created_at, memo.as_deref()) {
        return Some(transaction);
    }

    if let Some(transfer) = map_native_transfer(transaction) {
        let transaction = Transaction::new(
            hash,
            chain.as_asset_id(),
            transfer.sender,
            transfer.recipient,
            None,
            TransactionType::Transfer,
            state,
            BigUint::from(fee),
            fee_asset_id,
            BigUint::from(transfer.lamports),
            memo,
            None,
            created_at,
        );
        return Some(transaction);
    }

    let pre_token_balances = &transaction.meta.pre_token_balances;
    let post_token_balances = &transaction.meta.post_token_balances;

    if let Some(first_balance) = pre_token_balances.first() {
        let token_id = &first_balance.mint;
        if account_keys.iter().any(|key| key == TOKEN_PROGRAM || key == TOKEN_PROGRAM_2022)
            && (pre_token_balances.len() == 1 || pre_token_balances.len() == 2)
            && post_token_balances.len() == 2
            && pre_token_balances.iter().all(|b| &b.mint == token_id)
            && post_token_balances.iter().all(|b| &b.mint == token_id)
        {
            let sender_account_index: i64 = if transaction.meta.pre_token_balances.len() == 1 {
                transaction.meta.pre_token_balances.first()?.account_index
            } else if pre_token_balances.first()?.get_amount() >= post_token_balances.first()?.get_amount() {
                pre_token_balances.first()?.account_index
            } else {
                post_token_balances.last()?.account_index
            };
            let recipient_account_index = post_token_balances.iter().find(|b| b.account_index != sender_account_index)?.account_index;

            let sender = transaction.meta.get_post_token_balance(sender_account_index)?;
            let recipient = transaction.meta.get_post_token_balance(recipient_account_index)?;
            let from_value = transaction.meta.get_pre_token_balance(sender_account_index)?.get_amount();
            let to_value = transaction.meta.get_post_token_balance(sender_account_index)?.get_amount();

            if to_value > from_value {
                return None;
            }
            let value = from_value - to_value;
            let from = sender.owner;
            let to = recipient.owner;

            let is_nft = is_nft_token_transfer(transaction, token_id, &value);
            let (transaction_type, asset_id, metadata) = if is_nft {
                let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(chain, token_id, token_id));
                (TransactionType::TransferNFT, chain.as_asset_id(), serde_json::to_value(metadata).ok())
            } else {
                (TransactionType::Transfer, AssetId { chain, token_id: Some(token_id.clone()) }, None)
            };

            let transaction = Transaction::new(hash, asset_id, from, to, None, transaction_type, state, BigUint::from(fee), fee_asset_id, value, memo, metadata, created_at);
            return Some(transaction);
        }
    }

    if let Some(nft) = map_metaplex_core_nft_transfer(transaction, account_keys) {
        let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(chain, &nft.collection, &nft.asset));
        return Some(Transaction::new(
            hash,
            chain.as_asset_id(),
            nft.sender,
            nft.new_owner,
            None,
            TransactionType::TransferNFT,
            state,
            BigUint::from(fee),
            fee_asset_id,
            BigUint::from(0u32),
            memo,
            serde_json::to_value(metadata).ok(),
            created_at,
        ));
    }

    let sender = account_keys.first()?.clone();
    if let Some((provider, program_id)) = get_swap_provider(account_keys)
        && let Some(swap) = map_swap_metadata(transaction, &sender, provider)
    {
        let transaction = Transaction::new(
            hash,
            swap.from_asset.clone(),
            sender.clone(),
            sender,
            Some(program_id.to_string()),
            TransactionType::Swap,
            state,
            BigUint::from(fee),
            chain.as_asset_id(),
            swap.from_value.clone(),
            memo,
            serde_json::to_value(swap).ok(),
            created_at,
        );
        return Some(transaction);
    }

    let contract = transaction
        .transaction
        .message
        .instructions
        .iter()
        .filter_map(|instruction| transaction.instruction_program(instruction))
        .find(|program| !SYSTEM_PROGRAMS.contains(program))?;
    let value = transaction.get_balance_change(&sender);

    Some(Transaction::new(
        hash,
        chain.as_asset_id(),
        sender.clone(),
        sender,
        Some(contract.to_string()),
        TransactionType::SmartContractCall,
        state,
        BigUint::from(fee),
        fee_asset_id,
        BigUint::from(value),
        memo,
        None,
        created_at,
    ))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use crate::{PYUSD_TOKEN_MINT, RELAY_DEPOSITORY_PROGRAM_ID, USDC_TOKEN_MINT, USDT_TOKEN_MINT, models::SolanaTransaction};
    use gem_jsonrpc::types::JsonRpcErrorResponse;
    use primitives::{JsonRpcResult, asset_constants::SOLANA_USDC_ASSET_ID, testkit::json::load_json_rpc_result};

    const SPL_NFT_MINT: &str = "4umMdShNxbdnoV2EZjUp6h5GYYneZFLH9otBEU2K3ZYP";
    const PNFT_MINT: &str = "HP82kPNXnQcozjDrV4dLYfV6wwABQDMVPJXezDbZXHEy";
    const CORE_ASSET: &str = "JATWmjADckr2M7TX5xMfo1HNfYS66DKot15fJ4hVLrVE";
    const CORE_COLLECTION: &str = "5pQfZttNUtaj8sySRY9RsdtB81aEAQDh2vnacpxiwTpT";

    #[test]
    fn test_map_mayan_deposit_without_local_swap_output() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/mayan_deposit_jupiter_route.json"))).unwrap();

        assert_eq!(transaction.hash(), "vXUaBxa3MtnodHE7by5sXpExWHoMdcp7qiMKk1UoM8CnYBGD84KpzbErWLVf7RfjAmsrhJxjJ3Df7hqJ4Qh4umb");
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_orca_swap_referral_fee() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/swap_orca_referral_fee.json"))).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.provider.as_deref(), Some("orca"));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
                value: BigUint::from(6000u32),
            })
        );
    }

    #[test]
    fn test_map_orca_swap_native_output_referral_fee() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/swap_orca_native_referral_fee.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(
            transaction.swap_metadata(),
            Some(TransactionSwapMetadata {
                from_asset: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
                from_value: BigUint::from(1_000_000u32),
                to_asset: AssetId::from_chain(Chain::Solana),
                to_value: BigUint::from(9_391_724u32),
                provider: Some("orca".to_string()),
                referral_fee: Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Solana),
                    value: BigUint::from(47_196u32),
                }),
            })
        );
    }

    #[test]
    fn test_map_raydium_swap_native_output_referral_fee() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/swap_raydium_native_referral_fee.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(
            transaction.swap_metadata(),
            Some(TransactionSwapMetadata {
                from_asset: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
                from_value: BigUint::from(1_000_000u32),
                to_asset: AssetId::from_chain(Chain::Solana),
                to_value: BigUint::from(9_530_731u32),
                provider: Some("raydium_clmm".to_string()),
                referral_fee: Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Solana),
                    value: BigUint::from(47_893u32),
                }),
            })
        );
    }

    #[test]
    fn test_map_orca_swap_native_input_referral_fee() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/swap_orca_native_input_referral_fee.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(
            transaction.swap_metadata(),
            Some(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(Chain::Solana),
                from_value: BigUint::from(10_000_000u32),
                to_asset: AssetId::from_token(Chain::Solana, USDC_TOKEN_MINT),
                to_value: BigUint::from(1_053_552u32),
                provider: Some("orca".to_string()),
                referral_fee: Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Solana),
                    value: BigUint::from(50_000u32),
                }),
            })
        );
    }

    #[test]
    fn test_map_transfer_sol_with_memo_and_compute_budget() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_with_memo_compute.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "BUW2vSJMbNojvi9vh1oRg6Qd88GAYSqZifXzZ7JN8uKV");
        assert_eq!(transaction.to, "7YjizA4PUnLvuKijmEHjdDDbfWyG2LMqGCLVjpWrARCA");
        assert!(transaction.memo.as_deref().is_some_and(|memo| memo.starts_with('=')));
    }

    #[test]
    fn test_transaction_relay_token_deposit_lookup_table() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/relay_deposit_token_lookup_table.json"))).unwrap();
        let expected = Transaction::new(
            "4JYHmWNmuCXuTMfNFWVYLWsoJSfbaukdzYLDQ12Cxj6A8DsChLwRuUHWKe33JqNtKGM7wPHckz3EvGvvaLE8ormn".to_string(),
            SOLANA_USDC_ASSET_ID.clone(),
            "B1nzrk99FEDAYB2M82yepdvEv1YBRJKcx5Y5R6MSDW1Q".to_string(),
            RELAY_DEPOSITORY_PROGRAM_ID.to_string(),
            Some(RELAY_DEPOSITORY_PROGRAM_ID.to_string()),
            TransactionType::Swap,
            TransactionState::Confirmed,
            BigUint::from(5000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(5_000_000u64),
            None,
            None,
            DateTime::from_timestamp(1789191680),
        )
        .with_data(Some("0x0b9c60da27a3b413404b4c0000000000c6a8c2d31b9e05f9f719c821dff2de43bc811d4efabd39565a355977c7143370".to_string()));

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_nft_spl_token_transfer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/nft_spl_token_transfer.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.asset_id, Chain::Solana.as_asset_id());
        assert_eq!(transaction.from, "2k5AXX4guW9XwRQ1AKCpAuUqgWDpQpwFfpVFh3hnm2Ha");
        assert_eq!(transaction.to, "2xSHLfiPs3aEhzbLnYbyzWYMEaYnwSwJwAnVh5CwHWwX");
        assert_eq!(transaction.value, BigUint::from(1u64));
        assert_eq!(transaction.fee, BigUint::from(10000u64));
        assert_eq!(transaction.created_at, DateTime::from_timestamp(1687444563));

        let metadata: TransactionNFTTransferMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.asset_id, NFTAssetId::new(Chain::Solana, SPL_NFT_MINT, SPL_NFT_MINT));
    }

    #[test]
    fn test_transaction_nft_token_metadata_transfer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/nft_token_program_transfer.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.asset_id, Chain::Solana.as_asset_id());
        assert_eq!(transaction.from, "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H");
        assert_eq!(transaction.to, "FGbkx8rYTPJubjyScReeps6GA83D1nSmFr3BrN7buokb");
        assert_eq!(transaction.value, BigUint::from(1u64));
        assert_eq!(transaction.fee, BigUint::from(7500u64));
        assert_eq!(transaction.created_at, DateTime::from_timestamp(1779221552));

        let metadata: TransactionNFTTransferMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.asset_id, NFTAssetId::new(Chain::Solana, PNFT_MINT, PNFT_MINT));
    }

    #[test]
    fn test_transaction_nft_mplcore_transfer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/nft_mplcore_transfer.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.asset_id, Chain::Solana.as_asset_id());
        assert_eq!(transaction.from, "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H");
        assert_eq!(transaction.to, "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR");
        assert_eq!(transaction.value, BigUint::from(0u64));
        assert_eq!(transaction.fee, BigUint::from(79998u64));
        assert_eq!(transaction.created_at, DateTime::from_timestamp(1752111467));

        let metadata: TransactionNFTTransferMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.asset_id, NFTAssetId::new(Chain::Solana, CORE_COLLECTION, CORE_ASSET));
    }

    #[test]
    fn test_transaction_swap_token_to_sol() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_token_to_sol.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let expected = TransactionSwapMetadata::new(
            AssetId::from_token(Chain::Solana, "BKpSnSdNdANUxKPsn4AQ8mf4b9BoeVs9JD1Q8cVkpump"),
            BigUint::from(393647577456u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(140927839u64),
            SwapProvider::Jupiter,
        );

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_token_to_token() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_token_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let expected = TransactionSwapMetadata::new(
            AssetId::from_token(Chain::Solana, PYUSD_TOKEN_MINT),
            BigUint::from(1000000u64),
            AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
            BigUint::from(999932u64),
            SwapProvider::Jupiter,
        );

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_sol_to_token() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_sol_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let expected = TransactionSwapMetadata {
            referral_fee: Some(TransactionSwapReferralFee {
                asset_id: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
                value: BigUint::from(8436u64),
            }),
            ..TransactionSwapMetadata::new(
                Chain::Solana.as_asset_id(),
                BigUint::from(10000000u64),
                AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
                BigUint::from(1678930u64),
                SwapProvider::Jupiter,
            )
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_jupiter_referral_fee() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_jupiter_referral_fee.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Solana, "LinkhB3afbBKb2EQQu7s7umdZceV3wcvAUJhQAfQ23L"));
        assert_eq!(metadata.from_value, BigUint::from(3000000000u64));
        assert_eq!(metadata.to_asset, SOLANA_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(44913642u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: SOLANA_USDC_ASSET_ID.clone(),
                value: BigUint::from(225696u64),
            })
        );
    }

    #[test]
    fn test_transaction_swap_jupiter_new_token_account_rent() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_jupiter_new_token_account_rent.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Solana, "CASHx9KJUStyftLFWGvEVf59SGeG9sh5FfcnZMVPCASH"));
        assert_eq!(metadata.from_value, BigUint::from(100000u64));
        assert_eq!(metadata.to_asset, Chain::Solana.as_asset_id());
        assert_eq!(metadata.to_value, BigUint::from(837080u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Solana.as_asset_id(),
                value: BigUint::from(4206u64),
            })
        );
    }

    #[test]
    fn test_transaction_swap_okx_token_to_token() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_okx_token_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();
        let expected = TransactionSwapMetadata {
            referral_fee: Some(TransactionSwapReferralFee {
                asset_id: SOLANA_USDC_ASSET_ID.clone(),
                value: BigUint::from(280306u64),
            }),
            ..TransactionSwapMetadata::new(
                SOLANA_USDC_ASSET_ID.clone(),
                BigUint::from(56061275u64),
                AssetId::from_token(Chain::Solana, "HmMubgKx91Tpq3jmfcKQwsv5HrErqnCTTRJMB6afFR2u"),
                BigUint::from(2190151370200u64),
                SwapProvider::Okx,
            )
        };

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.asset_id, SOLANA_USDC_ASSET_ID.clone());
        assert_eq!(transaction.contract, Some(OKX_DEX_V2_PROGRAM_ID.to_string()));
        assert_eq!(transaction.value, BigUint::from(56061275u64));
        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_transfer_sol() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/transfer_sol.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1751394455)).unwrap();
        let expected = Transaction::new(
            "t6DpS6U7G2UG4QwDq4mPM7F45Rnttxp2pHGRwTYsF7frxAs7KmSWWDcpMneMUULbKndkZy8iUvSU1AZUsqzDCPN".to_string(),
            Chain::Solana.as_asset_id(),
            "DyB4TbDBqPUsCfsJMuoqjktEAod7D3KMNULSo7R1Rb61".to_string(),
            "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(5000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(2173u64),
            None,
            None,
            DateTime::from_timestamp(1751394455),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_to_self() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_to_self.json"))).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "Ch4Hfd7yStCT1WzWKXpWFWgJzXfYRUzhK19E42MPM4b6");
        assert_eq!(transaction.to, "Ch4Hfd7yStCT1WzWKXpWFWgJzXfYRUzhK19E42MPM4b6");
        assert_eq!(transaction.value, BigUint::from(1_000_000u64));
        assert_eq!(transaction.fee, BigUint::from(6250u64));
    }

    #[test]
    fn test_transaction_transfer_sol_with_compute() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/transfer_sol_with_compute.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1750884182)).unwrap();
        let expected = Transaction::new(
            "2QeBm7G7qLmVTCVKAkbSUuZvcFjg6mBRVqaVSKWSZsTqJxHVTzMUwxDtu1Myfu8RzpUv5YMEBFFpGbwVM9ZQY8DL".to_string(),
            Chain::Solana.as_asset_id(),
            "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H".to_string(),
            "7nVDzZUjrBA3gHs3gNcHidhmR96CH7KpKsU8pyBZGHUr".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(7500u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(69000000u64),
            None,
            None,
            DateTime::from_timestamp(1750884182),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_compute_program_last() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_compute_program_last.json"))).unwrap();
        let expected = Transaction::new(
            "55cdNy5LCuvW6SC3uZstkHwAt9Tpu3EMn1LujddzX93X6LNd2VaXsbvoUMkgdyuDJ8W6MGDrucVxMGqiipf596fv".to_string(),
            Chain::Solana.as_asset_id(),
            "ASTyfSima4LLAdDgoFGkgqoKowG1LZFDr9fAQrg7iaJZ".to_string(),
            "BEMoVoNd74UEaRsyKsqhQvwawtm3qDKMKWUZtwKWVT3d".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(8125u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(66825800u64),
            None,
            None,
            DateTime::from_timestamp(1786173123),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_durable_nonce() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_durable_nonce.json"))).unwrap();
        let expected = Transaction::new(
            "3QQuTBRaWbKMDRrLb1xy4VAwUsvbhqanSc38qQMjZo6JT34qs9GXmkJm6BWk4xAvVWADXAGdrnneQna9tFDXtaA4".to_string(),
            Chain::Solana.as_asset_id(),
            "5HQZd9ovzAF1TLnHRAq1zcSnXC9HAp3EwhoxMHvo8rxB".to_string(),
            "Bnm4JEZAgvk8FE8ex9da61Wv1WQyJbr71DZvwwdXyXa".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(10035u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(15_577_216_950u64),
            None,
            None,
            DateTime::from_timestamp(1795291836),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_fee_payer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_fee_payer.json"))).unwrap();
        let expected = Transaction::new(
            "5a5jXE94xYxB85zSbSwxzsPfxx9w8fgCyb8HsFv6hykLkoq4tnjWuvp3iwQFbakLwFAzqoQr1SWLGgcqsbVBe6XT".to_string(),
            Chain::Solana.as_asset_id(),
            "B48kNVXs4YK4amkBCH2XokQiv1SeiVQGHDR17xDeKAAn".to_string(),
            "8LsfsSBxo81xKL37VYbCwWFrTw2hVnTEESq77qc6dMaW".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(250_000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(99_979_200u64),
            None,
            None,
            DateTime::from_timestamp(1795387778),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_reference_key() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/transfer_sol_reference_key.json"))).unwrap();
        let expected = Transaction::new(
            "4rfMPzroYf5KFvoPUGmHPsSQPAG117YivLSs7snwTHahyZzS87RFeV6kxevBsMWJUdVyjK8Lh14TK1fe1CP4fZuW".to_string(),
            Chain::Solana.as_asset_id(),
            "4dkNBGW4TSyUhZCQcTQ8ti3fzvvmbtkEZAQx4vHAKwLu".to_string(),
            "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(5000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(500_000u64),
            None,
            None,
            DateTime::from_timestamp(1795291875),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_error_maps_to_reverted() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/transaction_reverted_program_account_not_found.json")).unwrap();

        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1)).unwrap();

        assert_eq!(transaction.state, TransactionState::Reverted);
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/usdc_transfer.json"))).unwrap();
        let expected = Transaction::new(
            TEST_TRANSACTION_ID.to_string(),
            SOLANA_USDC_ASSET_ID.clone(),
            "37BenMAXFJMo3GaXKb2XLsNQXmd6VbbdShZWnwDj9D6k".to_string(),
            "3UJQqKq8Xyx4aVRmHgEwpQZiW7toYRQCTy6Bgp1RdKnK".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(5500u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(100000u64),
            None,
            None,
            DateTime::from_timestamp(1753346616),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_solana_pay_usdc_transfer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/solana_pay_usdc_transfer.json"))).unwrap();
        let expected = Transaction::new(
            "4sk4xbQPfT4YAmKHFCAxfkFvkEfuJhjf4kWHsPCGFmJS5VDRRDP3LX6P2QAbAfJcwAtYoXTXKm1R6wtr6PC2taeU".to_string(),
            SOLANA_USDC_ASSET_ID.clone(),
            "2xLHHcLBbiTHT6QmyrW9ayjFxrbAUpH91nSZKEv8ZLST".to_string(),
            "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(5000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(19000000u64),
            Some("ck:262:operator:m:1787598390".to_string()),
            None,
            DateTime::from_timestamp(1787598395),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_usdc_fee_payer() {
        let transaction = map_single_transaction(load_json_rpc_result(include_str!("../../testdata/usdc_transfer_fee_payer.json"))).unwrap();
        let expected = Transaction::new(
            "65MevEhHuXZzwQ8VftyQUmbgbs41bj2KMhf6zDM5hqsXN6jxsHYniHbi7MMWQ4kitcgfRdsuYe1s7zAqtzPYXZVG".to_string(),
            SOLANA_USDC_ASSET_ID.clone(),
            "5QtyKPHtWUf45bNb5buQ6UxpL2ekSJxBCK9g8xMCsc9U".to_string(),
            "B1nzrk99FEDAYB2M82yepdvEv1YBRJKcx5Y5R6MSDW1Q".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(10000u64),
            Chain::Solana.as_asset_id(),
            BigUint::from(24737625u64),
            None,
            None,
            DateTime::from_timestamp(1774244726),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_get_transaction_status() {
        let result: JsonRpcResult<SolanaTransaction> = serde_json::from_str(include_str!("../../testdata/transaction_state_transfer_sol.json")).unwrap();
        let transaction = result.result;

        let state = if transaction.slot > 0 {
            if transaction.meta.has_error() { TransactionState::Reverted } else { TransactionState::Confirmed }
        } else {
            TransactionState::Pending
        };

        assert_eq!(state, TransactionState::Confirmed);
        assert_eq!(transaction.slot, 361169359);
    }

    #[test]
    fn test_transaction_chainflip_vault_swap() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/chainflip_vault_swap.json")).unwrap();
        let transaction = map_transaction(&result.result, DateTime::from_timestamp(1772283531)).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.from, "CabroWmzUzcqqGvprUoC7RnJznuwX6qf5W1tSSaomri7");
        assert_eq!(transaction.contract, Some("J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT".to_string()));
        assert_eq!(transaction.value, BigUint::from(152686560u64));
    }

    #[test]
    fn test_transaction_broadcast_error() {
        let error_response: JsonRpcErrorResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_swap_error.json")).unwrap();

        assert_eq!(error_response.error.code, -32002);
        assert_eq!(error_response.error.message, "Transaction simulation failed: Error processing Instruction 3: custom program error: 0x1771");
        assert_eq!(error_response.id, Some(1755839259));
    }
}
