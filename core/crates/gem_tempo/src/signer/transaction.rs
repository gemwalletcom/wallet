use std::str::FromStr;

use alloy_primitives::{Address, B256, Bytes, U256, keccak256};
use gem_evm::signer::rlp;
use primitives::SignerError;
use signer::Signer;

const TEMPO_TX_TYPE_ID: u8 = 0x76;

pub(super) struct TransactionCall {
    to: Address,
    input: Bytes,
}

impl TransactionCall {
    pub(super) fn new(to: &str, input: Vec<u8>) -> Result<Self, SignerError> {
        Ok(Self {
            to: Address::from_str(to).map_err(SignerError::from_display)?,
            input: Bytes::from(input),
        })
    }

    fn encode(&self, out: &mut Vec<u8>) {
        let mut payload = Vec::new();
        rlp::encode_bytes(&mut payload, self.to.as_slice());
        rlp::encode_uint(&mut payload, &U256::ZERO.to_be_bytes::<32>());
        rlp::encode_bytes(&mut payload, &self.input);
        rlp::encode_list(out, &payload);
    }
}

pub(super) struct TempoTransaction {
    pub(super) chain_id: u64,
    pub(super) max_priority_fee_per_gas: u128,
    pub(super) max_fee_per_gas: u128,
    pub(super) gas_limit: u64,
    pub(super) nonce: u64,
    pub(super) fee_token: Address,
    pub(super) calls: Vec<TransactionCall>,
}

impl TempoTransaction {
    fn fields(&self) -> Vec<u8> {
        let mut out = Vec::new();
        rlp::encode_uint(&mut out, &self.chain_id.to_be_bytes());
        rlp::encode_uint(&mut out, &self.max_priority_fee_per_gas.to_be_bytes());
        rlp::encode_uint(&mut out, &self.max_fee_per_gas.to_be_bytes());
        rlp::encode_uint(&mut out, &self.gas_limit.to_be_bytes());
        let mut calls = Vec::new();
        for call in &self.calls {
            call.encode(&mut calls);
        }
        rlp::encode_list(&mut out, &calls);
        rlp::encode_list(&mut out, &[]);
        rlp::encode_uint(&mut out, &U256::ZERO.to_be_bytes::<32>());
        rlp::encode_uint(&mut out, &self.nonce.to_be_bytes());
        rlp::encode_bytes(&mut out, &[]);
        rlp::encode_bytes(&mut out, &[]);
        rlp::encode_bytes(&mut out, self.fee_token.as_slice());
        rlp::encode_bytes(&mut out, &[]);
        rlp::encode_list(&mut out, &[]);
        out
    }

    fn envelope(payload: &[u8]) -> Vec<u8> {
        let mut out = vec![TEMPO_TX_TYPE_ID];
        rlp::encode_list(&mut out, payload);
        out
    }

    fn signature_hash(&self) -> B256 {
        keccak256(Self::envelope(&self.fields()))
    }

    pub(super) fn sign(&self, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let signature = Signer::sign_ethereum_digest(self.signature_hash().as_slice(), private_key)?;
        let mut payload = self.fields();
        rlp::encode_bytes(&mut payload, &signature);
        Ok(Self::envelope(&payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;
    use primitives::{
        Chain,
        asset_constants::TEMPO_BRIDGED_USDC_TOKEN_ID,
        testkit::signer_mock::{TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS},
    };

    #[test]
    fn test_sign_is_byte_stable_and_recovers_signer() {
        let transaction = TempoTransaction {
            chain_id: Chain::Tempo.network_id().parse().unwrap(),
            max_priority_fee_per_gas: 0,
            max_fee_per_gas: 20_000_000_000,
            gas_limit: 300_000,
            nonce: 0,
            fee_token: TEMPO_BRIDGED_USDC_TOKEN_ID.parse().unwrap(),
            calls: vec![TransactionCall::new(crate::testkit::TEMPO_TEST_ROUTER_ADDRESS, vec![0xab, 0xcd]).unwrap()],
        };
        let signed = transaction.sign(&TEST_PRIVATE_KEY).unwrap();

        assert_eq!(
            hex::encode(&signed),
            "76f888821079808504a817c800830493e0dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000b9537d11c60e8b5080c0b841c3d38922c55d24a60fbea9e1012c22724bf123e3eb8945897966d2f42fe1a94b460d7e811e19b3d033c08ea6fda4067355b5115ba79f9dd15ad0f381e5edd0771b"
        );

        let hash = transaction.signature_hash();
        assert_eq!(hex::encode(hash), "0ab41dc111006b724a6c72b55621146c6a264064c718db369fe1ac6657d5db57");

        let signature_bytes = &signed[signed.len() - 65..];
        assert_eq!(Signer::recover_ethereum_address(&hash.0, signature_bytes).unwrap(), TEST_PRIVATE_KEY_ETHEREUM_ADDRESS);
    }
}
