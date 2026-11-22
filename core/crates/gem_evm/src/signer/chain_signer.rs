use primitives::hex;

use num_bigint::BigInt;
use primitives::{ChainSigner, SignerError, SignerInput, StakeType, TransactionFee, TransactionLoadInput, decode_hex, swap::SwapQuoteDataType};
use signer::Signer;

use super::Eip1559Transaction;
use crate::eip712::hash_typed_data;
use crate::transaction_params::{TransactionCalls, TransactionParams, transaction_call, transaction_calls};

pub struct EvmChainSigner;

impl ChainSigner for EvmChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_deposit(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_nft_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_token_approval(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        match input.input_type.get_swap_data()?.data.data_type {
            SwapQuoteDataType::Transfer => SignerError::invalid_input_err("unsupported EVM transfer swap"),
            SwapQuoteDataType::Contract => sign_calls(input, &input.fee, &transaction_calls(input)?, private_key),
        }
    }

    fn sign_earn(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let calls = transaction_calls(input)?;
        let provider_gas_limit = input.input_type.get_earn_data()?.gas_limit.as_ref().and_then(|gas_limit| gas_limit.parse::<u64>().ok());
        match (&calls, provider_gas_limit) {
            (TransactionCalls::Call(_), Some(gas_limit)) => sign_calls(
                input,
                &TransactionFee {
                    gas_limit: BigInt::from(gas_limit),
                    ..input.fee.clone()
                },
                &calls,
                private_key,
            ),
            _ => sign_calls(input, &input.fee, &calls, private_key),
        }
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let stake_type = input.input_type.get_stake_type()?;
        let contract_call = input.metadata.get_contract_call()?;
        let value = match stake_type {
            StakeType::Stake(_) => input.value_as_bigint(),
            _ => BigInt::ZERO,
        };
        let calls = TransactionCalls::Call(TransactionParams::new(contract_call.contract_address.clone(), decode_hex(&contract_call.call_data)?, value));
        sign_calls(input, &input.fee, &calls, private_key)
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(input, private_key)
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let json_str = std::str::from_utf8(message).map_err(|_| SignerError::invalid_input("message must be valid UTF-8"))?;
        let digest = hash_typed_data(json_str)?;
        let signature = Signer::sign_ethereum_digest(&digest, private_key)?;
        Ok(format!("0x{}", hex::encode(signature)))
    }
}

fn sign_transaction(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_and_encode(&Eip1559Transaction::new(input, &input.fee, &transaction_call(input)?)?, private_key)
}

fn sign_calls(input: &TransactionLoadInput, fee: &TransactionFee, calls: &TransactionCalls, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    Eip1559Transaction::from_calls(input, fee, calls)?.iter().map(|transaction| sign_and_encode(transaction, private_key)).collect()
}

fn sign_and_encode(transaction: &Eip1559Transaction, private_key: &[u8]) -> Result<String, SignerError> {
    Ok(hex::encode(transaction.sign(private_key)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_hash::keccak::keccak256;
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, Chain, ChainSigner, DelegationValidator, NFTType, SignerInput, TransactionInputType, TransactionLoadMetadata, TransferDataExtra, contract_call_data::ContractCallData, nft::NFTAsset, swap::*};

    #[test]
    fn test_sign_message_matches_eip712_reference_vector() {
        let signer = EvmChainSigner;
        let message = include_bytes!("../../testdata/eip712_reference_vector.json");
        let private_key = keccak256(b"cow");

        assert_eq!(
            signer.sign_message(message, &private_key).unwrap(),
            "0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c"
        );
    }

    #[test]
    fn test_sign_transfer() {
        let signer = EvmChainSigner;
        let input = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }, "1000000000000000000", 21000);
        assert_eq!(
            signer.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8730180843b9aca008504a817c800825208942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a764000080c001a0ea6700354e2542e163e08c111d7b1d7e2a9d371a06977c9a79c42783c3237af9a001809a71f1fa2309f204b4ebed1a9e68f0e60ab736b98284727f2d8427ab705f"
        );
    }

    #[test]
    fn test_sign_token_transfer() {
        let signer = EvmChainSigner;
        let input = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "1000000", 65000);
        assert_eq!(
            signer.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8b00180843b9aca008504a817c80082fde894a0b86a33e6441066d64bb38954e41f6b4b925c5980b844a9059cbb0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf00000000000000000000000000000000000000000000000000000000000f4240c001a09ca8ae6c1d3e9a70465ae36e44c4ca9982a0b94c3cb8ec7c56e6a183f2d04f16a02275a147339b8a41e36670cbec2df08df035ea3b403eedd8325b150b53a3d7f4"
        );
    }

    #[test]
    fn test_sign_nft_transfer() {
        let signer = EvmChainSigner;

        let input = SignerInput::mock_evm(
            TransactionInputType::TransferNft {
                asset: Asset::from_chain(Chain::Ethereum),
                nft_asset: NFTAsset::mock(),
            },
            "0",
            100000,
        );
        assert_eq!(
            signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8d10180843b9aca008504a817c800830186a094dac17f958d2ee523a2206206994597c13d831ec780b86442842e0e0000000000000000000000007e5f4552091a69125d5dfcb7b8c2659029395bdf0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf0000000000000000000000000000000000000000000000000000000000000001c080a08371f982a5384532d5ac3336a174239f571cd75663ddc1d6f3892a59c940c983a035902737dfc2f2af6df4244e741f652f4d764230aecf9d5a910d0d027dc4238d"
        );

        let input = SignerInput::mock_evm(
            TransactionInputType::TransferNft {
                asset: Asset::from_chain(Chain::Ethereum),
                nft_asset: NFTAsset::mock_with_type(NFTType::ERC1155),
            },
            "0",
            100000,
        );
        assert_eq!(
            signer.sign_nft_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f901310180843b9aca008504a817c800830186a094dac17f958d2ee523a2206206994597c13d831ec780b8c4f242432a0000000000000000000000007e5f4552091a69125d5dfcb7b8c2659029395bdf0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000c080a032eb8933adf3d5fb105c292e413bae35339b9ee6b30ac4b8b679e9f14a0009dba00964abf4af5aa48120c18942ef2b64189bda437a97a7616e14b39624c0870329"
        );
    }

    #[test]
    fn test_sign_token_approval() {
        let signer = EvmChainSigner;
        let input = SignerInput::mock_evm(
            TransactionInputType::TokenApprove {
                asset: Asset::from_chain(Chain::Ethereum),
                approval_data: ApprovalData::mock(),
            },
            "0",
            65000,
        );
        assert_eq!(
            signer.sign_token_approval(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
    }

    #[test]
    fn test_sign_swap_without_approval() {
        let signer = EvmChainSigner;
        let swap_data = SwapData {
            quote: SwapQuote::mock(),
            data: SwapQuoteData {
                value: BigUint::parse_bytes(b"1000000000000000000", 10).unwrap(),
                data: "abcd".to_string(),
                gas_limit: None,
                ..SwapQuoteData::mock()
            },
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::from_chain(Chain::Ethereum),
                to_asset: Asset::from_chain(Chain::Ethereum),
                swap_data,
            },
            "1000000000000000000",
            200000,
        );
        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f8760180843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a764000082abcdc001a0a576e21827f710051c5d9402777cd913469bfa46a6a87281c60b2c48eb620db1a052ec097cab72419fd228b09276db146c1ad8c4e6fa35a34166ba718bdeadc892"
        );

        let token_swap = SignerInput::mock_evm(
            TransactionInputType::mock_swap(
                Asset::mock_erc20(),
                SwapQuoteData {
                    value: BigUint::from(5000u32),
                    ..SwapQuoteData::mock()
                },
            ),
            "1000000",
            200000,
        );
        assert_eq!(
            signer.sign_swap(&token_swap, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "02f86e0180843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf82138880c001a07160cbddf76da145b3b8cd5e1ea33888cf7e8eb3999f5dfafbb40baf5c6bd3bca0681258ab4d42cce971d82e0b4efe315d49bdc30958fd516d52d1001470b8f3b6"
            ],
            "a token contract swap signs the quote's value"
        );
    }

    #[test]
    fn test_sign_swap_with_approval() {
        let signer = EvmChainSigner;
        let swap_data = SwapData {
            quote: SwapQuote::mock(),
            data: SwapQuoteData {
                data: "abcd".to_string(),
                approval: Some(ApprovalData::mock()),
                gas_limit: Some("200000".to_string()),
                ..SwapQuoteData::mock()
            },
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::from_chain(Chain::Ethereum),
                to_asset: Asset::from_chain(Chain::Ethereum),
                swap_data,
            },
            "0",
            65000,
        );
        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
        assert_eq!(
            result[1],
            "02f86e0101843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a02ecc5acc573cb465ae28b24756c384a5dfd5eb4ced9479d73d93e50dea7f30fba01cbb3e7bc343e8294ed26a005b001408498ed364402b5a37830be9b8d850bda4"
        );
    }

    #[test]
    fn test_sign_stake() {
        let signer = EvmChainSigner;
        let metadata = TransactionLoadMetadata::Evm {
            nonce: 5,
            chain_id: 1,
            contract_call: Some(ContractCallData::mock_with_call_data("3a29dbae0000000000000000000000000000000000000000000000000000000000000017")),
        };
        let input = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Ethereum),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
            "1000000000000000000",
            200000,
            metadata,
        );
        let result = signer.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f8980105843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf880de0b6b3a7640000a43a29dbae0000000000000000000000000000000000000000000000000000000000000017c001a067b915da126e46cfb9c78db7aa5c277743b2de43450dd744f2e5bc16146b0954a01fa8f62608f997199eab37464df9cc1821a2c66a6c8f36d6ba438206a7a3556a"
        );
    }

    #[test]
    fn test_sign_data() {
        let signer = EvmChainSigner;
        let extra = TransferDataExtra {
            gas_limit: Some(300_000.into()),
            ..TransferDataExtra::mock_encoded_transaction(vec![0xab, 0xcd])
        };
        let input = SignerInput::mock_evm(TransactionInputType::mock_generic(Asset::from_chain(Chain::Ethereum), extra), "0", 100000);
        assert_eq!(
            signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f86e0180843b9aca008504a817c800830186a0942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a085087f2d3c999ea4e253274ed68a9e58cc7eb9f2ee7e037897ce371ddc74f0bea06613bd4201e26f5738ca375fcbd19ee8e8a1fd633e7d2f2061a14f3d0ee1173a",
            "the fee's gas limit is signed, the one shown and balance-checked"
        );
    }

    #[test]
    fn test_sign_earn() {
        let signer = EvmChainSigner;
        let input = SignerInput::mock_evm(
            TransactionInputType::Earn {
                asset: Asset::from_chain(Chain::Ethereum),
                earn_type: primitives::EarnType::Deposit(DelegationValidator::mock()),
                data: ContractCallData::mock(),
            },
            "0",
            200000,
        );
        let result = signer.sign_earn(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            "02f86e0180843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc001a00a342182b976d28a460ede1a104708d57d1174a5f4ba383eef91c2a774dfab62a0657d5a65976b2241a13554260c538372066938c51eb8116bbad6211c6b29bfff"
        );

        let input = SignerInput::mock_evm(
            TransactionInputType::mock_earn(ContractCallData {
                gas_limit: Some("300000".to_string()),
                ..ContractCallData::mock()
            }),
            "0",
            200000,
        );
        assert_eq!(
            signer.sign_earn(&input, &TEST_PRIVATE_KEY).unwrap(),
            vec![
                "02f86e0180843b9aca008504a817c800830493e0942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a09babac95a7855e263320ab8b237c3d3618fb147e7d3c539bb91d9c6c33850614a07fc3c7cf7b166133af818eddbe45aae6962040e4d166247ae056315176643f18"
            ],
            "the provider's gas limit is signed"
        );
    }

    #[test]
    fn test_sign_earn_with_approval() {
        let signer = EvmChainSigner;
        let earn_data = ContractCallData {
            approval: Some(ApprovalData::mock()),
            gas_limit: Some("200000".to_string()),
            ..ContractCallData::mock()
        };
        let input = SignerInput::mock_evm(
            TransactionInputType::Earn {
                asset: Asset::from_chain(Chain::Ethereum),
                earn_type: primitives::EarnType::Deposit(DelegationValidator::mock()),
                data: earn_data,
            },
            "0",
            65000,
        );
        let result = signer.sign_earn(&input, &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            "02f8b00180843b9aca008504a817c80082fde894dac17f958d2ee523a2206206994597c13d831ec780b844095ea7b30000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc001a02fde3a01cfa4c2349782fa026932003f5eae7077763db75b31be593e7c15f4a3a048071a05fccf3905cb59be79fc9a0442d801907506193c158858cd6b7ef31fa3"
        );
        assert_eq!(
            result[1],
            "02f86e0101843b9aca008504a817c80083030d40942b5ad5c4795c026514f8317c7a215e218dccd6cf8082abcdc080a02ecc5acc573cb465ae28b24756c384a5dfd5eb4ced9479d73d93e50dea7f30fba01cbb3e7bc343e8294ed26a005b001408498ed364402b5a37830be9b8d850bda4"
        );
    }

    #[test]
    fn test_invalid_metadata() {
        let signer = EvmChainSigner;
        let input = SignerInput::mock_evm_with_metadata(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }, "1000000000000000000", 21000, TransactionLoadMetadata::None);
        assert!(signer.sign_transfer(&input, &TEST_PRIVATE_KEY).is_err());
    }
}
