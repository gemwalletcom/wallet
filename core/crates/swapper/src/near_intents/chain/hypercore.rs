use chrono::Utc;
use gem_evm::address::ethereum_address_checksum;
use gem_hypercore::{
    core::{actions::user::send_asset::SendAsset, hypercore::send_asset_typed_data},
    models::token::spot_token_id_for_asset_id,
};
use number_formatter::BigNumberFormatter;

use crate::{SwapperError, SwapperQuoteAsset, near_intents::model::DepositData};

pub(crate) fn build_deposit_data(from_asset: &SwapperQuoteAsset, deposit_address: &str, amount_in: &str) -> Result<DepositData, SwapperError> {
    let nonce = Utc::now().timestamp_millis().try_into().map_err(SwapperError::transaction_error)?;
    build_deposit_data_at(from_asset, deposit_address, amount_in, nonce)
}

fn build_deposit_data_at(from_asset: &SwapperQuoteAsset, deposit_address: &str, amount_in: &str, nonce: u64) -> Result<DepositData, SwapperError> {
    let to = ethereum_address_checksum(deposit_address)?;
    let amount = BigNumberFormatter::plain_value(amount_in, from_asset.decimals)?;
    let token = spot_token_id_for_asset_id(&from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
    let send_asset = SendAsset::spot(amount, to.clone(), token, nonce);
    let data = send_asset_typed_data(send_asset).map_err(SwapperError::TransactionError)?;

    Ok(DepositData {
        to,
        value: amount_in.to_string(),
        data,
        memo: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::near_intents::testkit::mock_hypercore_quote_request;
    use gem_hypercore::signer::HyperCoreSigner;
    use primitives::{
        ChainSigner, SignerInput, SwapProvider, TransactionInputType, TransactionLoadMetadata,
        known_assets::{ETHEREUM_USDC, HYPERCORE_SPOT_USDC},
        swap::SwapData,
        testkit::signer_mock::{TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS},
    };
    use serde_json::Value;

    #[test]
    fn test_build_deposit_data() {
        let from_asset = mock_hypercore_quote_request().from_asset;
        let data = build_deposit_data_at(&from_asset, "0x1085c5f70f7f7591d97da281a64688385455c2bd", "10002430500", 1755004027201).unwrap();
        let expected: Value = serde_json::from_str(include_str!("../testdata/hypercore_send_asset.json")).unwrap();

        assert_eq!(serde_json::from_str::<Value>(&data.data).unwrap(), expected);
        assert_eq!((data.to.as_str(), data.value.as_str(), data.memo), ("0x1085c5f70F7F7591D97da281A64688385455c2bD", "10002430500", None));
        assert!(build_deposit_data_at(&from_asset, "invalid", "10002430500", 1755004027201).is_err());

        let input = SignerInput::mock_with_input_type(
            TransactionInputType::Swap {
                from_asset: HYPERCORE_SPOT_USDC.clone(),
                to_asset: ETHEREUM_USDC.clone(),
                swap_data: SwapData::mock_with_provider_data(SwapProvider::NearIntents, &data.data, None),
            },
            TEST_PRIVATE_KEY_ETHEREUM_ADDRESS,
            &data.to,
            &data.value,
            TransactionLoadMetadata::None,
        );
        let signed: Vec<Value> = HyperCoreSigner.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap().iter().map(|value| serde_json::from_str(value).unwrap()).collect();
        assert_eq!(signed, vec![serde_json::from_str::<Value>(include_str!("../testdata/hypercore_send_asset_signed.json")).unwrap()]);
    }
}
