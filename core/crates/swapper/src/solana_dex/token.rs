use gem_solana::{Pubkey, SolanaTokenProgramIdExt, associated_token::get_associated_token_address_with_program_id, get_pubkey_by_asset, instructions::program_ids::token_program, models::Mint, wrapped_sol::wrapped_sol_mint};
use hex_lit::hex;
use primitives::SolanaTokenProgramId;

use crate::{SwapperError, SwapperQuoteAsset};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SwapToken {
    pub mint: Pubkey,
    pub token_program: Pubkey,
    pub decimals: u8,
    pub is_native: bool,
}

pub(crate) const USDC_MINT: Pubkey = Pubkey::new(hex!("c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d61"));

const WRAPPED_SOL: SwapToken = SwapToken {
    mint: wrapped_sol_mint(),
    token_program: token_program(),
    decimals: 9,
    is_native: false,
};

pub(crate) static SOL_INTERMEDIATE: [SwapToken; 1] = [WRAPPED_SOL];

pub(crate) static INTERMEDIATE_TOKENS: [SwapToken; 2] = [
    WRAPPED_SOL,
    SwapToken {
        mint: USDC_MINT,
        token_program: token_program(),
        decimals: 6,
        is_native: false,
    },
];

impl SwapToken {
    pub fn from_asset(asset: &SwapperQuoteAsset) -> Result<Self, SwapperError> {
        let asset_id = asset.asset_id();
        let token_program = if asset_id.is_native() {
            token_program()
        } else {
            SolanaTokenProgramId::from_asset_type(&asset.asset_type).ok_or(SwapperError::NotSupportedAsset)?.program_id()
        };
        Ok(Self {
            mint: get_pubkey_by_asset(&asset_id).ok_or(SwapperError::NotSupportedAsset)?,
            token_program,
            decimals: u8::try_from(asset.decimals).map_err(|_| SwapperError::NotSupportedAsset)?,
            is_native: asset_id.is_native(),
        })
    }

    pub fn account(&self, owner: &Pubkey) -> Result<Pubkey, SwapperError> {
        get_associated_token_address_with_program_id(owner, &self.mint, &self.token_program).map_err(SwapperError::transaction_error)
    }
}

pub(crate) fn decode_mint(data: Option<Vec<u8>>) -> Result<Mint, SwapperError> {
    data.and_then(|data| Mint::decode(&data).ok()).filter(|mint| !mint.has_transfer_restrictions()).ok_or(SwapperError::NotSupportedAsset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::testkit::mock_solana_asset;
    use gem_solana::{USDC_TOKEN_MINT, instructions::program_ids::token_2022_program};
    use primitives::{AssetId, AssetType, Chain};

    #[test]
    fn test_from_asset() {
        let usdc = mock_solana_asset(Some(USDC_TOKEN_MINT), 6);
        let sol = SwapToken::from_asset(&SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana))).unwrap();

        assert_eq!((sol.mint, sol.token_program, sol.is_native), (wrapped_sol_mint(), token_program(), true));
        assert_eq!(SwapToken::from_asset(&usdc).unwrap(), SwapToken::mock());
        assert_eq!(
            SwapToken::from_asset(&SwapperQuoteAsset {
                asset_type: AssetType::SPL2022,
                ..usdc.clone()
            })
            .unwrap()
            .token_program,
            token_2022_program()
        );
        assert_eq!(SwapToken::from_asset(&SwapperQuoteAsset { asset_type: AssetType::ERC20, ..usdc }), Err(SwapperError::NotSupportedAsset));
    }

    #[test]
    fn test_usdc_mint() {
        assert_eq!(USDC_MINT.to_base58(), USDC_TOKEN_MINT);
    }

    #[test]
    fn test_decode_mint() {
        let non_transferable = [[0; 165].as_slice(), &[1], &[9, 0, 0, 0]].concat();

        assert_eq!(decode_mint(Some(vec![0; 82])), Ok(Mint { supply: 0, extensions: vec![] }));
        assert_eq!(decode_mint(Some(non_transferable)), Err(SwapperError::NotSupportedAsset));
        assert_eq!(decode_mint(Some(vec![0; 10])), Err(SwapperError::NotSupportedAsset));
        assert_eq!(decode_mint(None), Err(SwapperError::NotSupportedAsset));
    }
}
