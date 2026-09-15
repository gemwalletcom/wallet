use bitcoin::Psbt;
use primitives::{BitcoinChain, SignerError, SignerInput, SwapProvider, hex::decode_hex};

use super::psbt;

pub fn is_order_id(memo: &str) -> bool {
    decode_hex(memo).is_ok_and(|bytes| bytes.len() == 32)
}

pub fn validate_swap_transaction(chain: BitcoinChain, input: &SignerInput) -> Result<(Psbt, u64), SignerError> {
    if input.input_type.get_swap_data()?.quote.provider_data.provider != SwapProvider::Relay {
        return SignerError::invalid_input_err("unsupported Bitcoin contract swap");
    }
    psbt::validate_swap_transaction(chain, input)
}

pub fn sign_swap(chain: BitcoinChain, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    let (psbt, fee) = validate_swap_transaction(chain, input)?;
    if fee != input.fee.fee()? {
        return SignerError::invalid_input_err("Bitcoin PSBT does not match confirmed fee");
    }
    psbt::sign_transaction(&psbt, &input.sender_address, private_key)
}

#[cfg(test)]
mod tests {
    use bitcoin::{
        Amount, Psbt, ScriptBuf, Transaction, WPubkeyHash, Witness, consensus::deserialize, hashes::Hash, opcodes::all::OP_RETURN, script::Builder, sighash::EcdsaSighashType,
    };
    #[cfg(feature = "rpc")]
    use num_bigint::BigInt;
    use primitives::{ChainSigner, GasPriceType, SwapProvider, TransactionInputType};

    #[cfg(feature = "rpc")]
    use crate::signer::estimate_transaction_fee;

    use super::*;
    use crate::{
        signer::BitcoinChainSigner,
        testkit::{psbt_mock::mock_segwit_psbt, signer_mock::TEST_PRIVATE_KEY},
    };

    fn sign(psbt: &Psbt, mut input: SignerInput) -> Result<Vec<String>, SignerError> {
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.data.data = hex::encode(psbt.serialize());
        }
        BitcoinChainSigner::new(BitcoinChain::Bitcoin).sign_swap(&input, &TEST_PRIVATE_KEY)
    }

    #[test]
    fn test_sign_swap_preserves_memo_outputs() {
        let (mut psbt, input) = mock_segwit_psbt();
        psbt.unsigned_tx.output[2].script_pubkey = Builder::new().push_opcode(OP_RETURN).push_slice(b"other memo").into_script();
        psbt.unsigned_tx.output.push(psbt.unsigned_tx.output[2].clone());
        psbt.outputs.push(Default::default());
        for output_count in [4, 3, 2] {
            psbt.unsigned_tx.output.truncate(output_count);
            psbt.outputs.truncate(output_count);
            let signed = sign(&psbt, input.clone()).unwrap();
            let transaction: Transaction = deserialize(&hex::decode(&signed[0]).unwrap()).unwrap();
            assert_eq!(transaction.output, psbt.unsigned_tx.output);
            assert!(!transaction.input[0].witness.is_empty());
        }
    }

    #[test]
    fn test_sign_segwit_swap() {
        let (psbt, input) = mock_segwit_psbt();
        assert_eq!(sign(&psbt, input).unwrap(), vec!["0200000000010100000000000000000000000000000000000000000000000000000000000000000000000000fdffffff0310270000000000001600140202020202020202020202020202020202020202589800000000000016001479b000887626b294a914501a4cd226b58b2359830000000000000000446a423078313231323132313231323132313231323132313231323132313231323132313231323132313231323132313231323132313231323132313231323132313231320247304402206883839bb4ef146ce04e516ac81fa7bc77892a5c50e22a71b0cf7070c5de616a02205815ffd5f065a5d6c803e27e55a2e3e3c5e63a78d3f56260c1dc95a8559736180121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f00000000".to_string()]);
    }

    #[test]
    fn test_reject_invalid_swap() {
        let cases: &[(&str, fn(&mut Psbt, &mut SignerInput), &str)] = &[
            (
                "provider",
                |_, input| {
                    if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
                        swap_data.quote.provider_data.provider = SwapProvider::Chainflip;
                    }
                },
                "unsupported Bitcoin contract swap",
            ),
            (
                "deposit amount",
                |psbt, _| {
                    psbt.unsigned_tx.output[0].value = Amount::from_sat(10_001);
                },
                "Bitcoin PSBT outputs do not match swap amount",
            ),
            (
                "redirected change",
                |psbt, _| {
                    psbt.unsigned_tx.output[1].script_pubkey = psbt.unsigned_tx.output[0].script_pubkey.clone();
                },
                "Bitcoin PSBT outputs do not match swap amount",
            ),
            (
                "destination",
                |psbt, _| {
                    psbt.unsigned_tx.output[0].script_pubkey = ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array([3; 20]));
                },
                "Bitcoin PSBT does not match swap destination",
            ),
            (
                "confirmed fee",
                |psbt, _| {
                    psbt.unsigned_tx.output[1].value = Amount::from_sat(38_999);
                },
                "Bitcoin PSBT does not match confirmed fee",
            ),
            (
                "UTXO value",
                |psbt, _| {
                    psbt.inputs[0].witness_utxo.as_mut().unwrap().value = Amount::from_sat(50_001);
                },
                "Bitcoin PSBT input does not match UTXO",
            ),
            (
                "unknown input",
                |psbt, _| {
                    psbt.unsigned_tx.input[0].previous_output.vout = 1;
                },
                "unknown Bitcoin PSBT input",
            ),
            (
                "missing UTXO",
                |psbt, _| {
                    psbt.inputs[0].witness_utxo = None;
                },
                "missing Bitcoin PSBT witness UTXO",
            ),
            (
                "UTXO script",
                |psbt, _| {
                    psbt.inputs[0].witness_utxo.as_mut().unwrap().script_pubkey = psbt.unsigned_tx.output[0].script_pubkey.clone();
                },
                "Bitcoin PSBT inputs do not match sender",
            ),
            (
                "duplicate input",
                |psbt, _| {
                    psbt.unsigned_tx.input.push(psbt.unsigned_tx.input[0].clone());
                    psbt.inputs.push(psbt.inputs[0].clone());
                },
                "Bitcoin PSBT inputs do not match sender",
            ),
            (
                "input amount",
                |_, input| {
                    input.input.value += 1u8;
                },
                "Bitcoin PSBT does not match swap quote",
            ),
            (
                "quote amount",
                |_, input| {
                    if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
                        swap_data.quote.from_value += 1u8;
                    }
                },
                "Bitcoin PSBT does not match swap quote",
            ),
            (
                "memo value",
                |psbt, _| {
                    psbt.unsigned_tx.output[2].value = Amount::from_sat(1);
                },
                "Bitcoin PSBT OP_RETURN output must have zero value",
            ),
        ];
        for (name, modify, error) in cases {
            let (mut psbt, mut input) = mock_segwit_psbt();
            modify(&mut psbt, &mut input);
            assert_eq!(sign(&psbt, input), SignerError::invalid_input_err(*error), "{name}");
        }

        for sighash in [EcdsaSighashType::None, EcdsaSighashType::Single, EcdsaSighashType::AllPlusAnyoneCanPay] {
            let (mut psbt, input) = mock_segwit_psbt();
            psbt.inputs[0].sighash_type = Some(sighash.into());
            assert_eq!(sign(&psbt, input), SignerError::invalid_input_err("unsupported Bitcoin PSBT sighash"), "{sighash}");
        }

        let (mut psbt, input) = mock_segwit_psbt();
        psbt.unsigned_tx.output[1].value = Amount::from_sat(50_000);
        assert!(sign(&psbt, input).is_err());

        let (psbt, mut input) = mock_segwit_psbt();
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.data.data = hex::encode(psbt.serialize());
        }
        assert_eq!(
            sign_swap(BitcoinChain::Bitcoin, &input, &[2; 32]),
            SignerError::invalid_input_err("Bitcoin private key does not match sender address")
        );
    }

    #[test]
    fn test_sign_swap_uses_confirmed_fee() {
        let (psbt, input) = mock_segwit_psbt();
        let expected = sign(&psbt, input.clone()).unwrap();
        for fee_rate in [0, 1] {
            let mut input = input.clone();
            input.fee.gas_price_type = GasPriceType::regular(fee_rate);
            assert_eq!(sign(&psbt, input).unwrap(), expected);
        }
    }

    #[test]
    fn test_ignore_provider_signatures() {
        let (mut psbt, input) = mock_segwit_psbt();
        let expected = sign(&psbt, input.clone()).unwrap();
        psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[vec![1, 2, 3]]));
        psbt.inputs[0].final_script_sig = Some(ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array([3; 20])));
        assert_eq!(sign(&psbt, input).unwrap(), expected);
    }

    #[test]
    fn test_max_swap_preserves_fixed_quote_amount() {
        let (psbt, mut input) = mock_segwit_psbt();
        let expected = sign(&psbt, input.clone()).unwrap();
        input.input.is_max_value = true;
        input.input.value = 50_000u64.into();
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.quote.use_max_amount = Some(true);
        }
        assert_eq!(sign(&psbt, input.clone()).unwrap(), expected);
        input.input.value = 9_000u64.into();
        assert_eq!(sign(&psbt, input), SignerError::invalid_input_err("Bitcoin PSBT does not match swap quote"));
    }

    #[cfg(feature = "rpc")]
    #[test]
    fn test_estimate_contract_swap_fee() {
        let (psbt, mut input) = mock_segwit_psbt();
        if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
            swap_data.data.data = hex::encode(psbt.serialize());
            swap_data.data.gas_limit = Some("999999".to_string());
        }
        let fee = estimate_transaction_fee(BitcoinChain::Bitcoin, &input.input).unwrap();
        assert_eq!(fee.fee, BigInt::from(1000));
    }
}
