use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetSubtype, ChainSigner, SignerError, SignerInput, TransactionSwapReferralFee,
    swap::{SwapQuoteData, SwapQuoteDataType, XRP_REFERRAL_ADDRESS, XRP_REFERRAL_FEE_ASSET_IDS},
};

use crate::address::XrpAddress;
use crate::models::XrpSwap;
use crate::signer::amount::XrpAmount;
use crate::signer::transaction::{XrpExchange, XrpOperation, XrpPaymentMemo, XrpTransaction, XrpTransactionParams};

const LEDGER_SEQUENCE_OFFSET: u64 = 24;
const TRUST_LINE_LIMIT: &str = "690000000000";

#[derive(Default)]
pub struct XrpChainSigner;

impl ChainSigner for XrpChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = XrpAmount::native(&input.value.to_string())?;

        sign_payment(input, private_key, amount, &input.destination_address, payment_memo(input.get_memo())?)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = asset_amount(input.input_type.get_asset(), &input.value.to_string())?;
        sign_payment(input, private_key, amount, &input.destination_address, payment_memo(input.get_memo())?)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap = input.input_type.get_swap_data()?;
        let signed = match swap.data.data_type {
            SwapQuoteDataType::Contract => sign_swap_batch(input, private_key, &serde_json::from_str(&swap.data.data)?)?,
            SwapQuoteDataType::Transfer => {
                let amount = asset_amount(input.input_type.get_asset(), &swap.data.value.to_string())?;
                sign_payment(input, private_key, amount, &swap.data.to, swap_memo(&swap.data))?
            }
        };
        Ok(vec![signed])
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = trust_line_limit_amount(input.input_type.get_asset())?;
        XrpTransaction::new_trust_set(params(input, private_key)?, amount).sign(private_key)
    }
}

fn sign_swap_batch(input: &SignerInput, private_key: &[u8], swap: &XrpSwap) -> Result<String, SignerError> {
    let params = params(input, private_key)?;
    let (from_asset, to_asset) = (input.input_type.get_asset(), input.input_type.get_recipient_asset());
    if swap.input_value(&from_asset.id) != input.value {
        return SignerError::invalid_input_err("XRP swap amount does not match the amount paid");
    }
    let trust_set = match swap.trust_line {
        true => Some(XrpOperation::TrustSet {
            limit_amount: trust_line_limit_amount(to_asset)?,
        }),
        false => None,
    };
    let exchange = XrpExchange {
        send_max: asset_amount(from_asset, &swap.send_max.to_string())?,
        deliver_min: asset_amount(to_asset, &swap.deliver_min.to_string())?,
        paths: swap.paths.clone(),
    };
    let payment = XrpOperation::new_exchange(&params.account, asset_amount(to_asset, &swap.amount.to_string())?, exchange);
    let fee = swap.fee.as_ref().map(|fee| referral_payment(fee, [from_asset, to_asset])).transpose()?;
    let transaction = match <[XrpOperation; 1]>::try_from(trust_set.into_iter().chain([payment]).chain(fee).collect::<Vec<_>>()) {
        Ok([operation]) => XrpTransaction::new(params, operation),
        Err(operations) => XrpTransaction::new_batch(params, operations)?,
    };
    transaction.sign(private_key)
}

fn referral_payment(fee: &TransactionSwapReferralFee, swapped: [&Asset; 2]) -> Result<XrpOperation, SignerError> {
    let asset = swapped
        .into_iter()
        .find(|asset| asset.id == fee.asset_id && XRP_REFERRAL_FEE_ASSET_IDS.contains(&asset.id))
        .ok_or_else(|| SignerError::invalid_input("XRP swap fee must be XRP or RLUSD, sold or bought"))?;
    XrpOperation::new_payment(asset_amount(asset, &fee.value.to_string())?, XRP_REFERRAL_ADDRESS, XrpPaymentMemo::None)
}

fn sign_payment(input: &SignerInput, private_key: &[u8], amount: XrpAmount, destination: &str, memo: XrpPaymentMemo) -> Result<String, SignerError> {
    XrpTransaction::new_payment(params(input, private_key)?, amount, destination, memo)?.sign(private_key)
}

fn params(input: &SignerInput, private_key: &[u8]) -> Result<XrpTransactionParams, SignerError> {
    let block_number = input.metadata.get_block_number()?;
    let sequence = input.metadata.get_sequence()?;
    let last_ledger_sequence = block_number.checked_add(LEDGER_SEQUENCE_OFFSET).ok_or_else(|| SignerError::invalid_input("XRP last ledger sequence overflow"))?;

    Ok(XrpTransactionParams {
        account: XrpAddress::parse(&input.sender_address)?,
        fee: input.fee.fee.to_u64().ok_or_else(|| SignerError::invalid_input("invalid XRP fee"))?,
        sequence: u32::try_from(sequence).map_err(SignerError::from_display)?,
        last_ledger_sequence: u32::try_from(last_ledger_sequence).map_err(SignerError::from_display)?,
        signing_pub_key: ::signer::secp256k1_public_key(private_key)?,
    })
}

fn asset_amount(asset: &Asset, value: &str) -> Result<XrpAmount, SignerError> {
    match asset.id.token_subtype() {
        AssetSubtype::NATIVE => XrpAmount::native(value),
        AssetSubtype::TOKEN => {
            let value = BigNumberFormatter::value(value, asset.decimals).map_err(SignerError::from_display)?;
            XrpAmount::issued(&value, &asset.symbol, asset.id.get_token_id()?)
        }
    }
}

fn trust_line_limit_amount(asset: &Asset) -> Result<XrpAmount, SignerError> {
    XrpAmount::issued(TRUST_LINE_LIMIT, &asset.symbol, asset.id.get_token_id()?)
}

fn payment_memo(memo: Option<&str>) -> Result<XrpPaymentMemo, SignerError> {
    let Some(memo) = memo.map(str::trim).filter(|memo| !memo.is_empty()) else {
        return Ok(XrpPaymentMemo::None);
    };

    match memo.parse::<u64>() {
        Ok(value) => Ok(XrpPaymentMemo::DestinationTag(u32::try_from(value).map_err(SignerError::from_display)?)),
        Err(_) => Ok(XrpPaymentMemo::Memo(memo.strip_prefix("0x").unwrap_or(memo).as_bytes().to_vec())),
    }
}

fn swap_memo(data: &SwapQuoteData) -> XrpPaymentMemo {
    if let Some(memo) = data.memo.as_deref().filter(|memo| !memo.is_empty()) {
        return XrpPaymentMemo::Memo(memo.as_bytes().to_vec());
    }
    if data.data.is_empty() {
        return XrpPaymentMemo::None;
    }
    XrpPaymentMemo::Memo(data.data.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{
        AccountDataType, Asset, AssetId, Chain, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadMetadata,
        asset_constants::{XRP_RLUSD_ASSET_ID, XRP_USDC_ASSET_ID},
        swap::{SwapData, SwapQuoteData},
    };

    use super::*;
    use crate::models::XrpPathStep;

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/62ef27c56c6769b3aec3c5167d925dc085646a5c/tests/chains/XRP/TWAnySignerTests.cpp#L134-L152
    const RLUSD_PRIVATE_KEY: &str = "574e99f7946cfa2a6ca9368ca72fd37e42583cddb9ecc746aa4cb194ef4b2480";
    const RLUSD_SENDER: &str = "rDgEGKXWkHHr1HYq2ETnNAs9MdV4R8Gyt";
    const RLUSD_DESTINATION: &str = "r4oPb529jpRA1tVTDARmBuZPYB2CJjKFac";
    const RLUSD_VALUE: u64 = 1_000_000_000_000_000;
    const RLUSD_SIGNED_PAYMENT: &str = "12000022000000002405955dc7201b0b2abbbe61d4838d7ea4c68000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e68400000000000000c7321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d744630440220552e90f417c2cabe39368bb45cf7495ba6ebe395f259a6509c9f3a7296e76a0d02201b37dae0c4c77fa70a451cd4a61c10575c8b052c282c082a32c229e7624a05e381140265c09d122fab2a261a80ee59f1f4cd8fba8cf88314ef20a3d93b00cc729eec11a3058d3d1feb4465e0";

    fn rlusd_payment_input(input_type: TransactionInputType, destination: &str) -> SignerInput {
        SignerInput {
            fee: TransactionFee::new_from_fee(12.into(), AssetId::from_chain(Chain::Xrp)),
            ..SignerInput::mock_with_input_type(
                input_type,
                RLUSD_SENDER,
                destination,
                &RLUSD_VALUE.to_string(),
                TransactionLoadMetadata::Xrp {
                    sequence: 93_674_951,
                    block_number: 187_349_950 - LEDGER_SEQUENCE_OFFSET,
                    is_destination_address_exist: true,
                },
            )
        }
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/62ef27c56c6769b3aec3c5167d925dc085646a5c/tests/chains/XRP/TWAnySignerTests.cpp#L17-L33
    #[test]
    fn test_sign_transfer_matches_wallet_core() {
        let private_key = hex::decode("a5576c0f63da10e584568c8d134569ff44017b0a249eb70657127ae04f38cc77").unwrap();
        let input = SignerInput {
            fee: TransactionFee::new_from_fee(10.into(), AssetId::from_chain(Chain::Xrp)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Xrp) },
                "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq",
                "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF",
                "10",
                TransactionLoadMetadata::Xrp {
                    sequence: 32_268_248,
                    block_number: 32_268_269 - LEDGER_SEQUENCE_OFFSET,
                    is_destination_address_exist: true,
                },
            )
        };

        assert_eq!(
            XrpChainSigner.sign_transfer(&input, &private_key).unwrap(),
            "12000022000000002401ec5fd8201b01ec5fed61400000000000000a68400000000000000a732103d13e1152965a51a4a9fd9a8b4ea3dd82a4eba6b25fcad5f460a2342bb650333f74463044022037d32835c9394f39b2cfd4eaf5b0a80e0db397ace06630fa2b099ff73e425dbc02205288f780330b7a88a1980fa83c647b5908502ad7de9a44500c08f0750b0d9e8481144c55f5a78067206507580be7bb2686c8460adff983148132e4e20aecf29090ac428a9c43f230a829220d"
        );
    }

    #[test]
    fn test_sign_token_transfer_matches_wallet_core_custom_currency() {
        let private_key = hex::decode(RLUSD_PRIVATE_KEY).unwrap();
        let input = rlusd_payment_input(TransactionInputType::Transfer { asset: Asset::mock_xrp_rlusd() }, RLUSD_DESTINATION);

        assert_eq!(XrpChainSigner.sign_token_transfer(&input, &private_key).unwrap(), RLUSD_SIGNED_PAYMENT);
    }

    #[test]
    fn test_destination_tag_and_memo() {
        assert_eq!(payment_memo(Some("123")).unwrap(), XrpPaymentMemo::DestinationTag(123));
        assert_eq!(payment_memo(Some("memo")).unwrap(), XrpPaymentMemo::Memo(b"memo".to_vec()));
        assert_eq!(payment_memo(Some("0xhello")).unwrap(), XrpPaymentMemo::Memo(b"hello".to_vec()));
        assert_eq!(payment_memo(Some("0")).unwrap(), XrpPaymentMemo::DestinationTag(0));
        assert_eq!(payment_memo(Some(" 123456\n")).unwrap(), XrpPaymentMemo::DestinationTag(123456));
        assert_eq!(payment_memo(Some("  ")).unwrap(), XrpPaymentMemo::None);
        assert_eq!(payment_memo(None).unwrap(), XrpPaymentMemo::None);
        assert!(payment_memo(Some("4294967296")).is_err());
    }

    #[test]
    fn test_swap_memo_is_always_memo_data() {
        let data = SwapQuoteData {
            data: "fallback".to_string(),
            ..SwapQuoteData::new_transfer("rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF".to_string(), BigUint::from(10u64), Some("123".to_string()))
        };

        assert_eq!(swap_memo(&data), XrpPaymentMemo::Memo(b"123".to_vec()));
    }

    #[test]
    fn test_sign_swap_uses_payload_not_provider() {
        let private_key = hex::decode("a5576c0f63da10e584568c8d134569ff44017b0a249eb70657127ae04f38cc77").unwrap();
        let input = SignerInput {
            fee: TransactionFee::new_from_fee(10.into(), AssetId::from_chain(Chain::Xrp)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Swap {
                    from_asset: Asset::from_chain(Chain::Xrp),
                    to_asset: Asset::from_chain(Chain::Xrp),
                    swap_data: SwapData {
                        data: SwapQuoteData {
                            data: "swap:memo".to_string(),
                            ..SwapQuoteData::new_transfer("rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF".to_string(), BigUint::from(10u64), None)
                        },
                        ..SwapData::mock_with_provider(SwapProvider::Okx)
                    },
                },
                "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq",
                "",
                "999",
                TransactionLoadMetadata::Xrp {
                    sequence: 32_268_248,
                    block_number: 32_268_269 - LEDGER_SEQUENCE_OFFSET,
                    is_destination_address_exist: true,
                },
            )
        };

        let signed = XrpChainSigner.sign_swap(&input, &private_key).unwrap();
        assert_eq!(signed.len(), 1);
        assert_eq!(
            signed[0],
            "12000022000000002401ec5fd8201b01ec5fed61400000000000000a68400000000000000a732103d13e1152965a51a4a9fd9a8b4ea3dd82a4eba6b25fcad5f460a2342bb650333f74473045022100d18b758a360fc0a4d013b095014410b4fbf0e97a265c10d01d85e86ff35e009a02203c7e860711fd18ca486d211c07669638cbfa39b5f01ccc430e12fdecfa11399281144c55f5a78067206507580be7bb2686c8460adff983148132e4e20aecf29090ac428a9c43f230a829220df9ea7d09737761703a6d656d6fe1f1"
        );
    }

    #[test]
    fn test_sign_swap_token_signs_issued_payment() {
        let private_key = hex::decode(RLUSD_PRIVATE_KEY).unwrap();
        let input = rlusd_payment_input(
            TransactionInputType::Swap {
                from_asset: Asset::mock_xrp_rlusd(),
                to_asset: Asset::from_chain(Chain::Xrp),
                swap_data: SwapData {
                    data: SwapQuoteData::new_transfer(RLUSD_DESTINATION.to_string(), BigUint::from(RLUSD_VALUE), None),
                    ..SwapData::mock_with_provider(SwapProvider::Bridgers)
                },
            },
            "",
        );

        assert_eq!(XrpChainSigner.sign_swap(&input, &private_key).unwrap(), vec![RLUSD_SIGNED_PAYMENT.to_string()]);
    }

    #[test]
    fn test_account_action_signs_trust_set() {
        let private_key = hex::decode(RLUSD_PRIVATE_KEY).unwrap();
        let input = SignerInput {
            fee: TransactionFee::new_from_fee(500.into(), AssetId::from_chain(Chain::Xrp)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Account {
                    asset: Asset::mock_xrp_rlusd(),
                    account_type: AccountDataType::Activate,
                },
                RLUSD_SENDER,
                "",
                "0",
                TransactionLoadMetadata::Xrp {
                    sequence: 93_674_950,
                    block_number: 187_349_950 - LEDGER_SEQUENCE_OFFSET,
                    is_destination_address_exist: true,
                },
            )
        };

        let signed = XrpChainSigner.sign_account_action(&input, &private_key).unwrap();
        assert_eq!(
            signed,
            "12001422000000002405955dc6201b0b2abbbe63d758838370f34000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e6840000000000001f47321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d74473045022100833630562a6946f42f0471b6ce6193f51d761c4b98775104ed5df1fe3ce320680220382dafe6bf27b1f2be2dffdda3c139bcf2c7f8e04872af14f197b5228734800181140265c09d122fab2a261a80ee59f1f4cd8fba8cf8"
        );
    }

    #[test]
    fn test_trust_line_limit_is_not_scaled_by_token_decimals() {
        let input = SignerInput::mock_with_input_type(
            TransactionInputType::Account {
                asset: Asset::mock_xrp_rlusd(),
                account_type: AccountDataType::Activate,
            },
            "rDgEGKXWkHHr1HYq2ETnNAs9MdV4R8Gyt",
            "",
            "0",
            TransactionLoadMetadata::None,
        );
        let amount = trust_line_limit_amount(input.input_type.get_asset()).unwrap();

        assert_eq!(amount, XrpAmount::issued(TRUST_LINE_LIMIT, "RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De").unwrap());
    }

    const XRP_TO_RLUSD_SIGNED_BATCH: &str = "12004722000100002405955dc7201b066e31e06840000000000000647321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d7446304402205d98c172b917d7927a2cc6eb8a1f1f4d14d1bd55d6638fe0c0043cd9b1935db2022074b49f494d43f7dc29ceb6d735f55fbbdc441c26478d9b2f7e5c29f2a7ce418681140265c09d122fab2a261a80ee59f1f4cd8fba8cf8f01ee02212001422400000002405955dc863d758838370f34000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e684000000000000000730081140265c09d122fab2a261a80ee59f1f4cd8fba8cf8e1e02212000022400200002405955dc961d544a3d79434f0b4524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e68400000000000000069400000003b4e7ec06ad54497f6b40135f9524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e730081140265c09d122fab2a261a80ee59f1f4cd8fba8cf883140265c09d122fab2a261a80ee59f1f4cd8fba8cf8011230524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e00e1e02212000022400000002405955dca6140000000004c4b40684000000000000000730081140265c09d122fab2a261a80ee59f1f4cd8fba8cf8831453f7c208ea96c0b2a16c05e342fb992b521ba110e1f1";
    const XRP_TO_RLUSD_SIGNED_PAYMENT: &str = "12000022000200002405955dc7201b066e31e061d544a3d79434f0b4524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e68400000000000001469400000003b4e7ec06ad54497f6b40135f9524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e7321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d74463044022054c7c8a1e122a75cfd923b90d418d45d6cc999dda1c46bfbc2521b61bc1eb9ce02206d13c391ad84a1f798f0d1761a44d69467a6661b4fad9246d62bd1c71c9dd6cb81140265c09d122fab2a261a80ee59f1f4cd8fba8cf883140265c09d122fab2a261a80ee59f1f4cd8fba8cf8011230524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e00";
    const RLUSD_TO_XRP_SIGNED_BATCH: &str = "12004722000100002405955dc7201b066e31e06840000000000000507321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d74463044022000f4589c4ee4847a01a413881a5bc49a1b0b7bd2aa2561dde405ae886ab0be66022026fa39048ba5f7898526539f8a0f8f55a59ec406484a31ff1fc80752511f497581140265c09d122fab2a261a80ee59f1f4cd8fba8cf8f01ee02212000022400200002405955dc861400000002d530f8e68400000000000000069d5235978e783e000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e6a400000002cdf07b8730081140265c09d122fab2a261a80ee59f1f4cd8fba8cf883140265c09d122fab2a261a80ee59f1f4cd8fba8cf8011210000000000000000000000000000000000000000000e1e02212000022400000002405955dc961d491c37937e08000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e684000000000000000730081140265c09d122fab2a261a80ee59f1f4cd8fba8cf8831453f7c208ea96c0b2a16c05e342fb992b521ba110e1f1";

    #[test]
    fn test_sign_swap_batch() {
        let private_key = hex::decode(RLUSD_PRIVATE_KEY).unwrap();
        let (xrp, rlusd, usdc) = (Asset::from_chain(Chain::Xrp), Asset::mock_xrp_rlusd(), Asset::mock_xrp_usdc());
        let xrp_to_rlusd = XrpSwap { trust_line: true, ..XrpSwap::mock() };
        let rlusd_to_xrp = XrpSwap {
            send_max: BigUint::from(995_000_000_000_000_000u64),
            amount: BigUint::from(760_418_190u64),
            deliver_min: BigUint::from(752_814_008u64),
            paths: vec![vec![XrpPathStep {
                account: None,
                currency: Some("XRP".to_string()),
                issuer: None,
            }]],
            fee: Some(TransactionSwapReferralFee {
                asset_id: XRP_RLUSD_ASSET_ID.clone(),
                value: BigUint::from(5_000_000_000_000_000u64),
            }),
            trust_line: false,
        };
        let usdc_fee = XrpSwap {
            fee: Some(TransactionSwapReferralFee {
                asset_id: XRP_USDC_ASSET_ID.clone(),
                value: BigUint::from(5_000_000_000_000_000u64),
            }),
            ..rlusd_to_xrp.clone()
        };
        let mut foreign_destination = serde_json::to_value(&xrp_to_rlusd).unwrap();
        foreign_destination["fee"]["destination"] = serde_json::json!(RLUSD_DESTINATION);

        let json = |swap: &XrpSwap| serde_json::to_string(swap).unwrap();
        let refused = |message: &str| Err(SignerError::invalid_input(message));

        for (from_asset, to_asset, data, value, fee, expected) in [
            (&xrp, &rlusd, json(&xrp_to_rlusd), "1000000000", 100, Ok(vec![XRP_TO_RLUSD_SIGNED_BATCH.to_string()])),
            (&xrp, &rlusd, foreign_destination.to_string(), "1000000000", 100, Ok(vec![XRP_TO_RLUSD_SIGNED_BATCH.to_string()])),
            (&xrp, &rlusd, json(&xrp_to_rlusd), "999999999", 100, refused("XRP swap amount does not match the amount paid")),
            (&rlusd, &xrp, json(&rlusd_to_xrp), "1000000000000000000", 80, Ok(vec![RLUSD_TO_XRP_SIGNED_BATCH.to_string()])),
            (&usdc, &xrp, json(&usdc_fee), "1000000000000000000", 80, refused("XRP swap fee must be XRP or RLUSD, sold or bought")),
            (&xrp, &rlusd, json(&XrpSwap { fee: None, ..XrpSwap::mock() }), "995000000", 20, Ok(vec![XRP_TO_RLUSD_SIGNED_PAYMENT.to_string()])),
        ] {
            let input = SignerInput {
                fee: TransactionFee::new_from_fee(fee.into(), AssetId::from_chain(Chain::Xrp)),
                ..SignerInput::mock_with_input_type(
                    TransactionInputType::Swap {
                        from_asset: from_asset.clone(),
                        to_asset: to_asset.clone(),
                        swap_data: SwapData::mock_with_provider_data(SwapProvider::XrpLedger, &data, None),
                    },
                    RLUSD_SENDER,
                    "",
                    value,
                    TransactionLoadMetadata::Xrp {
                        sequence: 93_674_951,
                        block_number: 107_885_000,
                        is_destination_address_exist: false,
                    },
                )
            };
            assert_eq!(XrpChainSigner.sign_swap(&input, &private_key), expected);
        }
    }
}
