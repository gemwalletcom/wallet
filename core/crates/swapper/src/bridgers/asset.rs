use std::{collections::BTreeSet, sync::LazyLock};

use primitives::{
    AssetId, Chain, ChainType,
    asset_constants::{
        ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, AVALANCHE_USDC_ASSET_ID, AVALANCHE_USDT_ASSET_ID, BASE_USDC_ASSET_ID, CELO_USDC_ASSET_ID, CELO_USDT_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, HYPEREVM_USDC_ASSET_ID,
        LINEA_USDC_E_ASSET_ID, MANTLE_USDT0_ASSET_ID, OPTIMISM_USDC_ASSET_ID, OPTIMISM_USDT_ASSET_ID, POLYGON_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID, SMARTCHAIN_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID,
        SOLANA_USDT_ASSET_ID, XLAYER_USDT_ASSET_ID,
    },
    contract_constants::EVM_NATIVE_TOKEN_ADDRESS,
};

use crate::{SwapperChainAsset, SwapperError, cross_chain::VaultAddresses};

#[derive(Debug, Clone, Copy)]
pub(super) struct Network {
    pub chain: Chain,
    pub code: &'static str,
    router: Option<&'static str>,
}

pub(super) const NETWORKS: [Network; 17] = [
    Network::new(Chain::SmartChain, "BSC", "0xc1d13492285eb664951E201BF7C80c7c6318a1b5"),
    Network::new(Chain::Polygon, "POLYGON", "0xc1d13492285eb664951E201BF7C80c7c6318a1b5"),
    Network::new(Chain::Arbitrum, "ARBITRUM", "0xc1d13492285eb664951E201BF7C80c7c6318a1b5"),
    Network::new(Chain::AvalancheC, "AVALANCHE", "0xc1d13492285eb664951E201BF7C80c7c6318a1b5"),
    Network::new(Chain::Optimism, "OPTIMISM", "0xc1d13492285eb664951E201BF7C80c7c6318a1b5"),
    Network::new(Chain::Base, "BASE", "0xa18968Cc31232724F1DBD0D1e8d0b323D89F3501"),
    Network::new(Chain::OpBNB, "opBNB", "0x8F957Ed3F969D7b6e5d6dF81e61A5Ff45F594dD1"),
    Network::new(Chain::Mantle, "MNT", "0xD1088D3376C2384D469d1c0d55D503695e1BE3E6"),
    Network::new(Chain::Linea, "LINEA", "0x8159891DFe9de7fC3bF1b665EB1aDda60F2aCd0e"),
    Network::new(Chain::Celo, "CELO", "0xD1088D3376C2384D469d1c0d55D503695e1BE3E6"),
    Network::new(Chain::XLayer, "XLayer", "0xD1088D3376C2384D469d1c0d55D503695e1BE3E6"),
    Network::new(Chain::Sonic, "Sonic", "0x89A70b162bE7dBc8b5e7579066fA58190C48d693"),
    Network::new(Chain::Robinhood, "Robinhood", "0x89A70b162bE7dBc8b5e7579066fA58190C48d693"),
    Network::new(Chain::Hyperliquid, "HyperEVM", "0x89A70b162bE7dBc8b5e7579066fA58190C48d693"),
    Network::destination(Chain::Ethereum, "ETH"),
    Network::destination(Chain::Solana, "SOLANA"),
    Network::destination(Chain::Bitcoin, "BTC"),
];

impl Network {
    const fn new(chain: Chain, code: &'static str, router: &'static str) -> Self {
        Self { chain, code, router: Some(router) }
    }

    const fn destination(chain: Chain, code: &'static str) -> Self {
        Self { chain, code, router: None }
    }

    pub fn router(&self) -> Result<&'static str, SwapperError> {
        self.router.ok_or(SwapperError::NotSupportedChain)
    }

    pub fn from_chain(chain: Chain) -> Result<Self, SwapperError> {
        NETWORKS.iter().find(|network| network.chain == chain).copied().ok_or(SwapperError::NotSupportedChain)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Token {
    pub asset_id: AssetId,
    pub code: &'static str,
    pub decimals: u32,
}

impl Token {
    fn native(chain: Chain, code: &'static str, decimals: u32) -> Self {
        Self {
            asset_id: AssetId::from_chain(chain),
            code,
            decimals,
        }
    }

    fn new(asset_id: &AssetId, code: &'static str, decimals: u32) -> Self {
        Self { asset_id: asset_id.clone(), code, decimals }
    }

    pub fn from_asset_id(asset_id: &AssetId) -> Result<Self, SwapperError> {
        TOKENS.iter().find(|token| &token.asset_id == asset_id).cloned().ok_or(SwapperError::NotSupportedAsset)
    }

    pub fn address(&self) -> String {
        self.asset_id.token_id.clone().unwrap_or_else(|| EVM_NATIVE_TOKEN_ADDRESS.to_lowercase())
    }
}

pub(super) static TOKENS: LazyLock<Vec<Token>> = LazyLock::new(|| {
    vec![
        Token::native(Chain::SmartChain, "BNB(BSC)", 18),
        Token::new(&SMARTCHAIN_USDC_ASSET_ID, "USDC(BSC)", 18),
        Token::new(&SMARTCHAIN_USDT_ASSET_ID, "USDT(BSC)", 18),
        Token::native(Chain::Polygon, "POL(POL)", 18),
        Token::new(&POLYGON_USDC_ASSET_ID, "USDC(POL)", 6),
        Token::new(&POLYGON_USDT_ASSET_ID, "USDT(POL)", 6),
        Token::native(Chain::Arbitrum, "ETH(ARB)", 18),
        Token::new(&ARBITRUM_USDC_ASSET_ID, "USDC(ARB)", 6),
        Token::new(&ARBITRUM_USDT_ASSET_ID, "USDT(ARB)", 6),
        Token::native(Chain::AvalancheC, "AVAX(C-Chain)", 18),
        Token::new(&AVALANCHE_USDC_ASSET_ID, "USDC(C-Chain)", 6),
        Token::new(&AVALANCHE_USDT_ASSET_ID, "USDT(C-Chain)", 6),
        Token::native(Chain::Optimism, "ETH(Optimism)", 18),
        Token::new(&OPTIMISM_USDC_ASSET_ID, "USDC(Optimism)", 6),
        Token::new(&OPTIMISM_USDT_ASSET_ID, "USDT(Optimism)", 6),
        Token::native(Chain::Base, "ETH(BASE)", 18),
        Token::new(&BASE_USDC_ASSET_ID, "USDC(BASE)", 6),
        Token::native(Chain::OpBNB, "BNB(opBNB)", 18),
        Token::new(&MANTLE_USDT0_ASSET_ID, "USDT0(MNT)", 6),
        Token::native(Chain::Linea, "ETH(LINEA)", 18),
        Token::new(&LINEA_USDC_E_ASSET_ID, "USDC(LINEA)", 6),
        Token::native(Chain::Celo, "CELO", 18),
        Token::new(&CELO_USDC_ASSET_ID, "USDC(CELO)", 6),
        Token::new(&CELO_USDT_ASSET_ID, "USDT(CELO)", 6),
        Token::native(Chain::XLayer, "OKB(XLayer)", 18),
        Token::new(&XLAYER_USDT_ASSET_ID, "USDT0(XLayer)", 6),
        Token::native(Chain::Sonic, "S(Sonic)", 18),
        Token::native(Chain::Robinhood, "ETH(Robinhood)", 18),
        Token::native(Chain::Hyperliquid, "HYPE(HyperEVM)", 18),
        Token::new(&HYPEREVM_USDC_ASSET_ID, "USDC(HyperEVM)", 6),
        Token::native(Chain::Ethereum, "ETH", 18),
        Token::new(&ETHEREUM_USDC_ASSET_ID, "USDC", 6),
        Token::new(&ETHEREUM_USDT_ASSET_ID, "USDT(ERC20)", 6),
        Token::native(Chain::Solana, "SOL", 9),
        Token::new(&SOLANA_USDC_ASSET_ID, "USDC(SOL)", 6),
        Token::new(&SOLANA_USDT_ASSET_ID, "USDT(SOL)", 6),
        Token::native(Chain::Bitcoin, "BTC", 8),
    ]
});

pub(super) fn supported_assets() -> Vec<SwapperChainAsset> {
    NETWORKS
        .iter()
        .map(|network| {
            let tokens = TOKENS.iter().filter(|token| token.asset_id.chain == network.chain && token.asset_id.is_token()).map(|token| token.asset_id.clone()).collect();
            SwapperChainAsset::Assets(network.chain, tokens)
        })
        .collect()
}

pub(super) fn vault_addresses() -> VaultAddresses {
    let deposit: Vec<String> = NETWORKS.iter().filter_map(|network| network.router).map(str::to_string).collect::<BTreeSet<_>>().into_iter().collect();
    let send = NETWORKS
        .iter()
        .filter(|network| network.chain.chain_type() == ChainType::Ethereum)
        .filter_map(|network| network.router)
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    VaultAddresses { deposit, send }
}
