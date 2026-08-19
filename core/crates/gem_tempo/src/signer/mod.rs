mod transaction;

use std::str::FromStr;

use alloy_primitives::{Address, Bytes, U256};
use gem_evm::encode::encode_erc20_approve_max_value;
use gem_evm::signer::{EvmSigner, TransactionParams};
use primitives::{SignerError, SignerInput, decode_hex};

use transaction::{TempoTransaction, TransactionCall};

pub struct TempoSigner;

impl EvmSigner for TempoSigner {
    fn sign_transfer(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::invalid_input("Tempo does not support native transfers"))
    }

    fn sign_swap_contract(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = &input.input_type.get_swap_data()?.data;
        let value = U256::from_str(&swap_data.value).map_err(SignerError::from_display)?;
        if value != U256::ZERO {
            return Err(SignerError::invalid_input(
                "Tempo's CALLVALUE is always 0; swap value must route through the ERC-20 call, not msg.value",
            ));
        }
        let params = TransactionParams::from_input(input)?;
        let swap_gas_limit = input.swap_gas_limit()?;
        let swap_call = TransactionCall::new(
            Address::from_str(&swap_data.to).map_err(SignerError::from_display)?,
            Bytes::from(decode_hex(&swap_data.data)?),
        );
        let (calls, gas_limit) = match &swap_data.approval {
            Some(approval) => {
                let approve_call = TransactionCall::new(
                    Address::from_str(&approval.token).map_err(SignerError::from_display)?,
                    Bytes::from(encode_erc20_approve_max_value(&approval.spender)?),
                );
                (vec![approve_call, swap_call], params.gas_limit + swap_gas_limit)
            }
            None => (vec![swap_call], swap_gas_limit),
        };

        let transaction = TempoTransaction {
            chain_id: params.chain_id,
            max_priority_fee_per_gas: params.max_priority_fee_per_gas,
            max_fee_per_gas: params.max_fee_per_gas,
            gas_limit,
            nonce: params.nonce,
            fee_token: get_fee_token(input)?,
            calls,
        };
        Ok(vec![hex::encode(transaction.sign(private_key)?)])
    }
}

fn get_fee_token(input: &SignerInput) -> Result<Address, SignerError> {
    let fee_asset = &input.fee.fee_asset;
    if fee_asset.chain != primitives::Chain::Tempo || input.input_type.get_asset().chain != primitives::Chain::Tempo {
        return Err(SignerError::invalid_input("mismatched Tempo fee asset"));
    }
    Address::from_str(fee_asset.get_token_id()?).map_err(SignerError::from_display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::mock_tempo_swap_input;
    use gem_evm::{constants::TOKEN_TRANSFER_GAS_LIMIT, signer::EvmChainSigner};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{
        Asset, AssetId, AssetType, Chain, ChainSigner, TransactionInputType, TransactionLoadMetadata,
        asset_constants::TEMPO_USDC_TOKEN_ID,
        known_assets::{TEMPO_PATHUSD, TEMPO_USDC},
    };

    #[test]
    fn test_rejects_native_transfer() {
        let signer = EvmChainSigner::new(TempoSigner);
        let metadata = TransactionLoadMetadata::mock_evm(0, Chain::Tempo.network_id().parse().unwrap());
        let input = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo)),
            "1000000",
            TOKEN_TRANSFER_GAS_LIMIT,
            metadata,
        );
        assert_eq!(
            signer.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap_err(),
            SignerError::invalid_input("Tempo does not support native transfers")
        );
    }

    #[test]
    fn test_get_fee_token() {
        let usdc = TEMPO_USDC.clone();
        let user_token = crate::testkit::TEMPO_TEST_USER_FEE_TOKEN;

        let token_input = mock_tempo_swap_input(usdc.clone(), usdc.id.clone(), None);
        assert_eq!(get_fee_token(&token_input).unwrap(), TEMPO_USDC_TOKEN_ID.parse::<Address>().unwrap());

        let user_token_input = mock_tempo_swap_input(
            usdc.clone(),
            Asset::mock_with_params(Chain::Tempo, Some(user_token.to_string()), "User USD".to_string(), "USD".to_string(), 6, AssetType::TIP20).id,
            None,
        );
        assert_eq!(get_fee_token(&user_token_input).unwrap(), user_token.parse::<Address>().unwrap());

        let pathusd_input = mock_tempo_swap_input(usdc.clone(), TEMPO_PATHUSD.id.clone(), None);
        assert_eq!(
            get_fee_token(&pathusd_input).unwrap(),
            TEMPO_PATHUSD.token_id.as_deref().unwrap().parse::<Address>().unwrap()
        );

        let native_input = mock_tempo_swap_input(usdc.clone(), AssetId::from_chain(Chain::Tempo), None);
        assert!(get_fee_token(&native_input).is_err());

        let wrong_chain_input = mock_tempo_swap_input(usdc.clone(), AssetId::from_chain(Chain::Ethereum), None);
        assert!(get_fee_token(&wrong_chain_input).is_err());

        let ethereum_input = mock_tempo_swap_input(Asset::mock_eth(), AssetId::from_chain(Chain::Ethereum), None);
        assert!(get_fee_token(&ethereum_input).is_err());
    }

    #[test]
    fn test_sign_swap() {
        let signer = EvmChainSigner::new(TempoSigner);
        let usdc = TEMPO_USDC.clone();

        let input = mock_tempo_swap_input(usdc.clone(), usdc.id.clone(), None);
        assert_eq!(
            signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "76f88c821079843b9aca008504a817c8008307a120dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000b9537d11c60e8b5080c0b841ed4158d43d9de7697b3d19905fea7b291143cbcc0378cf9e1be6d27db8e97dae7fe047d546325b111fd00a89f6efe3ae62cb86e362820cb81968e18d0c79862c1b"
            ]
        );

        let input_with_approval = mock_tempo_swap_input(usdc.clone(), usdc.id.clone(), Some(primitives::swap::ApprovalData::mock()));
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
    }
}
