use std::str::FromStr;

use gem_encoding::encode_base64;
use gem_solana::{
    AccountMeta, BLOCKHASH_SET_AT_SIGNING, DEFAULT_SWAP_GAS_LIMIT, Instruction, InstructionBuilder, Pubkey, TransactionBuilder,
    compute_budget::{set_compute_unit_limit, set_compute_unit_price},
};
use primitives::{
    hex::{decode_hex, encode},
    swap::SwapQuoteDataType::Contract,
};

use super::{broker::SolanaVaultSwapResponse, vault_swap::TronVaultCall};
use crate::{SwapperError, SwapperQuoteData};

pub(super) fn build_tron_quote_data(note: &str, call: TronVaultCall) -> SwapperQuoteData {
    SwapperQuoteData {
        to: call.to.to_string(),
        data_type: Contract,
        value: call.value,
        data: encode(call.calldata),
        memo: Some(note.to_string()),
        approval: None,
        gas_limit: None,
        permit2: None,
    }
}

pub(super) fn solana_instruction(response: &SolanaVaultSwapResponse) -> Result<Instruction, SwapperError> {
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
    Ok(InstructionBuilder::new(program_id).accounts(accounts).data(data).build())
}

pub(super) fn build_solana_transaction(fee_payer: &str, instruction: Instruction) -> Result<String, SwapperError> {
    let fee_payer = Pubkey::from_str(fee_payer).map_err(|_| SwapperError::transaction_error("Invalid fee payer"))?;
    let mut transaction_builder = TransactionBuilder::new(fee_payer, BLOCKHASH_SET_AT_SIGNING);
    transaction_builder.add_instructions(vec![set_compute_unit_price(0), set_compute_unit_limit(DEFAULT_SWAP_GAS_LIMIT), instruction]);

    let transaction = transaction_builder.build().map_err(SwapperError::transaction_error)?;
    let bytes = transaction.serialize().map_err(SwapperError::transaction_error)?;

    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_check::solana::{StepLimits, verify_solana},
        chainflip::{broker::SolanaVaultSwapResponse, provider::SOLANA_VAULT, testkit::TRON_NOTE},
    };
    use gem_jsonrpc::types::JsonRpcResponse;
    use gem_solana::decode_transaction;
    use gem_tron::address::TronAddress;
    use num_bigint::BigUint;

    #[test]
    fn test_build_tron_quote_data_maps_trc20_contract_call_fields() {
        let data = build_tron_quote_data(
            TRON_NOTE,
            TronVaultCall {
                to: TronAddress::parse("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf").unwrap(),
                value: BigUint::ZERO,
                calldata: vec![0xa9, 0x05, 0x9c, 0xbb],
            },
        );

        assert_eq!(data.data_type, Contract);
        assert_eq!(data.to, "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf");
        assert_eq!(data.memo, Some(TRON_NOTE.to_string()));
        assert_eq!(data.value, BigUint::from(0u64));
        assert_eq!(data.data, "a9059cbb");
    }

    #[test]
    fn test_build_tron_quote_data_maps_native_contract_transfer_fields() {
        let data = build_tron_quote_data(
            TRON_NOTE,
            TronVaultCall {
                to: TronAddress::parse("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi").unwrap(),
                value: BigUint::from(50_000_000u64),
                calldata: vec![],
            },
        );

        assert_eq!(data.to, "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi");
        assert_eq!(data.value, BigUint::from(50000000u64));
        assert_eq!(data.data, "");
    }

    #[test]
    fn test_build_solana_transaction() -> Result<(), SwapperError> {
        let wallet_address = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";
        let response: JsonRpcResponse<SolanaVaultSwapResponse> = serde_json::from_str(include_str!("./test/chainflip_sol_arb_usdc_quote_data.json"))?;

        let tx_b64 = build_solana_transaction(wallet_address, solana_instruction(&response.result)?)?;
        verify_solana(&tx_b64, wallet_address, &[SOLANA_VAULT], &StepLimits::default())?;

        assert_eq!(
            tx_b64,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAQIhfupPuKcYE+oWKNRaIwBKQhB6vsZxjpwpHXTx7w7758q21EdC4D4NruUv9F26xeVqhYm0WXVWkSIjeQIxD3II9tUC6aOjrGBy017zEItREWS3QDEQI/vMhwSVTo/1e2664X/uFi6gx6sRwFnSAPu1ODmcAsu2sf8IuwYArWOf4gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAiKB2TmOdpVByNvc2jO/SqWcRJnwnp6i4PhwcXOdR2sf+adsEMvxMdgZ9RYJ0BKLVq++GfFFu+oFIYBJkEkLMJgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwUACQMAAAAAAAAAAAUABQKgaAYABwYGAQADAgS2AaMmXOLzaY3EgB0sBAAAAAAEAAAAFAAAAFFLyx+aq7kE5hBr0QUrZtJwbbu3BwAAAABsAAAAAAoAAACF+6k+4pxgT6hYo1FojAEpCEHq+xnGOnCkddPHvDvvn8qhRbbz/dR46Sb6cwLQdTEAAAAAAAAAAAAAAAAAAAAAAAAeg9KXLT3KOjMNYMJ3fuW40laDxj+jWRFphWCYMPQgVAUABAAtEQAAADiSTMM0VhiQ46gZQHNTcQ4J"
        );

        let transaction = decode_transaction(&tx_b64).unwrap();
        assert_eq!(transaction.get_compute_unit_price(), Some(0));
        assert_eq!(transaction.get_compute_unit_limit(), Some(DEFAULT_SWAP_GAS_LIMIT));

        Ok(())
    }
}
