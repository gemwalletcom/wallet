use std::{collections::BTreeSet, sync::LazyLock};

use primitives::{
    AssetId, Chain,
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
    payout: Option<&'static str>,
}

pub(super) const NETWORKS: [Network; 19] = [
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
    Network::payout(Chain::Bitcoin, "BTC", "bc1q7t4vyehjsexdme84qhdgd4dawcn54djh0m78fz"),
    Network::payout(Chain::Litecoin, "LTC", "ltc1q0sc0myh0a2wap7eshhyz2ahc2xdn86g7fv32r2"),
    Network::payout(Chain::Doge, "DOGE", "DNqWDenhxgWTfy2bpJn4tES6uMwz1nqt6C"),
];

impl Network {
    const fn new(chain: Chain, code: &'static str, router: &'static str) -> Self {
        Self {
            chain,
            code,
            router: Some(router),
            payout: None,
        }
    }

    const fn destination(chain: Chain, code: &'static str) -> Self {
        Self { chain, code, router: None, payout: None }
    }

    const fn payout(chain: Chain, code: &'static str, address: &'static str) -> Self {
        Self {
            chain,
            code,
            router: None,
            payout: Some(address),
        }
    }

    pub fn router(&self) -> Result<&'static str, SwapperError> {
        self.router.ok_or(SwapperError::NotSupportedChain)
    }

    pub fn from_chain(chain: Chain) -> Result<Self, SwapperError> {
        NETWORKS.iter().find(|network| network.chain == chain).copied().ok_or(SwapperError::NotSupportedChain)
    }

    pub fn from_source_chain(chain: Chain) -> Result<Self, SwapperError> {
        Some(Self::from_chain(chain)?).filter(|network| network.router.is_some()).ok_or(SwapperError::NotSupportedChain)
    }
}

static TOKENS: LazyLock<Vec<(AssetId, &'static str)>> = LazyLock::new(|| {
    vec![
        (AssetId::from_chain(Chain::SmartChain), "BNB(BSC)"),
        (SMARTCHAIN_USDC_ASSET_ID.clone(), "USDC(BSC)"),
        (SMARTCHAIN_USDT_ASSET_ID.clone(), "USDT(BSC)"),
        (AssetId::from_chain(Chain::Polygon), "POL(POL)"),
        (POLYGON_USDC_ASSET_ID.clone(), "USDC(POL)"),
        (POLYGON_USDT_ASSET_ID.clone(), "USDT(POL)"),
        (AssetId::from_chain(Chain::Arbitrum), "ETH(ARB)"),
        (ARBITRUM_USDC_ASSET_ID.clone(), "USDC(ARB)"),
        (ARBITRUM_USDT_ASSET_ID.clone(), "USDT(ARB)"),
        (AssetId::from_chain(Chain::AvalancheC), "AVAX(C-Chain)"),
        (AVALANCHE_USDC_ASSET_ID.clone(), "USDC(C-Chain)"),
        (AVALANCHE_USDT_ASSET_ID.clone(), "USDT(C-Chain)"),
        (AssetId::from_chain(Chain::Optimism), "ETH(Optimism)"),
        (OPTIMISM_USDC_ASSET_ID.clone(), "USDC(Optimism)"),
        (OPTIMISM_USDT_ASSET_ID.clone(), "USDT(Optimism)"),
        (AssetId::from_chain(Chain::Base), "ETH(BASE)"),
        (BASE_USDC_ASSET_ID.clone(), "USDC(BASE)"),
        (AssetId::from_chain(Chain::OpBNB), "BNB(opBNB)"),
        (MANTLE_USDT0_ASSET_ID.clone(), "USDT0(MNT)"),
        (AssetId::from_chain(Chain::Linea), "ETH(LINEA)"),
        (LINEA_USDC_E_ASSET_ID.clone(), "USDC(LINEA)"),
        (AssetId::from_chain(Chain::Celo), "CELO"),
        (CELO_USDC_ASSET_ID.clone(), "USDC(CELO)"),
        (CELO_USDT_ASSET_ID.clone(), "USDT(CELO)"),
        (AssetId::from_chain(Chain::XLayer), "OKB(XLayer)"),
        (XLAYER_USDT_ASSET_ID.clone(), "USDT0(XLayer)"),
        (AssetId::from_chain(Chain::Sonic), "S(Sonic)"),
        (AssetId::from_chain(Chain::Robinhood), "ETH(Robinhood)"),
        (AssetId::from_chain(Chain::Hyperliquid), "HYPE(HyperEVM)"),
        (HYPEREVM_USDC_ASSET_ID.clone(), "USDC(HyperEVM)"),
        (AssetId::from_chain(Chain::Ethereum), "ETH"),
        (ETHEREUM_USDC_ASSET_ID.clone(), "USDC"),
        (ETHEREUM_USDT_ASSET_ID.clone(), "USDT(ERC20)"),
        (AssetId::from_chain(Chain::Solana), "SOL"),
        (SOLANA_USDC_ASSET_ID.clone(), "USDC(SOL)"),
        (SOLANA_USDT_ASSET_ID.clone(), "USDT(SOL)"),
        (AssetId::from_chain(Chain::Bitcoin), "BTC"),
        (AssetId::from_chain(Chain::Litecoin), "LTC"),
        (AssetId::from_chain(Chain::Doge), "DOGE"),
    ]
});

pub(super) fn get_token_code(asset_id: &AssetId) -> Result<&'static str, SwapperError> {
    TOKENS.iter().find(|(id, _)| id == asset_id).map(|(_, code)| *code).ok_or(SwapperError::NotSupportedAsset)
}

pub(super) fn get_token_address(asset_id: &AssetId) -> Result<String, SwapperError> {
    TOKENS.iter().find(|(id, _)| id == asset_id).map(|(id, _)| token_address(id)).ok_or(SwapperError::NotSupportedAsset)
}

fn token_address(asset_id: &AssetId) -> String {
    asset_id.token_id.clone().unwrap_or_else(|| EVM_NATIVE_TOKEN_ADDRESS.to_lowercase())
}

pub(super) fn get_asset_id(code: &str, address: &str) -> Option<AssetId> {
    let chain = NETWORKS.iter().find(|network| network.code == code)?.chain;
    TOKENS.iter().map(|(id, _)| id).find(|id| id.chain == chain && token_address(id) == address).cloned()
}

pub(super) fn supported_assets() -> Vec<SwapperChainAsset> {
    NETWORKS
        .iter()
        .map(|network| {
            let tokens = TOKENS.iter().filter(|(id, _)| id.chain == network.chain && id.is_token()).map(|(id, _)| id.clone()).collect();
            SwapperChainAsset::Assets(network.chain, tokens)
        })
        .collect()
}

pub(super) fn vault_addresses() -> VaultAddresses {
    let routers: Vec<String> = NETWORKS.iter().filter_map(|network| network.router).map(str::to_string).collect::<BTreeSet<_>>().into_iter().collect();
    let payouts = NETWORKS.iter().filter_map(|network| network.payout).map(str::to_string);
    VaultAddresses {
        deposit: routers.clone(),
        send: routers.into_iter().chain(payouts).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_from_chain() {
        assert_eq!(Network::from_chain(Chain::OpBNB).unwrap().code, "opBNB");
        assert_eq!(Network::from_chain(Chain::Hyperliquid).unwrap().code, "HyperEVM");
        assert_eq!(Network::from_chain(Chain::Mantle).unwrap().code, "MNT");
        assert_eq!(Network::from_chain(Chain::Litecoin).unwrap().router(), Err(SwapperError::NotSupportedChain));
        assert_eq!(Network::from_chain(Chain::Bitcoin).unwrap().router(), Err(SwapperError::NotSupportedChain));
        assert_eq!(Network::from_chain(Chain::Tron).unwrap_err(), SwapperError::NotSupportedChain);
        assert_eq!(Network::from_source_chain(Chain::OpBNB).unwrap().code, "opBNB");
        assert_eq!(Network::from_source_chain(Chain::Bitcoin).unwrap_err(), SwapperError::NotSupportedChain);
    }

    #[test]
    fn test_vault_addresses() {
        let vault_addresses = vault_addresses();

        assert!(!vault_addresses.deposit.contains(&"ltc1q0sc0myh0a2wap7eshhyz2ahc2xdn86g7fv32r2".to_string()));
        assert!(vault_addresses.send.contains(&"ltc1q0sc0myh0a2wap7eshhyz2ahc2xdn86g7fv32r2".to_string()));
        assert!(vault_addresses.send.contains(&"0xD1088D3376C2384D469d1c0d55D503695e1BE3E6".to_string()));
    }
}
