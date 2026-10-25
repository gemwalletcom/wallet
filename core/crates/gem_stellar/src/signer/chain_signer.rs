use crate::models::signing::StellarTransaction;
use crate::signer::signing::sign_transaction;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct StellarChainSigner;

impl ChainSigner for StellarChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::transfer(input)?, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::token_transfer(input)?, private_key)
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::account_action(input)?, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![sign_transaction(&StellarTransaction::swap(input)?, private_key)?])
    }
}

#[cfg(test)]
mod tests {
    // Tests taken from https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Stellar/TWAnySignerTests.cpp
    use super::*;
    use crate::address::StellarAddress;
    use crate::models::signing::{Memo, Operation, StellarAssetData, StellarTransaction};
    use crate::models::swap::{StellarSwapData, StellarSwapFee};
    use crate::signer::serialization::encode_transaction;
    use crate::signer::signing::sign_transaction;
    use gem_encoding::decode_base64;
    use primitives::{
        Address, Asset, AssetId, AssetType, Chain, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadMetadata,
        asset_constants::{STELLAR_USDC_ASSET_ID, STELLAR_USDC_TOKEN_ID},
        swap::{STELLAR_REFERRAL_ADDRESS, SwapData},
    };
    use signer::Ed25519KeyPair;

    const PRIVATE_KEY: &str = "59a313f46ef1c23a9e4f71cea10fc0c56a2a6bb8a4b9ea3d5348823e5a478722";
    const SENDER: &str = "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI";
    const DESTINATION: &str = "GDCYBNRRPIHLHG7X7TKPUPAZ7WVUXCN3VO7WCCK64RIFV5XM5V5K4A52";

    #[test]
    fn test_sign_stellar_transactions() {
        let key = hex::decode(PRIVATE_KEY).unwrap();

        // Native transfer with memo
        let mut input = SignerInput {
            fee: TransactionFee::new_from_fee(1000.into(), AssetId::from_chain(Chain::Stellar)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Stellar) },
                SENDER,
                DESTINATION,
                "10000000",
                TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: true,
                },
            )
        };
        input.input.memo = Some("Hello, world!".to_string());
        let signed = StellarChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAEAAAANSGVsbG8sIHdvcmxkIQAAAAAAAAEAAAAAAAAAAQAAAADFgLYxeg6zm/f81Po8Gf2rS4m7q79hCV7kUFr27O16rgAAAAAAAAAAAJiWgAAAAAAAAAABGd9cogAAAEBQQldEkYJ6rMvOHilkwFCYyroGGUvrNeWVqr/sn3iFFqgz91XxgUT0ou7bMSPRgPROfBYDfQCFfFxbcDPrrCwB"
        );

        // Transfer to non-existent destination (creates account)
        let input = SignerInput {
            fee: TransactionFee::new_from_fee(1000.into(), AssetId::from_chain(Chain::Stellar)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Stellar) },
                SENDER,
                DESTINATION,
                "10000000",
                TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: false,
                },
            )
        };
        let signed = StellarChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAxYC2MXoOs5v3/NT6PBn9q0uJu6u/YQle5FBa9uzteq4AAAAAAJiWgAAAAAAAAAABGd9cogAAAEA6vrVXe4OUNPKKlGtzJiNzGi1p1yAd6pxoTEcoixXZbWponp6L5XOVweg5tTM36pZVQjQIxEjOgktinR96Wf8O"
        );

        // Token transfer
        let mobi = Asset::mock_with_params(Chain::Stellar, Some("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH::MOBI".into()), "MOBI".into(), "MOBI".into(), 7, AssetType::TOKEN);
        let input = SignerInput {
            fee: TransactionFee::new_from_fee(1000.into(), AssetId::from_chain(Chain::Stellar)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Transfer { asset: mobi },
                "GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL",
                "GA3ISGYIE2ZTH3UAKEKBVHBPKUSL3LT4UQ6C5CUGP2IM5F467O267KI7",
                "12000000",
                TransactionLoadMetadata::Stellar {
                    sequence: 144098454883270661,
                    is_destination_address_exist: true,
                },
            )
        };
        let signed = StellarChainSigner
            .sign_token_transfer(&input, &hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap())
            .unwrap();
        assert_eq!(
            signed,
            "AAAAAMpFJQVVMv16RJUPlzQUTlgZOHVurhw3igGacP1305F1AAAD6AH/8MgAAAAFAAAAAAAAAAAAAAABAAAAAAAAAAEAAAAANokbCCazM+6AURQanC9VJL2ufKQ8LoqGfpDOl577te8AAAABTU9CSQAAAAA8cTArnmXa4wEQJxDHOw5SwBaDVjBfAP5lRMNZkRtlZAAAAAAAtxsAAAAAAAAAAAF305F1AAAAQEuWZZvKZuF6SMuSGIyfLqx5sn5O55+Kd489uP4g9jZH4UE7zZ4ME0+74I0BU8YDsYOmmxcfp/vdwTd+n3oGCQw="
        );
    }

    #[test]
    fn test_sign_change_trust_with_time_bounds() {
        let transaction = StellarTransaction {
            account: StellarAddress::from_str("GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL").unwrap(),
            fee: 10000,
            sequence: 144098454883270659,
            memo: Memo::None,
            time_bounds: Some(1613336576),
            operations: vec![Operation::ChangeTrust {
                asset: StellarAssetData::new("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH", "MOBI").unwrap(),
            }],
        };

        let signed = sign_transaction(&transaction, &hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap()).unwrap();
        assert_eq!(
            signed,
            "AAAAAMpFJQVVMv16RJUPlzQUTlgZOHVurhw3igGacP1305F1AAAnEAH/8MgAAAADAAAAAQAAAAAAAAAAAAAAAGApkAAAAAAAAAAAAQAAAAAAAAAGAAAAAU1PQkkAAAAAPHEwK55l2uMBECcQxzsOUsAWg1YwXwD+ZUTDWZEbZWR//////////wAAAAAAAAABd9ORdQAAAEAnfyXyaNQX5Bq3AEQVBIaYd+cLib+y2sNY7DF/NYVSE51dZ6swGGElz094ObsPefmVmeRrkGsSc/fF5pmth+wJ"
        );
    }

    #[test]
    fn test_stellar_signing_validation_and_hint() {
        let transfer_key = hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap();

        let signed = StellarChainSigner
            .sign_transfer(
                &SignerInput {
                    fee: TransactionFee::new_from_fee(1000.into(), AssetId::from_chain(Chain::Stellar)),
                    ..SignerInput::mock_with_input_type(
                        TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Stellar) },
                        SENDER,
                        DESTINATION,
                        "10000000",
                        TransactionLoadMetadata::Stellar {
                            sequence: 2,
                            is_destination_address_exist: true,
                        },
                    )
                },
                &transfer_key,
            )
            .unwrap();
        let envelope = decode_base64(&signed).unwrap();
        let signer_key = Ed25519KeyPair::from_private_key(&transfer_key).unwrap();
        let sender = StellarAddress::from_str(SENDER).unwrap();
        let hint_offset = envelope.len() - 72;

        assert_eq!(&envelope[hint_offset..hint_offset + 4], &signer_key.public_key_bytes[28..32]);
        assert_ne!(&envelope[hint_offset..hint_offset + 4], &sender.as_bytes()[28..32]);
    }

    const USDC_ISSUER: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
    const EURC_TOKEN_ID: &str = "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC";

    fn usdc() -> Asset {
        Asset::mock_with_params(Chain::Stellar, Some(STELLAR_USDC_TOKEN_ID.into()), "USDC".into(), "USDC".into(), 7, AssetType::TOKEN)
    }

    fn eurc() -> AssetId {
        AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID)
    }

    fn address(value: &str) -> StellarAddress {
        StellarAddress::from_str(value).unwrap()
    }

    fn fee(asset: AssetId) -> Option<StellarSwapFee> {
        Some(StellarSwapFee {
            destination: STELLAR_REFERRAL_ADDRESS.to_string(),
            asset,
            amount: 5_000_000,
        })
    }

    fn swap(change_trust: bool, fee: Option<StellarSwapFee>) -> StellarSwapData {
        StellarSwapData {
            send_amount: 995_000_000,
            destination_min: 168_300_000,
            path: vec![eurc()],
            change_trust,
            fee,
            max_time: 1_792_902_087,
        }
    }

    fn swap_input(from_asset: Asset, to_asset: Asset, swap: &StellarSwapData) -> SignerInput {
        SignerInput {
            fee: TransactionFee::new_from_fee(300.into(), AssetId::from_chain(Chain::Stellar)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Swap {
                    from_asset,
                    to_asset,
                    swap_data: SwapData::mock_with_provider_data(SwapProvider::StellarDex, &serde_json::to_string(swap).unwrap(), None),
                },
                SENDER,
                "",
                "0",
                TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: false,
                },
            )
        }
    }

    fn swap_transaction(operations: Vec<Operation>) -> StellarTransaction {
        StellarTransaction {
            account: address(SENDER),
            fee: 300,
            sequence: 2,
            memo: Memo::None,
            time_bounds: Some(1_792_902_087),
            operations,
        }
    }

    fn usdc_data() -> StellarAssetData {
        StellarAssetData::new(USDC_ISSUER, "USDC").unwrap()
    }

    fn eurc_data() -> StellarAssetData {
        StellarAssetData::new("GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2", "EURC").unwrap()
    }

    #[test]
    fn test_sign_swap_adds_the_trustline_pays_the_sender_and_pays_gem_in_xlm() {
        let input = swap_input(Asset::from_chain(Chain::Stellar), usdc(), &swap(true, fee(AssetId::from_chain(Chain::Stellar))));
        let expected = swap_transaction(vec![
            Operation::ChangeTrust { asset: usdc_data() },
            Operation::PathPaymentStrictSend {
                send_asset: None,
                send_amount: 995_000_000,
                destination: address(SENDER),
                destination_asset: Some(usdc_data()),
                destination_min: 168_300_000,
                path: vec![Some(eurc_data())],
            },
            Operation::Payment {
                destination: address(STELLAR_REFERRAL_ADDRESS),
                asset: None,
                amount: 5_000_000,
            },
        ]);

        assert_eq!(encode_transaction(&StellarTransaction::swap(&input).unwrap()), encode_transaction(&expected));
        assert_eq!(
            StellarChainSigner.sign_swap(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap(),
            vec![
                "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAABLAAAAAAAAAACAAAAAQAAAAAAAAAAAAAAAGrdg8cAAAAAAAAAAwAAAAAAAAAGAAAAAVVTREMAAAAAO5kROA7+mIugqJAOsc/kTzZvfb6Ua+0HckD39iTfFcV//////////wAAAAAAAAANAAAAAAAAAAA7Tn7AAAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAAAVVTREMAAAAAO5kROA7+mIugqJAOsc/kTzZvfb6Ua+0HckD39iTfFcUAAAAACggN4AAAAAEAAAABRVVSQwAAAADPT1om4gkLs63PAsep1z2/5mWcxpBGFHW4ZDf6SccRNgAAAAAAAAABAAAAAGQz0WF5MIOeVrLJ3g0uMHEIkFi2yu4USif8QRRfPJNCAAAAAAAAAAAATEtAAAAAAAAAAAEZ31yiAAAAQL38ulZ/BndSOPTim02/HtN/GhafyelaPd540vX9CzKH5dGtRVYYg5IFTnfpMoIe6+lwEguRNIDZMyrKJWgthgo="
            ]
        );
    }

    #[test]
    fn test_sign_swap_pays_gem_in_the_usdc_sold() {
        let input = swap_input(usdc(), Asset::from_chain(Chain::Stellar), &swap(false, fee(STELLAR_USDC_ASSET_ID.clone())));
        let expected = swap_transaction(vec![
            Operation::PathPaymentStrictSend {
                send_asset: Some(usdc_data()),
                send_amount: 995_000_000,
                destination: address(SENDER),
                destination_asset: None,
                destination_min: 168_300_000,
                path: vec![Some(eurc_data())],
            },
            Operation::Payment {
                destination: address(STELLAR_REFERRAL_ADDRESS),
                asset: Some(usdc_data()),
                amount: 5_000_000,
            },
        ]);

        assert_eq!(encode_transaction(&StellarTransaction::swap(&input).unwrap()), encode_transaction(&expected));
    }

    #[test]
    fn test_sign_swap_refuses_a_fee_the_fee_account_cannot_receive() {
        let eurc_asset = Asset::mock_with_params(Chain::Stellar, Some(EURC_TOKEN_ID.into()), "EURC".into(), "EURC".into(), 7, AssetType::TOKEN);
        let key = hex::decode(PRIVATE_KEY).unwrap();

        assert!(StellarChainSigner.sign_swap(&swap_input(eurc_asset, Asset::from_chain(Chain::Stellar), &swap(false, fee(eurc()))), &key).is_err());
        assert!(StellarChainSigner.sign_swap(&swap_input(usdc(), Asset::from_chain(Chain::Stellar), &swap(true, None)), &key).is_err(), "XLM needs no trustline");
        assert!(
            StellarChainSigner
                .sign_swap(&swap_input(Asset::from_chain(Chain::Stellar), usdc(), &StellarSwapData { path: vec![eurc(); 6], ..swap(false, None) }), &key)
                .is_err(),
            "a path holds at most five assets"
        );
    }
}
