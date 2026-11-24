use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Pubkey, anchor::AnchorAccount, find_program_address, instructions::program_ids::orca_whirlpool_program};

use super::whirlpool::{TICK_ARRAY_SIZE, Whirlpool};
use crate::{
    SwapperError,
    clmm::{ClmmTick, MAX_TICK_INDEX, MIN_TICK_INDEX},
};

const TICK_GROWTHS_LEN: usize = 96;

#[derive(Default)]
struct FixedTick {
    initialized: bool,
    liquidity_net: i128,
}

impl BorshDeserialize for FixedTick {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self {
            initialized: reader.read()?,
            liquidity_net: reader.read()?,
        };
        reader.skip(TICK_GROWTHS_LEN)?;
        Ok(value)
    }
}

struct FixedTickArray {
    start_tick_index: i32,
    ticks: [FixedTick; TICK_ARRAY_SIZE as usize],
    whirlpool: Pubkey,
}

impl BorshDeserialize for FixedTickArray {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            start_tick_index: reader.read()?,
            ticks: reader.read()?,
            whirlpool: reader.read()?,
        })
    }
}

struct DynamicTickData {
    liquidity_net: i128,
}

impl BorshDeserialize for DynamicTickData {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let value = Self { liquidity_net: reader.read()? };
        reader.skip(TICK_GROWTHS_LEN)?;
        Ok(value)
    }
}

#[derive(Default)]
enum DynamicTick {
    #[default]
    Uninitialized,
    Initialized(DynamicTickData),
}

impl BorshDeserialize for DynamicTick {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Initialized(reader.read()?)),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

struct DynamicTickArray {
    start_tick_index: i32,
    whirlpool: Pubkey,
    ticks: [DynamicTick; TICK_ARRAY_SIZE as usize],
}

impl BorshDeserialize for DynamicTickArray {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            start_tick_index: reader.read()?,
            whirlpool: reader.read()?,
            ticks: reader.skip(16)?.read()?,
        })
    }
}

impl AnchorAccount for FixedTickArray {
    const NAME: &'static str = "TickArray";
}

impl AnchorAccount for DynamicTickArray {
    const NAME: &'static str = "DynamicTickArray";
}

impl From<FixedTickArray> for TickArray {
    fn from(array: FixedTickArray) -> Self {
        Self {
            start_tick_index: array.start_tick_index,
            whirlpool: array.whirlpool,
            liquidity_nets: array.ticks.iter().map(|tick| tick.initialized.then_some(tick.liquidity_net)).collect(),
        }
    }
}

impl From<DynamicTickArray> for TickArray {
    fn from(array: DynamicTickArray) -> Self {
        Self {
            start_tick_index: array.start_tick_index,
            whirlpool: array.whirlpool,
            liquidity_nets: array
                .ticks
                .iter()
                .map(|tick| match tick {
                    DynamicTick::Uninitialized => None,
                    DynamicTick::Initialized(data) => Some(data.liquidity_net),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct TickArray {
    pub start_tick_index: i32,
    whirlpool: Pubkey,
    liquidity_nets: Vec<Option<i128>>,
}

impl TickArray {
    pub fn address(whirlpool: &Pubkey, start_tick_index: i32) -> Result<Pubkey, SwapperError> {
        Ok(find_program_address(&orca_whirlpool_program(), &[b"tick_array".as_slice(), whirlpool.as_bytes(), start_tick_index.to_string().as_bytes()])?.0)
    }

    pub fn addresses(whirlpool: &Pubkey, start_tick_indexes: &[i32]) -> Result<Vec<Pubkey>, SwapperError> {
        start_tick_indexes.iter().map(|start| Self::address(whirlpool, *start)).collect()
    }

    pub fn decode(data: &[u8]) -> Result<Self, SwapperError> {
        Ok(FixedTickArray::decode(data).map(Self::from).or_else(|_| DynamicTickArray::decode(data).map(Self::from))?)
    }

    fn initialized_ticks(&self, tick_spacing: u16, a_to_b: bool) -> Vec<ClmmTick> {
        let ticks = (0..TICK_ARRAY_SIZE).zip(&self.liquidity_nets).filter_map(|(offset, liquidity_net)| {
            liquidity_net.map(|liquidity_net| ClmmTick {
                index: self.start_tick_index + offset * i32::from(tick_spacing),
                liquidity_net,
            })
        });
        if a_to_b { ticks.rev().collect() } else { ticks.collect() }
    }
}

pub(super) fn swap_ticks(address: &Pubkey, whirlpool: &Whirlpool, a_to_b: bool, accounts: &[Option<Vec<u8>>]) -> Result<Vec<ClmmTick>, SwapperError> {
    let starts = whirlpool.swap_start_tick_indexes(a_to_b);
    let last_start = *starts.last().ok_or(SwapperError::NoQuoteAvailable)?;
    let arrays = starts
        .iter()
        .zip(accounts)
        .filter_map(|(start, account)| account.as_ref().map(|data| (start, data)))
        .map(|(start, data)| {
            let array = TickArray::decode(data)?;
            if array.start_tick_index != *start || array.whirlpool != *address {
                return Err(SwapperError::compute_quote_error("Unexpected tick array"));
            }
            Ok(array)
        })
        .collect::<Result<Vec<_>, SwapperError>>()?;
    let current = whirlpool.tick_current_index;
    let range_end = if a_to_b { last_start.max(MIN_TICK_INDEX) } else { (last_start + whirlpool.ticks_in_array() - 1).min(MAX_TICK_INDEX) };
    let boundary = ClmmTick { index: range_end, liquidity_net: 0 };
    Ok(arrays
        .iter()
        .flat_map(|array| array.initialized_ticks(whirlpool.tick_spacing, a_to_b))
        .filter(|tick| if a_to_b { tick.index <= current } else { tick.index > current })
        .chain([boundary])
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orca::testkit::{TEST_WHIRLPOOL, mock_dynamic_tick_array, mock_tick_array_accounts};

    #[test]
    fn test_decode() {
        let dynamic = TickArray::decode(&mock_dynamic_tick_array()).unwrap();
        let fixed = TickArray::decode(mock_tick_array_accounts(&[-22_528])[0].as_ref().unwrap()).unwrap();

        assert_eq!(TickArray::address(&Pubkey::from_base58(TEST_WHIRLPOOL).unwrap(), -11_264).unwrap().to_base58(), "81T5kNuPRkyVzhwbe2RpKR7wmQpGJ7RBkGPdTqyfa5vq");
        assert_eq!(dynamic.whirlpool.to_base58(), "8dZ2YFFshshetWR7m99yaymqWGk36FDwuptnKZWrqEUe");
        assert_eq!(
            dynamic.initialized_ticks(1, false),
            vec![
                ClmmTick {
                    index: -86_325,
                    liquidity_net: 971_423_482_536_773_736
                },
                ClmmTick {
                    index: -86_314,
                    liquidity_net: -971_423_482_536_773_736
                },
            ]
        );
        assert_eq!(fixed.start_tick_index, -22_528);
        assert_eq!(fixed.whirlpool.to_base58(), TEST_WHIRLPOOL);
        assert_eq!(TickArray::decode(&[0; 16]), Err(SwapperError::from(gem_solana::SolanaError::invalid_input("Invalid DynamicTickArray account"))));
    }
}
