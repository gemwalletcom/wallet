use super::broker::{SolanaVaultSwapResponse, TronVaultSwapResponse};
use crate::{SwapperError, SwapperQuoteData, alien::RpcProvider, client_factory::create_client_with_chain};
use num_bigint::BigUint;

use gem_encoding::encode_base64;
use gem_solana::{SolanaClient, try_decode_blockhash};
use gem_tron::address::TronAddress;
use primitives::{
    Chain,
    hex::{decode_hex, encode},
    swap::SwapQuoteDataType::Contract,
};
use solana_primitives::{AccountMeta, InstructionBuilder, Pubkey, TransactionBuilder};
use std::{str::FromStr, sync::Arc};

pub(super) fn build_tron_quote_data(response: &TronVaultSwapResponse, value: BigUint) -> Result<SwapperQuoteData, SwapperError> {
    let address = response.source_token_address.as_deref().unwrap_or(&response.to);
    let to = TronAddress::parse_hex_or_base58(address)?.to_string();
    let calldata = decode_hex(&response.calldata).map_err(|_| SwapperError::TransactionError("invalid Tron calldata".to_string()))?;

    Ok(SwapperQuoteData {
        to,
        data_type: Contract,
        value,
        data: encode(calldata),
        memo: Some(response.note.clone()),
        approval: None,
        gas_limit: None,
    })
}

pub(super) async fn get_solana_blockhash(provider: Arc<dyn RpcProvider>) -> Result<[u8; 32], SwapperError> {
    let client = SolanaClient::new(create_client_with_chain(provider, Chain::Solana));
    let blockhash_response = client.get_latest_blockhash().await?;
    try_decode_blockhash(&blockhash_response.value.blockhash).ok_or_else(|| SwapperError::transaction_error("Invalid Solana blockhash"))
}

pub(super) fn build_solana_transaction(fee_payer: &str, response: &SolanaVaultSwapResponse, blockhash: [u8; 32]) -> Result<String, SwapperError> {
    let fee_payer = Pubkey::from_str(fee_payer).map_err(|_| SwapperError::transaction_error("Invalid fee payer"))?;
    let program_id = Pubkey::from_str(&response.program_id).map_err(|_| SwapperError::transaction_error("Invalid program ID"))?;
    let data = decode_hex(&response.data).map_err(|_| SwapperError::transaction_error("Invalid data"))?;
    let accounts = response
        .accounts
        .iter()
        .map(|account| {
            Ok(AccountMeta {
                is_signer: account.is_signer,
                is_writable: account.is_writable,
                pubkey: Pubkey::from_str(&account.pubkey).map_err(|_| SwapperError::transaction_error("Invalid Solana account"))?,
            })
        })
        .collect::<Result<_, SwapperError>>()?;
    let instruction = InstructionBuilder::new(program_id).accounts(accounts).data(data).build();

    let mut transaction_builder = TransactionBuilder::new(fee_payer, blockhash);
    transaction_builder.add_instruction(instruction);

    let transaction = transaction_builder.build().map_err(SwapperError::transaction_error)?;
    let bytes = transaction.serialize_legacy().map_err(SwapperError::transaction_error)?;

    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::broker::{SolanaVaultSwapResponse, TronVaultSwapResponse};
    use gem_jsonrpc::types::JsonRpcResponse;
    use gem_solana::decode_transaction;
    use num_bigint::BigUint;

    #[test]
    fn test_build_tron_quote_data_maps_trc20_contract_call_fields() {
        let note = "0x0300";
        let data = build_tron_quote_data(
            &TronVaultSwapResponse {
                calldata: "0xa9059cbb".to_string(),
                value: BigUint::from(0u32),
                to: "0x2523ae929fecd9d665f472f59b99a8ce6b179510".to_string(),
                note: note.to_string(),
                source_token_address: Some("0xeca9bc828a3005b9a3b909f2cc5c2a54794de05f".to_string()),
            },
            BigUint::ZERO,
        )
        .unwrap();

        assert_eq!(data.data_type, Contract);
        assert_eq!(data.to, "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf");
        assert_eq!(data.memo, Some(note.to_string()));
        assert_eq!(data.value, BigUint::from(0u64));
        assert_eq!(data.data, "a9059cbb");

        let data = build_tron_quote_data(
            &TronVaultSwapResponse {
                calldata: "0xa9059cbb".to_string(),
                value: BigUint::from(0u32),
                to: "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi".to_string(),
                note: note.to_string(),
                source_token_address: Some("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf".to_string()),
            },
            BigUint::ZERO,
        )
        .unwrap();

        assert_eq!(data.to, "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf");
    }

    #[test]
    fn test_build_tron_quote_data_maps_native_contract_transfer_fields() {
        let note = "0x0300";
        for calldata in ["", "0x"] {
            let data = build_tron_quote_data(
                &TronVaultSwapResponse {
                    calldata: calldata.to_string(),
                    value: BigUint::from(50_000_000u32),
                    to: "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi".to_string(),
                    note: note.to_string(),
                    source_token_address: None,
                },
                BigUint::from(50_000_000u64),
            )
            .unwrap();

            assert_eq!(data.to, "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi");
            assert_eq!(data.value, BigUint::from(50000000u64));
            assert_eq!(data.data, "");
        }
    }

    #[test]
    fn test_build_solana_transaction() -> Result<(), SwapperError> {
        let wallet_address = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";
        let blockhash = try_decode_blockhash("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").unwrap();
        let response: JsonRpcResponse<SolanaVaultSwapResponse> = serde_json::from_str(include_str!("./test/chainflip_sol_arb_usdc_quote_data.json"))?;

        let tx_b64 = build_solana_transaction(wallet_address, &response.result, blockhash)?;
        let transaction = decode_transaction(&tx_b64).map_err(SwapperError::transaction_error)?;

        assert_eq!(transaction.get_compute_unit_limit(), None);

        Ok(())
    }
}
