use alloy_sol_types::{Result, SolCall, SolInterface, SolType, SolValue, abi::AbiDecoderConfig, abi::TokenSeq};

fn bounded(data: &[u8]) -> AbiDecoderConfig {
    AbiDecoderConfig::new().memory_limit(data.len())
}

pub trait SolCallBounded: SolCall {
    fn abi_decode_bounded(data: &[u8]) -> Result<Self> {
        Self::abi_decode_with_config(data, bounded(data))
    }
}

impl<T: SolCall> SolCallBounded for T {}

pub trait SolInterfaceBounded: SolInterface {
    fn abi_decode_bounded(data: &[u8]) -> Result<Self> {
        Self::abi_decode_with_config(data, bounded(data))
    }
}

impl<T: SolInterface> SolInterfaceBounded for T {}

pub trait SolValueBounded: SolValue {
    fn abi_decode_bounded(data: &[u8]) -> Result<Self>
    where
        Self: From<<Self::SolType as SolType>::RustType>,
    {
        Self::abi_decode_with_config(data, bounded(data))
    }

    fn abi_decode_sequence_bounded<'de>(data: &'de [u8]) -> Result<Self>
    where
        Self: From<<Self::SolType as SolType>::RustType>,
        <Self::SolType as SolType>::Token<'de>: TokenSeq<'de>,
    {
        Self::abi_decode_sequence_with_config(data, bounded(data))
    }
}

impl<T: SolValue> SolValueBounded for T {}

pub trait SolTypeBounded: SolType {
    fn abi_decode_sequence_bounded<'de>(data: &'de [u8]) -> Result<Self::RustType>
    where
        Self::Token<'de>: TokenSeq<'de>,
    {
        Self::abi_decode_sequence_with_config(data, bounded(data))
    }
}

impl<T: SolType> SolTypeBounded for T {}
