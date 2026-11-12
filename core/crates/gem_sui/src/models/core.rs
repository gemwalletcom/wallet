use super::OwnedCoins;
use crate::types::{Address, Digest, ObjectInput, Transaction, transaction_signing_digest};
use gem_encoding::encode_base64;
use std::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coin {
    pub coin_type: String,
    pub balance: u64,
    pub object: Object,
}

impl Coin {
    pub fn to_input(&self) -> ObjectInput {
        self.object.to_input()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Object {
    pub object_id: Address,
    pub digest: Digest,
    pub version: u64,
}

impl Object {
    pub fn to_input(&self) -> ObjectInput {
        ObjectInput::immutable_or_owned(self.object_id, self.version, self.digest)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gas {
    pub budget: u64,
    pub price: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StakeInput {
    pub sender: String,
    pub validator: String,
    pub stake_amount: u64,
    pub gas: Gas,
    pub coins: OwnedCoins<Coin>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnstakeInput {
    pub sender: String,
    pub staked_sui: Object,
    pub gas: Gas,
    pub gas_coin: Coin,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub coins: OwnedCoins<Coin>,
    pub send_max: bool,
    pub gas: Gas,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub tokens: OwnedCoins<Coin>,
    pub gas: Gas,
    pub gas_coin: Coin,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TxOutput {
    pub tx_data: Vec<u8>,
    pub hash: Vec<u8>,
}

impl TxOutput {
    pub fn from_tx(tx_data: &Transaction) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let tx_data = bcs::to_bytes(tx_data)?;
        let hash = transaction_signing_digest(&tx_data).to_vec();
        Ok(Self { tx_data, hash })
    }

    pub fn base64_encoded(&self) -> String {
        encode_base64(&self.tx_data)
    }
}
