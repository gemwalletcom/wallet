use alloy_primitives::{Address, B256, Bytes, Signature, U256, keccak256};
use primitives::{SignerError, TransactionFee, TransactionLoadInput};
use signer::{SignatureScheme, Signer};

use super::rlp;
use crate::transaction_params::{TransactionCalls, TransactionParams};
use crate::u256::bigint_to_u256;

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
    pub fn new(input: &TransactionLoadInput, fee: &TransactionFee, call: &TransactionParams) -> Result<Self, SignerError> {
        Ok(Self {
            chain_id: input.metadata.get_chain_id_u64()?,
            nonce: input.metadata.get_sequence()?,
            gas_limit: fee.gas_limit()?,
            max_fee_per_gas: fee.gas_price_u64()?.into(),
            max_priority_fee_per_gas: fee.priority_fee_u64()?.into(),
            to: call.to.parse().map_err(|_| SignerError::invalid_input("invalid to address"))?,
            value: bigint_to_u256(&call.value)?,
            input: Bytes::from(call.data.clone()),
        })
    }

    pub fn from_calls(input: &TransactionLoadInput, fee: &TransactionFee, calls: &TransactionCalls) -> Result<Vec<Self>, SignerError> {
        match calls {
            TransactionCalls::Call(call) => Ok(vec![Self::new(input, fee, call)?]),
            TransactionCalls::ApproveAndCall { approval, call, call_gas_limit } => {
                let approval = Self::new(input, fee, approval)?;
                let call = Self {
                    nonce: approval.nonce + 1,
                    gas_limit: *call_gas_limit,
                    ..Self::new(input, fee, call)?
                };
                Ok(vec![approval, call])
            }
        }
    }

    pub fn sign(&self, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, self.signature_hash().as_slice(), private_key)?;
        let signature = Signature::try_from(signature.as_slice()).map_err(SignerError::from_display)?;
        Ok(self.encoded(Some(&signature)))
    }

    pub fn encoded_unsigned(&self) -> Vec<u8> {
        self.encoded(None)
    }

    fn signature_hash(&self) -> B256 {
        keccak256(self.encoded_unsigned())
    }

    fn encoded(&self, signature: Option<&Signature>) -> Vec<u8> {
        let mut payload = Vec::new();
        rlp::encode_uint(&mut payload, &self.chain_id.to_be_bytes());
        rlp::encode_uint(&mut payload, &self.nonce.to_be_bytes());
        rlp::encode_uint(&mut payload, &self.max_priority_fee_per_gas.to_be_bytes());
        rlp::encode_uint(&mut payload, &self.max_fee_per_gas.to_be_bytes());
        rlp::encode_uint(&mut payload, &self.gas_limit.to_be_bytes());
        rlp::encode_bytes(&mut payload, self.to.as_slice());
        rlp::encode_uint(&mut payload, &self.value.to_be_bytes::<32>());
        rlp::encode_bytes(&mut payload, &self.input);
        rlp::encode_list(&mut payload, &[]);
        if let Some(signature) = signature {
            rlp::encode_uint(&mut payload, &[u8::from(signature.v())]);
            rlp::encode_uint(&mut payload, &signature.r().to_be_bytes::<32>());
            rlp::encode_uint(&mut payload, &signature.s().to_be_bytes::<32>());
        }

        let mut encoded = vec![EIP1559_TX_TYPE_ID];
        rlp::encode_list(&mut encoded, &payload);
        encoded
    }
}
