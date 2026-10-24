use bitcoin::ScriptBuf;
use num_traits::ToPrimitive;
use primitives::{Asset, BitcoinChain, SignerError, SignerInput, UTXO, swap::SwapQuoteDataType};

use crate::signer::address::script_for_address;

#[derive(Debug, Clone, PartialEq)]
pub enum SpendDestination {
    Script(ScriptBuf),
    External,
}

impl SpendDestination {
    pub fn script(chain: BitcoinChain, address: &str) -> Result<Self, SignerError> {
        Ok(Self::Script(script_for_address(chain, address)?.script_pubkey))
    }
}

#[derive(Debug, Clone)]
pub struct SpendRequest {
    pub chain: BitcoinChain,
    pub sender_address: String,
    pub destination: SpendDestination,
    pub amount: u64,
    pub is_max: bool,
    pub memo: Option<Vec<u8>>,
    pub utxos: Vec<UTXO>,
}

impl SpendRequest {
    pub fn transfer(chain: BitcoinChain, input: &SignerInput, destination: impl FnOnce(&str) -> Result<SpendDestination, SignerError>) -> Result<Self, SignerError> {
        validate_native_chain_asset(chain, input.input_type.get_asset(), "unsupported Bitcoin asset transfer")?;

        Ok(Self {
            chain,
            sender_address: input.sender_address.clone(),
            destination: destination(&input.destination_address)?,
            amount: input.value_as_u64()?,
            is_max: input.is_max_value,
            memo: input.get_memo().map(|memo| memo.as_bytes().to_vec()),
            utxos: input.metadata.get_utxos()?,
        })
    }

    pub fn swap(chain: BitcoinChain, input: &SignerInput, destination: impl FnOnce(&str) -> Result<SpendDestination, SignerError>) -> Result<Self, SignerError> {
        let swap = input.input_type.get_swap_data().map_err(|_| SignerError::invalid_input("unsupported Bitcoin transaction type"))?;
        validate_native_chain_asset(chain, input.input_type.get_asset(), "unsupported Bitcoin swap asset")?;
        let memo = match &swap.data.data_type {
            SwapQuoteDataType::Transfer => swap.data.memo.as_ref().map(|memo| memo.as_bytes().to_vec()),
            SwapQuoteDataType::Contract => return SignerError::invalid_input_err("unsupported Bitcoin contract swap"),
        };

        Ok(Self {
            chain,
            sender_address: input.sender_address.clone(),
            destination: destination(&swap.data.to)?,
            amount: swap.data.value.to_u64().ok_or_else(|| SignerError::invalid_input(format!("{} swap amount is too large", chain.get_chain())))?,
            is_max: swap.quote.use_max_amount.unwrap_or(input.is_max_value),
            memo,
            utxos: input.metadata.get_utxos()?,
        })
    }
}

fn validate_native_chain_asset(chain: BitcoinChain, asset: &Asset, message: &'static str) -> Result<(), SignerError> {
    (asset.id.chain == chain.get_chain() && asset.id.is_native()).then_some(()).ok_or_else(|| SignerError::invalid_input(message))
}
