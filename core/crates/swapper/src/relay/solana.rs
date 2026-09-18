use std::sync::Arc;

use gem_solana::{
    AddressLookupTableAccount, DEFAULT_SWAP_GAS_LIMIT, HexInstructionData, SolanaAddress, SolanaClient, compute_budget, encode_v0_transaction, encode_v1_transaction,
    instructions_from_primitives,
};
use num_bigint::BigUint;
use primitives::Chain;

use super::model::SolanaStepData;
use crate::{SwapperError, SwapperQuoteData, alien::RpcProvider, client_factory::create_client_with_chain};

pub async fn build_quote_data(wallet_address: &str, step: &SolanaStepData, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
    let client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana)?);
    let blockhash = client.get_latest_blockhash().await?.value.blockhash;
    if let Some(quote_data) = build_v1_transaction(wallet_address, step, &blockhash)? {
        return Ok(quote_data);
    }
    let lookup_tables = client
        .get_address_lookup_tables(step.address_lookup_table_addresses.clone())
        .await
        .map_err(SwapperError::transaction_error)?;
    build_v0_transaction(wallet_address, step, &blockhash, &lookup_tables)
}

fn build_v1_transaction(wallet_address: &str, step: &SolanaStepData, blockhash: &str) -> Result<Option<SwapperQuoteData>, SwapperError> {
    let instructions = instructions_from_primitives::<HexInstructionData>(step.instructions.clone()).map_err(SwapperError::transaction_error)?;
    let fee_payer = SolanaAddress::parse(wallet_address).map_err(SwapperError::transaction_error)?.into();
    let compute_unit_limit = compute_budget::get_compute_unit_limit(&instructions);
    let data = encode_v1_transaction(fee_payer, blockhash, &instructions, compute_unit_limit.unwrap_or(DEFAULT_SWAP_GAS_LIMIT)).map_err(SwapperError::transaction_error)?;
    Ok(data.map(|data| SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, data, None, compute_unit_limit.map(|limit| limit.to_string()))))
}

fn build_v0_transaction(wallet_address: &str, step: &SolanaStepData, blockhash: &str, lookup_tables: &[AddressLookupTableAccount]) -> Result<SwapperQuoteData, SwapperError> {
    let mut instructions = instructions_from_primitives::<HexInstructionData>(step.instructions.clone()).map_err(SwapperError::transaction_error)?;
    compute_budget::ensure_compute_unit_price(&mut instructions, 0);
    let fee_payer = SolanaAddress::parse(wallet_address).map_err(SwapperError::transaction_error)?.into();
    let data = encode_v0_transaction(fee_payer, blockhash, &instructions, lookup_tables).map_err(SwapperError::transaction_error)?;
    let gas_limit = compute_budget::get_compute_unit_limit(&instructions).map(|limit| limit.to_string());
    Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, data, None, gas_limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::RelayQuoteResponse;
    use gem_solana::{
        CompiledInstruction, MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES, Pubkey, TransactionConfig, compute_budget::set_compute_unit_limit, decode_transaction,
        instructions::program_ids::compute_budget_program,
    };
    use hex_lit::hex;
    use primitives::{
        SolanaInstruction,
        contract_constants::{SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID},
    };

    const TEST_WALLET: &str = "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T";
    const TEST_BLOCKHASH: &str = "BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF";

    const DEPOSIT_INSTRUCTION_DATA: [u8; 48] = hex!("0d9e0ddf5fd51c0600e1f50500000000d8f6831d9a771a5b031cc86256987f54a9759e6107db24b07ec66eef4e555055");

    fn expected_account_keys() -> Vec<Pubkey> {
        [
            TEST_WALLET,
            "7uTT8Xi5RWXzy7h9XL244GRgEycDYDhLjr3ZyNdXi8pZ",
            SOLANA_SYSTEM_PROGRAM_ID,
            SOLANA_RELAY_DEPOSITORY_PROGRAM_ID,
            "Dodg2HifwU8rmaVVyMyUZDGTRbqAJTyVYxXPwcbNpBKc",
        ]
        .iter()
        .map(|key| Pubkey::from_base58(key).unwrap())
        .collect()
    }

    #[test]
    fn test_build_v1_transaction() {
        let step = serde_json::from_str::<RelayQuoteResponse>(include_str!("testdata/quote_sol_to_base_usdc.json"))
            .unwrap()
            .get_solana_step()
            .unwrap()
            .clone();

        let quote_data = build_v1_transaction(TEST_WALLET, &step, TEST_BLOCKHASH).unwrap().unwrap();

        let transaction = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(quote_data.gas_limit, None);
        assert_eq!(
            *transaction.transaction_config().unwrap(),
            TransactionConfig {
                priority_fee: Some(0),
                compute_unit_limit: Some(DEFAULT_SWAP_GAS_LIMIT),
                loaded_accounts_data_size_limit: Some(MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES),
                heap_size: None,
            }
        );
        assert_eq!(transaction.account_keys(), expected_account_keys());
        assert_eq!(
            transaction.instructions(),
            [CompiledInstruction {
                program_id_index: 3,
                accounts: vec![4, 0, 0, 1, 2],
                data: DEPOSIT_INSTRUCTION_DATA.to_vec(),
            }]
        );
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert!(quote_data.approval.is_none());
    }

    #[test]
    fn test_build_v1_transaction_drops_the_compute_budget_instructions() {
        let mut step = serde_json::from_str::<RelayQuoteResponse>(include_str!("testdata/quote_sol_to_base_usdc.json"))
            .unwrap()
            .get_solana_step()
            .unwrap()
            .clone();
        step.instructions.push(SolanaInstruction {
            program_id: compute_budget_program().to_base58(),
            accounts: vec![],
            data: hex::encode(set_compute_unit_limit(200_000).data),
        });

        let quote_data = build_v1_transaction(TEST_WALLET, &step, TEST_BLOCKHASH).unwrap().unwrap();

        let transaction = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(quote_data.gas_limit, Some("200000".to_string()));
        assert_eq!(transaction.get_compute_unit_limit(), Some(200_000));
        assert_eq!(transaction.account_keys(), expected_account_keys());
        assert_eq!(
            transaction.instructions(),
            [CompiledInstruction {
                program_id_index: 3,
                accounts: vec![4, 0, 0, 1, 2],
                data: DEPOSIT_INSTRUCTION_DATA.to_vec(),
            }]
        );
    }

    #[test]
    fn test_build_v0_transaction() {
        let step = serde_json::from_str::<RelayQuoteResponse>(include_str!("testdata/quote_sol_to_base_usdc.json"))
            .unwrap()
            .get_solana_step()
            .unwrap()
            .clone();

        let quote_data = build_v0_transaction(TEST_WALLET, &step, TEST_BLOCKHASH, &[]).unwrap();

        let transaction = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.get_compute_unit_price(), Some(0));
        assert_eq!(quote_data.gas_limit, None);
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert!(quote_data.approval.is_none());
    }
}
