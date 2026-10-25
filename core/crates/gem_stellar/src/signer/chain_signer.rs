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
    use crate::models::swap::StellarSwapData;
    use crate::signer::serialization::encode_transaction;
    use crate::signer::signing::sign_transaction;
    use gem_encoding::decode_base64;
    use num_bigint::BigUint;
    use primitives::{
        Address, Asset, AssetId, AssetType, Chain, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadMetadata, TransactionSwapReferralFee,
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

    const EURC_TOKEN_ID: &str = "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC";

    fn token(token_id: &str) -> Asset {
        let symbol = token_id.split("::").last().unwrap().to_string();
        Asset::mock_with_params(Chain::Stellar, Some(token_id.into()), symbol.clone(), symbol, 7, AssetType::TOKEN)
    }

    fn asset_data(token_id: &str) -> Option<StellarAssetData> {
        StellarAssetData::from_asset_id(&AssetId::from_token(Chain::Stellar, token_id)).unwrap()
    }

    fn swap(send_amount: u64, change_trust: bool, fee_asset: AssetId) -> serde_json::Value {
        serde_json::to_value(StellarSwapData {
            send_amount,
            destination_min: 168_300_000,
            path: vec![AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID)],
            change_trust,
            fee: Some(TransactionSwapReferralFee {
                asset_id: fee_asset,
                value: BigUint::from(5_000_000u32),
            }),
            max_time: 1_792_902_087,
        })
        .unwrap()
    }

    fn swap_input(from_asset: Asset, to_asset: Asset, swap: serde_json::Value) -> SignerInput {
        SignerInput {
            fee: TransactionFee::new_from_fee(300.into(), AssetId::from_chain(Chain::Stellar)),
            ..SignerInput::mock_with_input_type(
                TransactionInputType::Swap {
                    from_asset,
                    to_asset,
                    swap_data: SwapData::mock_with_provider_data(SwapProvider::StellarDex, &swap.to_string(), None),
                },
                SENDER,
                "",
                "1000000000",
                TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: false,
                },
            )
        }
    }

    fn swap_transaction(send: Option<StellarAssetData>, send_amount: u64, receive: Option<StellarAssetData>, fee_asset: Option<StellarAssetData>) -> Vec<Operation> {
        vec![
            Operation::PathPaymentStrictSend {
                send_asset: send,
                send_amount,
                destination: StellarAddress::from_str(SENDER).unwrap(),
                destination_asset: receive,
                destination_min: 168_300_000,
                path: vec![asset_data(EURC_TOKEN_ID)],
            },
            Operation::Payment {
                destination: StellarAddress::from_str(STELLAR_REFERRAL_ADDRESS).unwrap(),
                asset: fee_asset,
                amount: 5_000_000,
            },
        ]
    }

    fn encoded(operations: Vec<Operation>) -> Vec<u8> {
        encode_transaction(&StellarTransaction {
            account: StellarAddress::from_str(SENDER).unwrap(),
            fee: 300,
            sequence: 2,
            memo: Memo::None,
            time_bounds: Some(1_792_902_087),
            operations,
        })
    }

    #[test]
    fn test_sign_swap_pays_the_sender_and_gem_whatever_the_payload_names() {
        let xlm = AssetId::from_chain(Chain::Stellar);
        let mut with_foreign_destination = swap(995_000_000, true, xlm);
        with_foreign_destination["fee"]["destination"] = DESTINATION.into();
        let input = swap_input(Asset::from_chain(Chain::Stellar), token(STELLAR_USDC_TOKEN_ID), with_foreign_destination);
        let usdc = asset_data(STELLAR_USDC_TOKEN_ID);

        assert_eq!(
            encode_transaction(&StellarTransaction::swap(&input).unwrap()),
            encoded([vec![Operation::ChangeTrust { asset: usdc.clone().unwrap() }], swap_transaction(None, 995_000_000, usdc, None)].concat())
        );
        assert!(StellarChainSigner.sign_swap(&input, &hex::decode(PRIVATE_KEY).unwrap()).is_ok());
    }

    #[test]
    fn test_sign_swap_pays_gem_in_the_asset_of_its_side() {
        let usdc = asset_data(STELLAR_USDC_TOKEN_ID);
        let selling_usdc = swap_input(token(STELLAR_USDC_TOKEN_ID), Asset::from_chain(Chain::Stellar), swap(995_000_000, false, STELLAR_USDC_ASSET_ID.clone()));
        let buying_xlm = swap_input(token(EURC_TOKEN_ID), Asset::from_chain(Chain::Stellar), swap(1_000_000_000, false, AssetId::from_chain(Chain::Stellar)));

        assert_eq!(encode_transaction(&StellarTransaction::swap(&selling_usdc).unwrap()), encoded(swap_transaction(usdc.clone(), 995_000_000, None, usdc)));
        assert_eq!(
            encode_transaction(&StellarTransaction::swap(&buying_xlm).unwrap()),
            encoded(swap_transaction(asset_data(EURC_TOKEN_ID), 1_000_000_000, None, None))
        );
    }

    #[test]
    fn test_sign_swap_refuses_a_fee_gem_cannot_take_or_another_amount() {
        let key = hex::decode(PRIVATE_KEY).unwrap();
        let xlm = Asset::from_chain(Chain::Stellar);

        assert!(
            StellarChainSigner
                .sign_swap(&swap_input(xlm.clone(), token(EURC_TOKEN_ID), swap(1_000_000_000, true, STELLAR_USDC_ASSET_ID.clone())), &key)
                .is_err(),
            "USDC is neither sold nor bought"
        );
        assert!(
            StellarChainSigner
                .sign_swap(&swap_input(token(EURC_TOKEN_ID), xlm.clone(), swap(995_000_000, false, AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID))), &key)
                .is_err(),
            "Gem's fee account does not trust EURC"
        );
        assert!(
            StellarChainSigner
                .sign_swap(&swap_input(xlm, token(STELLAR_USDC_TOKEN_ID), swap(995_000_001, false, AssetId::from_chain(Chain::Stellar))), &key)
                .is_err(),
            "the swap and the fee spend more than the amount paid"
        );
    }
}
