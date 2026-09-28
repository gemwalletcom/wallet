mod transaction;

use std::str::FromStr;

use alloy_primitives::Address;
use gem_evm::encode::{encode_erc20_approve_max_value, encode_erc20_transfer};
use gem_evm::signer::{EvmChainSigner, TransactionParams};
use num_bigint::BigUint;
use primitives::{Chain, ChainSigner, SignerError, SignerInput, decode_hex, swap::SwapQuoteDataType};

use transaction::{TempoTransaction, TransactionCall};

pub struct TempoSigner;

impl ChainSigner for TempoSigner {
    fn sign_transfer(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::invalid_input("Tempo does not support native transfers"))
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let token_id = input.input_type.get_asset().id.get_token_id()?;
        let transfer_call = TransactionCall::new(token_id, encode_erc20_transfer(&input.destination_address, &input.value_as_bigint())?)?;
        sign_calls(input, vec![transfer_call], input.fee.gas_limit()?, private_key)
    }

    fn sign_token_approval(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let approval = input.input_type.get_approval_data()?;
        let approve_call = TransactionCall::new(&approval.token, encode_erc20_approve_max_value(&approval.spender)?)?;
        sign_calls(input, vec![approve_call], input.fee.gas_limit()?, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = &input.input_type.get_swap_data()?.data;
        match swap_data.data_type {
            SwapQuoteDataType::Transfer => {
                let token_id = input.input_type.get_asset().id.get_token_id()?;
                let transfer_call = TransactionCall::new(token_id, encode_erc20_transfer(&swap_data.to, &input.value_as_bigint())?)?;
                Ok(vec![sign_calls(input, vec![transfer_call], input.fee.gas_limit()?, private_key)?])
            }
            SwapQuoteDataType::Contract => {
                require_zero_call_value(&swap_data.value)?;
                let swap_gas_limit = input.swap_gas_limit()?;
                let swap_call = TransactionCall::new(&swap_data.to, decode_hex(&swap_data.data)?)?;
                let (calls, gas_limit) = match &swap_data.approval {
                    Some(approval) => {
                        let approve_call = TransactionCall::new(&approval.token, encode_erc20_approve_max_value(&approval.spender)?)?;
                        (vec![approve_call, swap_call], input.fee.gas_limit()? + swap_gas_limit)
                    }
                    None => (vec![swap_call], swap_gas_limit),
                };
                Ok(vec![sign_calls(input, calls, gas_limit, private_key)?])
            }
        }
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data()?;
        require_zero_call_value(&input.value)?;
        let gas_limit = match &extra.gas_limit {
            Some(gas_limit) => u64::try_from(gas_limit).map_err(SignerError::from_display)?,
            None => input.fee.gas_limit()?,
        };
        let call = TransactionCall::new(&extra.to, extra.data.clone().unwrap_or_default())?;
        sign_calls(input, vec![call], gas_limit, private_key)
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        EvmChainSigner.sign_message(message, private_key)
    }
}

fn sign_calls(input: &SignerInput, calls: Vec<TransactionCall>, gas_limit: u64, private_key: &[u8]) -> Result<String, SignerError> {
    let params = TransactionParams::from_input(input)?;
    let transaction = TempoTransaction {
        chain_id: params.chain_id,
        max_priority_fee_per_gas: params.max_priority_fee_per_gas,
        max_fee_per_gas: params.max_fee_per_gas,
        gas_limit,
        nonce: params.nonce,
        fee_token: get_fee_token(input)?,
        calls,
    };
    Ok(hex::encode(transaction.sign(private_key)?))
}

fn require_zero_call_value(value: &BigUint) -> Result<(), SignerError> {
    if *value != BigUint::ZERO {
        return Err(SignerError::invalid_input("Tempo's CALLVALUE is always 0; value must route through a TIP-20 call, not msg.value"));
    }
    Ok(())
}

fn get_fee_token(input: &SignerInput) -> Result<Address, SignerError> {
    let fee_asset = &input.fee.fee_asset;
    if fee_asset.chain != Chain::Tempo || input.input_type.get_asset().chain() != Chain::Tempo {
        return Err(SignerError::invalid_input("mismatched Tempo fee asset"));
    }
    Address::from_str(fee_asset.get_token_id()?).map_err(SignerError::from_display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_ROUTER_ADDRESS, mock_tempo_generic_input, mock_tempo_signer_input, mock_tempo_swap_input};
    use gem_evm::constants::TOKEN_TRANSFER_GAS_LIMIT;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{
        Asset, AssetId, AssetType, Chain, TransactionInputType,
        asset_constants::{TEMPO_BRIDGED_USDC_TOKEN_ID, TEMPO_PATHUSD_TOKEN_ID, TEMPO_USDT0_ASSET_ID, TEMPO_USDT0_TOKEN_ID},
        known_assets::{TEMPO_BRIDGED_USDC, TEMPO_PATHUSD},
        swap::{ApprovalData, SwapData, SwapQuoteData},
    };

    #[test]
    fn test_rejects_native_transfer() {
        let input = mock_tempo_signer_input(
            TransactionInputType::Transfer { asset: Asset::mock_with_chain(Chain::Tempo) },
            "1000000",
            TOKEN_TRANSFER_GAS_LIMIT,
            TEMPO_USDT0_ASSET_ID.clone(),
        );
        assert_eq!(TempoSigner.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap_err(), SignerError::invalid_input("Tempo does not support native transfers"));
    }

    #[test]
    fn test_sign_token_transfer() {
        let input = mock_tempo_signer_input(TransactionInputType::Transfer { asset: TEMPO_PATHUSD.clone() }, "10000", TOKEN_TRANSFER_GAS_LIMIT, TEMPO_USDT0_ASSET_ID.clone());
        assert_eq!(
            TempoSigner.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "76f8d0821079843b9aca008504a817c80082fde8f85ef85c9420c000000000000000000000000000000000000080b844a9059cbb0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf0000000000000000000000000000000000000000000000000000000000002710c0808080809420c00000000000000000000014f22ca97301eb7380c0b841d37f1f194cb38a8c1411bfc89677ec4f1f48a06ed9c5bf8ef6e12c0ef73f00ed15fafcb85062d137d9b55038c67c8d60aa7275cd2e1a9a64f781b60699b03db01c"
        );
    }

    #[test]
    fn test_sign_token_approval() {
        let input = mock_tempo_signer_input(
            TransactionInputType::TokenApprove {
                asset: TEMPO_BRIDGED_USDC.clone(),
                approval_data: ApprovalData::mock(),
            },
            "0",
            TOKEN_TRANSFER_GAS_LIMIT,
            TEMPO_USDT0_ASSET_ID.clone(),
        );
        assert_eq!(
            TempoSigner.sign_token_approval(&input, &TEST_PRIVATE_KEY).unwrap(),
            "76f8d0821079843b9aca008504a817c80082fde8f85ef85c94dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc0808080809420c00000000000000000000014f22ca97301eb7380c0b8413518aae0ec2ae1ea63093271ec8349dbcb69ac49a337476f049396326abe84621c8814d8724eee73b4bb29a213a6a90d7703db5964011d64905c0f8d0c2d8ed71c"
        );
    }

    #[test]
    fn test_sign_data() {
        let input = mock_tempo_signer_input(mock_tempo_generic_input(TEMPO_TEST_ROUTER_ADDRESS, vec![0xab, 0xcd]), "0", 100_000, TEMPO_USDT0_ASSET_ID.clone());
        assert_eq!(
            TempoSigner.sign_data(&input, &TEST_PRIVATE_KEY).unwrap(),
            "76f88c821079843b9aca008504a817c800830186a0dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c00000000000000000000014f22ca97301eb7380c0b841c04a35dd140d1713a54fcdc2ed189b611f4be76c2e1efa68614b3190375e989b030cbbacd46652c0f4f52166ec832cb3e8639eaf6c7d9a77a349ca837df10c5f1c"
        );

        let value_input = mock_tempo_signer_input(mock_tempo_generic_input(TEMPO_TEST_ROUTER_ADDRESS, vec![0xab, 0xcd]), "1", 100_000, TEMPO_USDT0_ASSET_ID.clone());
        assert_eq!(
            TempoSigner.sign_data(&value_input, &TEST_PRIVATE_KEY).unwrap_err(),
            SignerError::invalid_input("Tempo's CALLVALUE is always 0; value must route through a TIP-20 call, not msg.value")
        );
    }

    #[test]
    fn test_get_fee_token() {
        let usdc = TEMPO_BRIDGED_USDC.clone();
        let user_token = TEMPO_USDT0_TOKEN_ID;

        let token_input = mock_tempo_swap_input(usdc.clone(), usdc.id.clone(), None);
        assert_eq!(get_fee_token(&token_input).unwrap(), TEMPO_BRIDGED_USDC_TOKEN_ID.parse::<Address>().unwrap());

        let user_token_input = mock_tempo_swap_input(
            usdc.clone(),
            Asset::mock_with_params(Chain::Tempo, Some(user_token.to_string()), "User USD".to_string(), "USD".to_string(), 6, AssetType::TIP20).id,
            None,
        );
        assert_eq!(get_fee_token(&user_token_input).unwrap(), user_token.parse::<Address>().unwrap());

        let pathusd_input = mock_tempo_swap_input(usdc.clone(), TEMPO_PATHUSD.id.clone(), None);
        assert_eq!(get_fee_token(&pathusd_input).unwrap(), TEMPO_PATHUSD_TOKEN_ID.parse::<Address>().unwrap());

        let native_input = mock_tempo_swap_input(usdc.clone(), AssetId::from_chain(Chain::Tempo), None);
        assert!(get_fee_token(&native_input).is_err());

        let wrong_chain_input = mock_tempo_swap_input(usdc, AssetId::from_chain(Chain::Ethereum), None);
        assert!(get_fee_token(&wrong_chain_input).is_err());

        let ethereum_input = mock_tempo_swap_input(Asset::mock_eth(), AssetId::from_chain(Chain::Ethereum), None);
        assert!(get_fee_token(&ethereum_input).is_err());
    }

    #[test]
    fn test_sign_swap() {
        let signer = TempoSigner;
        let usdc = TEMPO_BRIDGED_USDC.clone();

        let input = mock_tempo_swap_input(usdc.clone(), usdc.id.clone(), None);
        assert_eq!(
            signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "76f88c821079843b9aca008504a817c8008307a120dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000b9537d11c60e8b5080c0b841ed4158d43d9de7697b3d19905fea7b291143cbcc0378cf9e1be6d27db8e97dae7fe047d546325b111fd00a89f6efe3ae62cb86e362820cb81968e18d0c79862c1b"
            ]
        );

        let input_with_approval = mock_tempo_swap_input(usdc.clone(), usdc.id, Some(ApprovalData::mock()));
        assert_eq!(
            signer.sign_swap(&input_with_approval, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "76f8eb821079843b9aca008504a817c80083089f08f878f85c94dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000b9537d11c60e8b5080c0b8415544316cf9789d3ba7ec06c96f68adb8703b8485fb31488f6af89b46bb77592301edc6d089d612a20c7413f8460eeafb08e517a3018c12396504772b5521991c1b"
            ]
        );

        let pathusd_input = mock_tempo_swap_input(TEMPO_PATHUSD.clone(), TEMPO_PATHUSD.id.clone(), None);
        assert_eq!(
            signer.sign_swap(&pathusd_input, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "76f88c821079843b9aca008504a817c8008307a120dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000000000000000000080c0b841bb18fb84e469f215edd9283dcb13d1d564bef790dd861b9abc89a145c76af3d04b3d9f6a5b509cc72010ee3258f058ac2e68b1dc6fea615e2a57cadbac9d3b3a1b"
            ]
        );

        let transfer_input = mock_tempo_signer_input(
            TransactionInputType::Swap {
                from_asset: TEMPO_PATHUSD.clone(),
                to_asset: TEMPO_BRIDGED_USDC.clone(),
                swap_data: SwapData {
                    data: SwapQuoteData {
                        to: TEMPO_TEST_ROUTER_ADDRESS.to_string(),
                        data_type: SwapQuoteDataType::Transfer,
                        ..SwapQuoteData::mock()
                    },
                    ..SwapData::mock()
                },
            },
            "10000",
            TOKEN_TRANSFER_GAS_LIMIT,
            TEMPO_USDT0_ASSET_ID.clone(),
        );
        assert_eq!(
            signer.sign_swap(&transfer_input, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "76f8d0821079843b9aca008504a817c80082fde8f85ef85c9420c000000000000000000000000000000000000080b844a9059cbb000000000000000000000000a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea910000000000000000000000000000000000000000000000000000000000002710c0808080809420c00000000000000000000014f22ca97301eb7380c0b841b465c58835ea5cf8d453763209ba3493b21e9acdf2f6ac17a91f5c86de983e18403d83e3628180b1d112ef100148167cbcd08f5d64112e5d2067a937b5ec73d71c"
            ]
        );
    }
}
