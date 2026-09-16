use super::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER, TEST_OSMOSIS_SENDER};
use crate::{
    ApplicationMetadata, Asset, AssetId, AssetType, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata,
    TransferDataExtra, TransferDataOutputAction, TransferDataOutputType, UTXO, asset_constants::NEAR_USDT_ASSET_ID,
};
use num_bigint::BigInt;
use num_bigint::BigUint;
use std::collections::HashMap;

impl TransactionLoadInput {
    pub fn mock() -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Sui),
            },
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            value: BigUint::from(1000000000u64),
            gas_price: GasPriceType::regular(BigInt::from(1000u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }

    pub fn mock_aptos_token_transfer(token_id: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer {
                asset: Asset::mock_with_params(Chain::Aptos, Some(token_id.to_string()), "USD Coin".to_string(), "USDC".to_string(), 6, AssetType::TOKEN),
            },
            sender_address: "0x1".to_string(),
            destination_address: "0x2".to_string(),
            value: BigUint::from(1u64),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Aptos { sequence: 0, data: None },
        }
    }

    pub fn mock_with_input_type(input_type: TransactionInputType) -> Self {
        TransactionLoadInput {
            input_type,
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            value: BigUint::from(1000000000u64),
            gas_price: GasPriceType::regular(BigInt::from(1000u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }

    pub fn mock_evm(input_type: TransactionInputType, value: &str) -> Self {
        Self::mock_evm_with_metadata(input_type, value, TransactionLoadMetadata::mock_evm(0, 1))
    }

    pub fn mock_evm_with_metadata(input_type: TransactionInputType, value: &str, metadata: TransactionLoadMetadata) -> Self {
        TransactionLoadInput {
            input_type,
            sender_address: TEST_EVM_SENDER.to_string(),
            destination_address: TEST_EVM_RECIPIENT.to_string(),
            value: value.parse().unwrap(),
            gas_price: GasPriceType::eip1559(20_000_000_000u64, 1_000_000_000u64),
            memo: None,
            is_max_value: false,
            metadata,
        }
    }
}

impl SignerInput {
    pub fn mock_evm(input_type: TransactionInputType, value: &str, gas_limit: u64) -> Self {
        SignerInput::new(TransactionLoadInput::mock_evm(input_type, value), TransactionFee::mock_eip1559(gas_limit))
    }

    pub fn mock_evm_with_metadata(input_type: TransactionInputType, value: &str, gas_limit: u64, metadata: TransactionLoadMetadata) -> Self {
        SignerInput::new(
            TransactionLoadInput::mock_evm_with_metadata(input_type, value, metadata),
            TransactionFee::mock_eip1559(gas_limit),
        )
    }

    pub fn mock_with_input_type(input_type: TransactionInputType, sender: &str, destination: &str, value: &str, metadata: TransactionLoadMetadata) -> Self {
        SignerInput::new(
            TransactionLoadInput {
                input_type,
                sender_address: sender.to_string(),
                destination_address: destination.to_string(),
                value: value.parse().unwrap(),
                gas_price: GasPriceType::regular(0),
                memo: None,
                is_max_value: false,
                metadata,
            },
            TransactionFee::mock(),
        )
    }

    pub fn mock_osmosis(input_type: TransactionInputType, destination: &str) -> Self {
        let fee_amount = BigInt::from(10_000u64);
        SignerInput::new(
            TransactionLoadInput {
                input_type,
                sender_address: TEST_OSMOSIS_SENDER.to_string(),
                destination_address: destination.to_string(),
                value: BigUint::from(10u64),
                gas_price: GasPriceType::regular(fee_amount.clone()),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::mock_osmosis(),
            },
            TransactionFee::new_gas_price_type(
                GasPriceType::regular(fee_amount.clone()),
                fee_amount,
                BigInt::from(200_000u64),
                HashMap::new(),
                AssetId::from_chain(Chain::Osmosis),
            ),
        )
    }

    pub fn mock_ton(input_type: TransactionInputType, metadata: TransactionLoadMetadata) -> Self {
        SignerInput::new(
            TransactionLoadInput {
                input_type,
                sender_address: "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
                destination_address: "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
                value: BigUint::from(10000u64),
                gas_price: GasPriceType::regular(0),
                memo: None,
                is_max_value: false,
                metadata,
            },
            TransactionFee::new_from_fee(BigInt::ZERO, AssetId::from_chain(Chain::Ton)),
        )
    }

    pub fn mock_sign_data(chain: Chain, sender: &str, data: &str, output_type: TransferDataOutputType) -> Self {
        SignerInput::new(
            TransactionLoadInput {
                sender_address: sender.to_string(),
                ..TransactionLoadInput::mock_sign_data(chain, data, output_type)
            },
            TransactionFee::mock(),
        )
    }

    pub fn mock_solana(block_hash: &str) -> Self {
        SignerInput::new(
            TransactionLoadInput::mock_solana(block_hash),
            TransactionFee::new_from_fee(BigInt::ZERO, AssetId::from_chain(Chain::Solana)),
        )
    }

    pub fn mock_near_token_transfer(memo: Option<&str>, fee: TransactionFee) -> Self {
        SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer {
                    asset: Asset::new(NEAR_USDT_ASSET_ID.clone(), "Tether".to_string(), "USDT".to_string(), 6, AssetType::TOKEN),
                },
                memo: memo.map(String::from),
                ..TransactionLoadInput::mock_near("test.near", "receiver.near", "1000000", 1, "244ZQ9cgj3CQ6bWBdytfrJMuMQ1jdXLFGnr4HhvtCTnM")
            },
            fee,
        )
    }
}

impl TransactionLoadInput {
    pub fn mock_near(sender: &str, destination: &str, value: &str, sequence: u64, block_hash: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Near),
            },
            sender_address: sender.into(),
            destination_address: destination.into(),
            value: value.parse().unwrap(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Near {
                sequence,
                block_hash: block_hash.into(),
            },
        }
    }

    pub fn mock_solana(block_hash: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer { asset: Asset::mock_sol() },
            sender_address: String::new(),
            destination_address: String::new(),
            value: BigUint::from(0u64),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::mock_solana(block_hash),
        }
    }

    pub fn mock_polkadot() -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Polkadot),
            },
            sender_address: "15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo".to_string(),
            destination_address: "15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo".to_string(),
            value: BigUint::from(10000u64),
            gas_price: GasPriceType::regular(10),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Polkadot {
                sequence: 0,
                genesis_hash: "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3".to_string(),
                block_hash: "0x6e3ffeaa3be9d19bd110e5b6e7cbbc92cceed0d2ec557276c296bf7970ace2e5".to_string(),
                block_number: 24_666_537,
                spec_version: 1_003_004,
                transaction_version: 26,
                period: 64,
            },
        }
    }

    pub fn mock_cardano(sender: &str, value: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Cardano),
            },
            sender_address: sender.to_string(),
            destination_address: "addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5".to_string(),
            value: value.parse().unwrap(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Cardano {
                block_number: 189_992_800,
                utxos: vec![
                    UTXO {
                        transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
                        vout: 1,
                        value: BigUint::from(1500000u64),
                        address: sender.to_string(),
                    },
                    UTXO {
                        transaction_id: "554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af0".to_string(),
                        vout: 0,
                        value: BigUint::from(6500000u64),
                        address: sender.to_string(),
                    },
                ],
            },
        }
    }

    pub fn mock_sign_data(chain: Chain, data: &str, output_type: TransferDataOutputType) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Generic {
                asset: Asset::from_chain(chain),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra {
                    data: Some(data.as_bytes().to_vec()),
                    output_type,
                    output_action: TransferDataOutputAction::Send,
                    ..Default::default()
                },
            },
            sender_address: "test".into(),
            destination_address: "test".into(),
            value: BigUint::from(0u64),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }
}
