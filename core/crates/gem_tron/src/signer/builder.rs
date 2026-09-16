use gem_encoding::protobuf::{MessageEncode, proto_encode};
use gem_hash::sha2::sha256;
use primitives::{Address as _, SignerError, SignerInput, TransactionLoadMetadata, TronVote, hex::decode_hex_array};
use serde::Serialize;

use crate::address::TronAddress;
use crate::transaction::{TronContract, TronContractVote, protobuf};

const EXPIRATION_DURATION_MS: u64 = 10 * 60 * 60 * 1000;
const BLOCK_HASH_LEN: usize = 32;

#[derive(Debug)]
pub(crate) struct TronRawData {
    ref_block_bytes: Vec<u8>,
    ref_block_hash: Vec<u8>,
    expiration: u64,
    data: Option<Vec<u8>>,
    contract: TronContract,
    timestamp: u64,
    fee_limit: u64,
}

impl TronRawData {
    pub(crate) fn from_input_with_data(input: &SignerInput, contract: TronContract, fee_limit: u64, data: Option<Vec<u8>>) -> Result<Self, SignerError> {
        let TransactionLoadMetadata::Tron {
            block_number,
            block_version,
            block_timestamp,
            transaction_tree_root,
            parent_hash,
            witness_address,
            ..
        } = &input.metadata
        else {
            return SignerError::invalid_input_err("Missing tron metadata");
        };
        let transaction_tree_root = decode_hex_array::<BLOCK_HASH_LEN>(transaction_tree_root)?;
        let parent_hash = decode_hex_array::<BLOCK_HASH_LEN>(parent_hash)?;

        let header = BlockHeaderRaw {
            timestamp: (*block_timestamp > 0).then_some(*block_timestamp),
            tx_trie_root: Some(transaction_tree_root.to_vec()),
            parent_hash: Some(parent_hash.to_vec()),
            number: (*block_number > 0).then_some(*block_number),
            witness_address: Some(
                TronAddress::from_hex(witness_address)
                    .ok_or_else(|| SignerError::invalid_input("invalid Tron witness address"))?
                    .as_bytes()
                    .to_vec(),
            ),
            version: (*block_version > 0).then_some(*block_version),
        };
        let block_hash = sha256(&header.encode());
        let block_number_bytes = block_number.to_be_bytes();

        Ok(Self {
            ref_block_bytes: block_number_bytes[6..8].to_vec(),
            ref_block_hash: block_hash[8..16].to_vec(),
            expiration: block_timestamp
                .checked_add(EXPIRATION_DURATION_MS)
                .ok_or_else(|| SignerError::invalid_input("Tron expiration overflow"))?,
            data: data.or_else(|| input.get_memo().map(|memo| memo.as_bytes().to_vec())),
            contract,
            timestamp: *block_timestamp,
            fee_limit,
        })
    }

    pub(crate) fn encode(&self) -> Vec<u8> {
        protobuf::RawData {
            ref_block_bytes: Some(self.ref_block_bytes.clone()),
            ref_block_hash: Some(self.ref_block_hash.clone()),
            expiration: (self.expiration > 0).then_some(self.expiration),
            data: self.data.clone(),
            contracts: vec![protobuf::ContractEnvelope::from(&self.contract)],
            timestamp: (self.timestamp > 0).then_some(self.timestamp),
            fee_limit: (self.fee_limit > 0).then_some(self.fee_limit),
        }
        .encode()
    }

    pub(crate) fn json(&self) -> TronRawDataJson<'_> {
        TronRawDataJson {
            contract: vec![&self.contract],
            expiration: self.expiration,
            fee_limit: (self.fee_limit > 0).then_some(self.fee_limit),
            ref_block_bytes: hex::encode(&self.ref_block_bytes),
            ref_block_hash: hex::encode(&self.ref_block_hash),
            data: self.data.as_ref().map(hex::encode),
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TronRawDataJson<'a> {
    contract: Vec<&'a TronContract>,
    expiration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    fee_limit: Option<u64>,
    ref_block_bytes: String,
    ref_block_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    timestamp: u64,
}

#[derive(Serialize)]
pub(crate) struct SignedTransactionJson<'a> {
    raw_data: TronRawDataJson<'a>,
    raw_data_hex: String,
    signature: Vec<String>,
    #[serde(rename = "txID")]
    transaction_id: String,
}

impl<'a> SignedTransactionJson<'a> {
    pub(crate) fn new(raw_data: TronRawDataJson<'a>, raw_data_bytes: &[u8], transaction_id: &[u8], signature: String) -> Self {
        Self {
            raw_data,
            raw_data_hex: hex::encode(raw_data_bytes),
            signature: vec![signature],
            transaction_id: hex::encode(transaction_id),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct BlockHeaderRaw {
    timestamp: Option<u64>,
    tx_trie_root: Option<Vec<u8>>,
    parent_hash: Option<Vec<u8>>,
    number: Option<u64>,
    witness_address: Option<Vec<u8>>,
    version: Option<u64>,
}

proto_encode!(BlockHeaderRaw {
    1 => timestamp: optional_varint_u64,
    2 => tx_trie_root: optional_bytes,
    3 => parent_hash: optional_bytes,
    7 => number: optional_varint_u64,
    9 => witness_address: optional_bytes,
    10 => version: optional_varint_u64,
});

impl TronContract {
    pub(crate) fn vote_witness(owner: TronAddress, votes: &[TronVote]) -> Result<Self, SignerError> {
        Ok(Self::VoteWitness {
            owner,
            votes: votes.iter().map(TronContractVote::try_from).collect::<Result<Vec<_>, _>>()?,
            support: true,
        })
    }
}

impl TryFrom<&TronVote> for TronContractVote {
    type Error = SignerError;

    fn try_from(vote: &TronVote) -> Result<Self, Self::Error> {
        Ok(Self {
            address: TronAddress::parse(&vote.validator)?,
            count: vote.count,
        })
    }
}
