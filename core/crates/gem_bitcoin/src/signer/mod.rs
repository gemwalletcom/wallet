pub(crate) mod address;
mod bitcoin_cash;
mod chain_signer;
mod encoding;
mod personalization;
mod planner;
mod signature;
mod transaction;
mod types;
mod zcash;

#[cfg(feature = "rpc")]
use std::collections::HashMap;

#[cfg(feature = "rpc")]
use num_bigint::BigInt;
#[cfg(feature = "rpc")]
use primitives::{BitcoinChain, SignerError, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput};

pub use chain_signer::BitcoinChainSigner;
#[cfg(test)]
pub(crate) use planner::PlanInput;
pub use signature::sign_personal;
pub use types::{BitcoinSignDataResponse, BitcoinSignMessageData};

#[cfg(feature = "rpc")]
pub(crate) fn estimate_transaction_fee(chain: BitcoinChain, input: &TransactionLoadInput) -> Result<TransactionFee, SignerError> {
    let signer_input = SignerInput::new(input.clone(), input.default_fee());
    let request = match &input.input_type {
        TransactionInputType::Transfer(_) => planner::SpendRequest::transfer(chain, &signer_input)?,
        TransactionInputType::Swap(_, _, _) => planner::SpendRequest::swap(chain, &signer_input)?,
        _ => return SignerError::invalid_input_err("unsupported Bitcoin transaction type"),
    };
    let plan = planner::UtxoPlanner::plan(request)?;

    Ok(TransactionFee {
        fee: BigInt::from(plan.fee),
        gas_price_type: input.gas_price.clone(),
        gas_limit: BigInt::from(1u8),
        options: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use bitcoin::consensus::encode::deserialize;
    use primitives::{BitcoinChain, ChainSigner};

    use super::BitcoinChainSigner;
    use crate::testkit::signer_mock::{
        TEST_PRIVATE_KEY, mock_contract_swap_input, mock_funded_transfer_input, mock_p2wpkh_transfer_input, mock_transfer_input, mock_transfer_swap_input,
    };

    fn sign_transfer(chain: BitcoinChain) -> String {
        BitcoinChainSigner::new(chain).sign_transfer(&mock_transfer_input(chain), &TEST_PRIVATE_KEY).unwrap()
    }

    fn assert_op_return_payload(script: &bitcoin::ScriptBuf, payload: &[u8]) {
        let bytes = script.as_bytes();
        assert_eq!(bytes[0], 0x6a);
        assert_eq!(bytes[1] as usize, payload.len());
        assert_eq!(&bytes[2..], payload);
    }

    #[test]
    fn test_sign_transfer_bitcoin() {
        let raw = sign_transfer(BitcoinChain::Bitcoin);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(&raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x01);
    }

    #[test]
    fn test_sign_transfer_doge() {
        let input = mock_funded_transfer_input(BitcoinChain::Doge);
        let raw = BitcoinChainSigner::new(BitcoinChain::Doge).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x01);
    }

    #[test]
    fn test_sign_transfer_bitcoin_cash() {
        let raw = sign_transfer(BitcoinChain::BitcoinCash);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x41);
    }

    #[test]
    fn test_signed_tx_is_rbf_signaled() {
        let raw = BitcoinChainSigner::new(BitcoinChain::Bitcoin)
            .sign_transfer(&mock_transfer_input(BitcoinChain::Bitcoin), &TEST_PRIVATE_KEY)
            .unwrap();
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(&raw).unwrap()).unwrap();

        assert_eq!(
            raw,
            "02000000010100000000000000000000000000000000000000000000000000000000000000000000006b483045022100ef21c70ee59cd7ef09f5d12252ed3c1c4fa971b33740cf2c29b90ab15fc3e5ea02205761ce83d839043d38341344c499cd13cb2ed4e03ebd88c6cf38fc84415b1e290121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac5e9b0000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000"
        );
        for input in &transaction.input {
            assert_eq!(input.sequence.0, 0xffff_fffd);
        }
    }

    #[test]
    fn test_sign_transfer_zcash() {
        let raw = sign_transfer(BitcoinChain::Zcash);
        let zcash_bytes = hex::decode(raw).unwrap();
        assert_eq!(&zcash_bytes[..20], hex::decode("050000800a27a726f04dec4d0000000000000000").unwrap().as_slice());
    }

    #[test]
    fn test_sign_transfer_vectors() {
        // Test vectors are checked against BitGoJS.
        let cases = [
            (
                "bitcoin_p2pkh",
                BitcoinChain::Bitcoin,
                mock_transfer_input(BitcoinChain::Bitcoin),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006b483045022100ef21c70ee59cd7ef09f5d12252ed3c1c4fa971b33740cf2c29b90ab15fc3e5ea02205761ce83d839043d38341344c499cd13cb2ed4e03ebd88c6cf38fc84415b1e290121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac5e9b0000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "bitcoin_p2wpkh",
                BitcoinChain::Bitcoin,
                mock_p2wpkh_transfer_input(),
                "0200000000010101000000000000000000000000000000000000000000000000000000000000000000000000fdffffff02102700000000000016001479b000887626b294a914501a4cd226b58b235983b39b00000000000016001479b000887626b294a914501a4cd226b58b235983024730440220093e657368a7f4cbb66153627854a437a4119fa2b43c8a172bef796b5a59054602201b55591497a8bd0be56dd1241dd22c595e3ad65fe0b03dc763139b1ce2c0b86b0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f00000000",
            ),
            (
                "litecoin",
                BitcoinChain::Litecoin,
                mock_transfer_input(BitcoinChain::Litecoin),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006a4730440220336a874568fc8f0817c10d804a0b8f9f3d595d280d3b2fa9a4de465ee56d273202200e9ad81a08b28a095b62695fb156d3a7e7b5044aa22dbb63f510028fa80f1a430121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a914020202020202020202020202020202020202020288acd6970000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "doge",
                BitcoinChain::Doge,
                mock_funded_transfer_input(BitcoinChain::Doge),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006b483045022100dc2bcc8259c3f355ff471cc8a44633b495d4febb8e90b30ee847c37a791476ba022079cff47cb5afcf61dc18a958ce97f97748bdde598a8ed442632f1703d44710290121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a914020202020202020202020202020202020202020288ac2047f205000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "bitcoin_cash",
                BitcoinChain::BitcoinCash,
                mock_transfer_input(BitcoinChain::BitcoinCash),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006a47304402202454d9566a573c0d47f6c2ee32e29fe946da4d859cf49d046b737fbc497070a6022077f20c9841368e646948066a0583b4f6aabeef0c2baff4c9e8733c1b3d98f5444121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a914020202020202020202020202020202020202020288acd6970000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "zcash",
                BitcoinChain::Zcash,
                mock_transfer_input(BitcoinChain::Zcash),
                "050000800a27a726f04dec4d0000000000000000010100000000000000000000000000000000000000000000000000000000000000000000006a473044022004354fd389558909b1ccfb05dd2f3c324423efcad18eb50a42cb65ebdd17513d02207dac81ff91f4108b4017329e55971c2bd6a213cf925886b95513519bd806fd9a0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914030303030303030303030303030303030303030388ac30750000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac000000",
            ),
        ];

        for (name, chain, input, expected) in cases {
            let raw = BitcoinChainSigner::new(chain).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
            assert_eq!(raw, expected, "{name}");
        }
    }

    #[test]
    fn test_sign_swap_memo_op_return() {
        let memo = "=:s:0xEe7E9CcFb529f2c1Cc02C0Aea8aCed7Ec7e98B5e:0/1/0:g1:50";
        let input = mock_transfer_swap_input(BitcoinChain::Doge, memo);
        let raw = BitcoinChainSigner::new(BitcoinChain::Doge).sign_swap(&input, &TEST_PRIVATE_KEY).unwrap().remove(0);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let memo_output = transaction.output.iter().find(|output| output.script_pubkey.is_op_return()).unwrap();
        assert_eq!(memo_output.value.to_sat(), 0);
        assert_op_return_payload(&memo_output.script_pubkey, memo.as_bytes());
    }

    #[test]
    fn test_sign_bitcoin_thorchain_transfer_memo_op_return() {
        let memo = "=:ETH.ETH:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:0/1/0";
        let mut input = mock_transfer_input(BitcoinChain::Bitcoin);
        input.input.memo = Some(memo.to_string());

        let raw = BitcoinChainSigner::new(BitcoinChain::Bitcoin).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();

        assert_eq!(transaction.output.len(), 3);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(transaction.output[1].value.to_sat(), 0);
        assert_op_return_payload(&transaction.output[1].script_pubkey, memo.as_bytes());
    }

    #[test]
    fn test_sign_contract_swap_rejected() {
        let input = mock_contract_swap_input(BitcoinChain::Bitcoin, "deadbeef001122", false);
        let error = BitcoinChainSigner::new(BitcoinChain::Bitcoin).sign_swap(&input, &TEST_PRIVATE_KEY).unwrap_err().to_string();

        assert!(error.contains("unsupported Bitcoin contract swap"));
    }
}
