use alloy_primitives::{Address, B256, Bytes, Signature, U256, keccak256};
use alloy_rlp::{Encodable, Header};
use primitives::SignerError;
use signer::{SignatureScheme, Signer};

const EIP1559_TX_TYPE_ID: u8 = 0x02;

pub struct Eip1559Transaction {
    pub chain_id: u64,
    pub nonce: u64,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub to: Address,
    pub value: U256,
    pub input: Bytes,
}

impl Eip1559Transaction {
    pub fn sign(&self, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, self.signature_hash().as_slice(), private_key)?;
        let signature = Signature::try_from(signature.as_slice()).map_err(SignerError::from_display)?;
        Ok(self.encoded(Some(&signature)))
    }

    fn signature_hash(&self) -> B256 {
        keccak256(self.encoded(None))
    }

    fn encoded(&self, signature: Option<&Signature>) -> Vec<u8> {
        let mut payload = Vec::new();
        self.chain_id.encode(&mut payload);
        self.nonce.encode(&mut payload);
        self.max_priority_fee_per_gas.encode(&mut payload);
        self.max_fee_per_gas.encode(&mut payload);
        self.gas_limit.encode(&mut payload);
        self.to.encode(&mut payload);
        self.value.encode(&mut payload);
        self.input.encode(&mut payload);
        Header { list: true, payload_length: 0 }.encode(&mut payload);
        if let Some(signature) = signature {
            signature.write_rlp_vrs(&mut payload, signature.v());
        }

        let mut encoded = vec![EIP1559_TX_TYPE_ID];
        Header { list: true, payload_length: payload.len() }.encode(&mut encoded);
        encoded.extend_from_slice(&payload);
        encoded
    }
}
