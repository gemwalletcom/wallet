use primitives::AssetId;

use super::{chain::ChainflipChain, client::chainflip_asset_index};
use crate::SwapperError;

const SOLANA_DESTINATION_OFFSET: usize = 16;
pub(super) const NO_CCM: u8 = 0;
const SCALE_TWO_BYTE_LENGTH: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSwapDestination {
    chain: ChainflipChain,
    asset: u8,
    address: Vec<u8>,
}

impl VaultSwapDestination {
    pub(super) fn new(asset_id: &AssetId, address: &str) -> Result<Self, SwapperError> {
        let chain = ChainflipChain::from_chain(asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;
        let asset = chainflip_asset_index(asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        let address = chain.encode_address(address).ok_or_else(|| SwapperError::TransactionError(format!("invalid Chainflip destination address: {address}")))?;
        Ok(Self { chain, asset, address })
    }

    pub(super) fn validate_call(&self, chain: u32, address: &[u8], asset: u32) -> Result<(), SwapperError> {
        self.validate(chain == self.chain.id() && address == self.address && asset == u32::from(self.asset))
    }

    pub(super) fn validate_solana<'a>(&self, data: &'a [u8]) -> Result<&'a [u8], SwapperError> {
        let (chain, address, asset, cf_parameters) = solana_destination(data).ok_or(SwapperError::TransactionError("invalid Chainflip Solana vault swap data".to_string()))?;
        self.validate_call(chain, address, asset)?;
        Ok(cf_parameters)
    }

    pub(super) fn validate_tron<'a>(&self, note: &'a [u8]) -> Result<&'a [u8], SwapperError> {
        let mut prefix = vec![self.asset, self.chain.scale_address_variant()];
        if self.chain == ChainflipChain::Bitcoin {
            prefix.push((self.address.len() as u8) << 2);
        }
        prefix.extend(&self.address);
        self.validate(note.starts_with(&prefix))?;
        tron_cf_parameters(&note[prefix.len()..]).ok_or(SwapperError::TransactionError("invalid Chainflip Tron vault swap note".to_string()))
    }

    fn validate(&self, matches: bool) -> Result<(), SwapperError> {
        if !matches {
            return Err(SwapperError::TransactionError(format!("Chainflip {} destination mismatch", self.chain.as_ref())));
        }
        Ok(())
    }
}

fn solana_destination(data: &[u8]) -> Option<(u32, &[u8], u32, &[u8])> {
    let read_u32 = |offset: usize| data.get(offset..offset + 4).and_then(|bytes| bytes.try_into().ok()).map(u32::from_le_bytes);
    let address_offset = SOLANA_DESTINATION_OFFSET + 8;
    let address_end = address_offset.checked_add(read_u32(SOLANA_DESTINATION_OFFSET + 4)? as usize)?;
    let address = data.get(address_offset..address_end)?;
    let ccm_offset = address_end + 4;
    data.get(ccm_offset).filter(|ccm| **ccm == NO_CCM)?;
    let cf_parameters_offset = ccm_offset + 5;
    let cf_parameters_end = cf_parameters_offset.checked_add(read_u32(ccm_offset + 1)? as usize)?;
    Some((read_u32(SOLANA_DESTINATION_OFFSET)?, address, read_u32(address_end)?, data.get(cf_parameters_offset..cf_parameters_end)?))
}

fn tron_cf_parameters(data: &[u8]) -> Option<&[u8]> {
    let (&[ccm, low, high], rest) = data.split_first_chunk::<3>()?;
    let length = u16::from_le_bytes([low, high]);
    if ccm != NO_CCM || length & 0b11 != SCALE_TWO_BYTE_LENGTH {
        return None;
    }
    rest.get(..usize::from(length >> 2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::testkit::bitcoin_destination;
    use primitives::Chain;
    use primitives::testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_SENDER};

    #[test]
    fn test_validate_tron_note_bitcoin_destination() {
        let destination = bitcoin_destination();
        let note = [&[5u8, 2, (TEST_BITCOIN_SENDER.len() as u8) << 2][..], TEST_BITCOIN_SENDER.as_bytes(), &[0, 0x91, 1], &[7; 100]].concat();

        assert_eq!(destination.validate_tron(&note), Ok(&[7; 100][..]));
        assert!(destination.validate_tron(&note[..note.len() - 1]).is_err());
    }

    #[test]
    fn test_new_rejects_invalid_address() {
        assert!(VaultSwapDestination::new(&AssetId::from_chain(Chain::Ethereum), TEST_BITCOIN_SENDER).is_err());
        assert!(VaultSwapDestination::new(&AssetId::from_chain(Chain::Solana), TEST_EVM_SENDER).is_err());
    }
}
