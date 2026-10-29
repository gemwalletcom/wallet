use primitives::AssetId;

use super::{chain::ChainflipChain, client::chainflip_asset_index};
use crate::SwapperError;

const SOLANA_DESTINATION_OFFSET: usize = 16;

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

    pub(super) fn validate_solana(&self, data: &[u8]) -> Result<(), SwapperError> {
        let (chain, address, asset) = solana_destination(data).ok_or(SwapperError::TransactionError("invalid Chainflip Solana vault swap data".to_string()))?;
        self.validate_call(chain, address, asset)
    }

    pub(super) fn validate_tron(&self, note: &[u8]) -> Result<(), SwapperError> {
        let mut prefix = vec![self.asset, self.chain.scale_address_variant()];
        if self.chain == ChainflipChain::Bitcoin {
            prefix.push((self.address.len() as u8) << 2);
        }
        prefix.extend(&self.address);
        self.validate(note.starts_with(&prefix))
    }

    fn validate(&self, matches: bool) -> Result<(), SwapperError> {
        if !matches {
            return Err(SwapperError::TransactionError(format!("Chainflip {} destination mismatch", self.chain.as_ref())));
        }
        Ok(())
    }
}

fn solana_destination(data: &[u8]) -> Option<(u32, &[u8], u32)> {
    let read_u32 = |offset: usize| data.get(offset..offset + 4).and_then(|bytes| bytes.try_into().ok()).map(u32::from_le_bytes);
    let address_offset = SOLANA_DESTINATION_OFFSET + 8;
    let address_end = address_offset.checked_add(read_u32(SOLANA_DESTINATION_OFFSET + 4)? as usize)?;
    Some((read_u32(SOLANA_DESTINATION_OFFSET)?, data.get(address_offset..address_end)?, read_u32(address_end)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::testkit::{ARBITRUM_DESTINATION, BITCOIN_DESTINATION, bitcoin_destination};
    use primitives::Chain;

    #[test]
    fn test_validate_tron_note_bitcoin_destination() {
        let destination = bitcoin_destination();
        let note = [&[5u8, 2, (BITCOIN_DESTINATION.len() as u8) << 2][..], BITCOIN_DESTINATION.as_bytes(), &[0]].concat();

        assert_eq!(destination.validate_tron(&note), Ok(()));
        assert!(destination.validate_tron(&note[..note.len() - 2]).is_err());
    }

    #[test]
    fn test_new_rejects_invalid_address() {
        assert!(VaultSwapDestination::new(&AssetId::from_chain(Chain::Ethereum), BITCOIN_DESTINATION).is_err());
        assert!(VaultSwapDestination::new(&AssetId::from_chain(Chain::Solana), ARBITRUM_DESTINATION).is_err());
    }
}
