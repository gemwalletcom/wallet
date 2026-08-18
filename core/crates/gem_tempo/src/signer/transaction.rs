use alloy_primitives::{Address, B256, Bytes, U256, keccak256};
use alloy_rlp::{BufMut, EMPTY_STRING_CODE, Encodable, Header};
use primitives::SignerError;
use signer::Signer;

/// <https://docs.tempo.xyz/protocol/transactions/spec-tempo-transaction>
pub(crate) const TEMPO_TX_TYPE_ID: u8 = 0x76;

pub struct TransactionCall {
    to: Address,
    input: Bytes,
}

impl TransactionCall {
    pub fn new(to: Address, input: Bytes) -> Self {
        Self { to, input }
    }

    fn header(&self) -> Header {
        Header {
            list: true,
            payload_length: self.to.length() + U256::ZERO.length() + self.input.length(),
        }
    }
}

impl Encodable for TransactionCall {
    fn encode(&self, out: &mut dyn BufMut) {
        self.header().encode(out);
        self.to.encode(out);
        U256::ZERO.encode(out);
        self.input.encode(out);
    }

    fn length(&self) -> usize {
        self.header().length_with_payload()
    }
}

pub struct TempoTransaction {
    pub chain_id: u64,
    pub max_priority_fee_per_gas: u128,
    pub max_fee_per_gas: u128,
    pub gas_limit: u64,
    pub nonce: u64,
    pub fee_token: Address,
    pub calls: Vec<TransactionCall>,
}

impl TempoTransaction {
    fn encode_fields(&self, out: &mut dyn BufMut) {
        self.chain_id.encode(out);
        self.max_priority_fee_per_gas.encode(out);
        self.max_fee_per_gas.encode(out);
        self.gas_limit.encode(out);
        self.calls.encode(out);
        Header { list: true, payload_length: 0 }.encode(out); // access_list = []
        U256::ZERO.encode(out); // nonce_key = 0 (protocol nonce)
        self.nonce.encode(out);
        out.put_u8(EMPTY_STRING_CODE); // valid_before = None
        out.put_u8(EMPTY_STRING_CODE); // valid_after = None
        self.fee_token.encode(out);
        out.put_u8(EMPTY_STRING_CODE); // fee_payer_signature = None
        Header { list: true, payload_length: 0 }.encode(out); // aa_authorization_list = []
        // key_authorization: omitted entirely (no bytes) when None, per spec
    }

    fn fields_payload_length(&self) -> usize {
        self.chain_id.length()
            + self.max_priority_fee_per_gas.length()
            + self.max_fee_per_gas.length()
            + self.gas_limit.length()
            + self.calls.length()
            + 1 // access_list = []
            + U256::ZERO.length()
            + self.nonce.length()
            + 1 // valid_before = None
            + 1 // valid_after = None
            + self.fee_token.length()
            + 1 // fee_payer_signature = None
            + 1 // aa_authorization_list = []
    }

    fn signature_hash(&self) -> B256 {
        let payload_length = self.fields_payload_length();
        let mut buf = Vec::with_capacity(1 + Header { list: true, payload_length }.length_with_payload());
        buf.put_u8(TEMPO_TX_TYPE_ID);
        Header { list: true, payload_length }.encode(&mut buf);
        self.encode_fields(&mut buf);
        keccak256(&buf)
    }

    pub(crate) fn sign(&self, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let signature = Bytes::from(Signer::sign_ethereum_digest(self.signature_hash().as_slice(), private_key)?);

        let payload_length = self.fields_payload_length() + signature.length();
        let mut buf = Vec::with_capacity(1 + Header { list: true, payload_length }.length_with_payload());
        buf.put_u8(TEMPO_TX_TYPE_ID);
        Header { list: true, payload_length }.encode(&mut buf);
        self.encode_fields(&mut buf);
        signature.encode(&mut buf);
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Signature;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;

    #[test]
    fn test_sign_is_byte_stable_and_recovers_signer() {
        let transaction = TempoTransaction::mock();
        let signed = transaction.sign(&TEST_PRIVATE_KEY).unwrap();

        assert_eq!(
            hex::encode(&signed),
            "76f888821079808504a817c800830493e0dad994a2dc7d0266f0cc50b3eeaf36c9bfcecff1beea918082abcdc0808080809420c000000000000000000000b9537d11c60e8b5080c0b841c3d38922c55d24a60fbea9e1012c22724bf123e3eb8945897966d2f42fe1a94b460d7e811e19b3d033c08ea6fda4067355b5115ba79f9dd15ad0f381e5edd0771b"
        );

        let hash = transaction.signature_hash();
        assert_eq!(hex::encode(hash), "0ab41dc111006b724a6c72b55621146c6a264064c718db369fe1ac6657d5db57");

        let signature_bytes = &signed[signed.len() - 65..];
        let signature = Signature::try_from(signature_bytes).unwrap();
        let signer = signature.recover_address_from_prehash(&hash).unwrap();
        assert_eq!(signer, "0x1a642f0e3c3af545e7acbd38b07251b3990914f1".parse::<Address>().unwrap());
    }
}
