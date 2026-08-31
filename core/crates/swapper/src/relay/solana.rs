use std::sync::Arc;

use futures::try_join;
use gem_solana::{HexInstructionData, SolanaAddress, SolanaClient, encode_v0_transaction, instructions_from_primitives};
use num_bigint::BigUint;
use primitives::Chain;
use solana_primitives::{AddressLookupTableAccount, compute_budget};

use super::model::SolanaStepData;
use crate::{SwapperError, SwapperQuoteData, alien::RpcProvider, client_factory::create_client_with_chain};

pub async fn build_quote_data(wallet_address: &str, step: &SolanaStepData, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
    let client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana));
    let lookup_tables = async {
        client
            .get_address_lookup_tables(step.address_lookup_table_addresses.clone())
            .await
            .map_err(SwapperError::transaction_error)
    };
    let blockhash = async { client.get_latest_blockhash().await.map(|response| response.value.blockhash).map_err(SwapperError::from) };
    let (lookup_tables, blockhash) = try_join!(lookup_tables, blockhash)?;
    build_transaction(wallet_address, step, &blockhash, &lookup_tables)
}

fn build_transaction(wallet_address: &str, step: &SolanaStepData, blockhash: &str, lookup_tables: &[AddressLookupTableAccount]) -> Result<SwapperQuoteData, SwapperError> {
    let instructions = instructions_from_primitives::<HexInstructionData>(step.instructions.clone()).map_err(SwapperError::transaction_error)?;
    let fee_payer = SolanaAddress::parse(wallet_address).map_err(SwapperError::transaction_error)?.into();
    let data = encode_v0_transaction(fee_payer, blockhash, &instructions, lookup_tables).map_err(SwapperError::transaction_error)?;
    let gas_limit = compute_budget::get_compute_unit_limit(&instructions).map(|limit| limit.to_string());
    Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, data, None, gas_limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::RelayQuoteResponse;
    use gem_solana::decode_transaction;

    #[test]
    fn test_build_transaction() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_sol_to_base_usdc.json")).unwrap();
        let step = response.get_solana_step().unwrap();

        let quote_data = build_transaction("4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T", step, "BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF", &[]).unwrap();

        let transaction = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(transaction.num_required_signatures(), 1);
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert!(quote_data.approval.is_none());

        let instructions = instructions_from_primitives::<HexInstructionData>(step.instructions.clone()).unwrap();
        assert_eq!(instructions[0].program_id.to_base58(), "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2");
        assert_eq!(&instructions[0].data[8..12], 100_000_000u32.to_le_bytes().as_slice());
    }
}
