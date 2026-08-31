use crate::{
    ChainType,
    asset_constants::{
        APTOS_USDT_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID, HYPERCORE_SPOT_USDC_TOKEN_ID, NEAR_USDT_TOKEN_ID, SOLANA_USDC_TOKEN_ID, STELLAR_USDC_TOKEN_ID, SUI_USDC_TOKEN_ID,
        TON_USDT_TOKEN_ID, TRON_USDT_TOKEN_ID, XRP_RLUSD_TOKEN_ID,
    },
};

impl ChainType {
    pub fn mock_token_id(&self) -> Option<&'static str> {
        match self {
            ChainType::Ethereum => Some(ETHEREUM_USDT_TOKEN_ID),
            ChainType::Solana => Some(SOLANA_USDC_TOKEN_ID),
            ChainType::Ton => Some(TON_USDT_TOKEN_ID),
            ChainType::Tron => Some(TRON_USDT_TOKEN_ID),
            ChainType::Aptos => Some(APTOS_USDT_TOKEN_ID),
            ChainType::Sui => Some(SUI_USDC_TOKEN_ID),
            ChainType::Near => Some(NEAR_USDT_TOKEN_ID),
            ChainType::Algorand => Some("31566704"),
            ChainType::Xrp => Some(XRP_RLUSD_TOKEN_ID),
            ChainType::Stellar => Some(STELLAR_USDC_TOKEN_ID),
            ChainType::HyperCore => Some(HYPERCORE_SPOT_USDC_TOKEN_ID),
            ChainType::Cosmos => Some("ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349"),
            ChainType::Bitcoin | ChainType::Polkadot | ChainType::Cardano => None,
        }
    }
}
