use super::{asset::AcrossAsset, fees::CapitalCostConfig};
use crate::ether_conv::EtherConv;
use alloy_primitives::{Address, map::HashSet};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, asset_constants::*, contract_constants::*};
use std::{collections::HashMap, vec};

pub struct AcrossDeployment {
    pub chain_id: u32,
    pub spoke_pool: &'static str,
    pub multicall_handler: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenPrice {
    Dollar,
    Chainlink { chain: Chain, feed: &'static str },
}

pub const ETHER_PRICE: TokenPrice = TokenPrice::Chainlink {
    chain: Chain::Ethereum,
    feed: ETHEREUM_CHAINLINK_ETH_USD_FEED_CONTRACT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillCost {
    pub gas_token: TokenPrice,
    pub gas: u64,
}

#[derive(Debug)]
pub struct AssetMapping {
    pub capital_cost: CapitalCostConfig,
    pub price: Option<TokenPrice>,
    pub set: HashSet<AssetId>,
}

impl AcrossDeployment {
    fn chain_id(chain: &Chain) -> Option<u32> {
        chain.network_id_value().and_then(|network_id| u32::try_from(network_id).ok())
    }

    pub fn deployment_by_chain(chain: &Chain) -> Option<Self> {
        let chain_id = Self::chain_id(chain)?;
        let spoke_pool = match chain {
            Chain::Ethereum => ETHEREUM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Arbitrum => ARBITRUM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::AvalancheC => AVALANCHE_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Base => BASE_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Linea => LINEA_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Optimism => OPTIMISM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Polygon => POLYGON_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::World => WORLD_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::ZkSync => ZKSYNC_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Ink => INK_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Unichain => UNICHAIN_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Monad => MONAD_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::SmartChain => SMARTCHAIN_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Hyperliquid => HYPEREVM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Plasma => PLASMA_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Robinhood => ROBINHOOD_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Arc => ARC_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Tron => TRON_ACROSS_SPOKE_POOL_CONTRACT,
            _ => return None,
        };
        let multicall_handler = match chain {
            Chain::AvalancheC => AVALANCHE_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Linea => LINEA_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::ZkSync => ZKSYNC_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::SmartChain => SMARTCHAIN_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Monad => MONAD_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Hyperliquid => HYPEREVM_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Plasma => PLASMA_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Robinhood => ROBINHOOD_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Arc => ARC_ACROSS_MULTICALL_HANDLER_CONTRACT,
            Chain::Ethereum | Chain::Arbitrum | Chain::Base | Chain::Optimism | Chain::Polygon | Chain::World | Chain::Ink | Chain::Unichain | Chain::Tron => ETHEREUM_ACROSS_MULTICALL_HANDLER_CONTRACT,
            _ => return None,
        };
        Some(Self { chain_id, spoke_pool, multicall_handler })
    }

    pub fn multicall_handler(&self) -> &'static str {
        self.multicall_handler
    }

    pub fn fill_cost(chain: &Chain) -> Option<FillCost> {
        let chainlink = |feed| TokenPrice::Chainlink { chain: *chain, feed };
        let (gas_token, gas) = match chain {
            Chain::Ethereum => (ETHER_PRICE, 310_000),
            Chain::Arbitrum => (ETHER_PRICE, 380_000),
            Chain::AvalancheC => (chainlink(AVALANCHE_CHAINLINK_AVAX_USD_FEED_CONTRACT), 360_000),
            Chain::Base => (ETHER_PRICE, 550_000),
            Chain::Linea => (ETHER_PRICE, 210_000),
            Chain::Optimism => (ETHER_PRICE, 250_000),
            Chain::Polygon => (chainlink(POLYGON_CHAINLINK_POL_USD_FEED_CONTRACT), 610_000),
            Chain::World => (ETHER_PRICE, 370_000),
            Chain::ZkSync => (ETHER_PRICE, 420_000),
            Chain::Ink => (ETHER_PRICE, 250_000),
            Chain::Unichain => (ETHER_PRICE, 280_000),
            Chain::Monad => (chainlink(MONAD_CHAINLINK_MON_USD_FEED_CONTRACT), 530_000),
            Chain::SmartChain => (chainlink(SMARTCHAIN_CHAINLINK_BNB_USD_FEED_CONTRACT), 200_000),
            Chain::Hyperliquid => (chainlink(HYPEREVM_CHAINLINK_HYPE_USD_FEED_CONTRACT), 470_000),
            Chain::Plasma => (chainlink(PLASMA_CHAINLINK_XPL_USD_FEED_CONTRACT), 240_000),
            Chain::Robinhood => (ETHER_PRICE, 400_000),
            Chain::Arc => (TokenPrice::Dollar, 320_000),
            _ => return None,
        };
        Some(FillCost { gas_token, gas })
    }

    pub fn supported_assets() -> HashMap<Chain, Vec<AssetId>> {
        HashMap::from([
            (Chain::Ethereum, vec![ETHEREUM_USDC_ASSET_ID.clone(), ETHEREUM_USDT_ASSET_ID.clone(), ETHEREUM_WETH_ASSET_ID.clone()]),
            (Chain::Optimism, vec![OPTIMISM_USDT_ASSET_ID.clone(), OPTIMISM_USDC_ASSET_ID.clone(), OPTIMISM_WETH_ASSET_ID.clone()]),
            (Chain::Polygon, vec![POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDT_ASSET_ID.clone(), POLYGON_WETH_ASSET_ID.clone()]),
            (Chain::Arbitrum, vec![ARBITRUM_USDT_ASSET_ID.clone(), ARBITRUM_USDC_ASSET_ID.clone(), ARBITRUM_WETH_ASSET_ID.clone()]),
            (Chain::Base, vec![BASE_WETH_ASSET_ID.clone(), BASE_USDC_ASSET_ID.clone()]),
            (Chain::AvalancheC, vec![AVALANCHE_USDC_ASSET_ID.clone(), AVALANCHE_USDT_ASSET_ID.clone()]),
            (Chain::Hyperliquid, vec![HYPEREVM_USDC_ASSET_ID.clone(), HYPEREVM_USDT_ASSET_ID.clone()]),
            (Chain::Linea, vec![LINEA_USDT_ASSET_ID.clone(), LINEA_WETH_ASSET_ID.clone(), LINEA_USDC_E_ASSET_ID.clone()]),
            (Chain::ZkSync, vec![ZKSYNC_WETH_ASSET_ID.clone(), ZKSYNC_USDT_ASSET_ID.clone(), ZKSYNC_USDC_E_ASSET_ID.clone()]),
            (Chain::World, vec![WORLD_WETH_ASSET_ID.clone(), WORLD_USDC_E_ASSET_ID.clone()]),
            (Chain::Ink, vec![INK_WETH_ASSET_ID.clone(), INK_USDT0_ASSET_ID.clone(), INK_USDC_ASSET_ID.clone()]),
            (Chain::Unichain, vec![UNICHAIN_WETH_ASSET_ID.clone(), UNICHAIN_USDC_ASSET_ID.clone(), UNICHAIN_USDT0_ASSET_ID.clone()]),
            (Chain::Monad, vec![MONAD_USDC_ASSET_ID.clone(), MONAD_USDT_ASSET_ID.clone()]),
            (Chain::SmartChain, vec![SMARTCHAIN_ETH_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone(), SMARTCHAIN_USDT_ASSET_ID.clone()]),
            (Chain::Plasma, vec![PLASMA_USDT_ASSET_ID.clone()]),
            (Chain::Robinhood, vec![ROBINHOOD_WETH_ASSET_ID.clone(), ROBINHOOD_USDG_ASSET_ID.clone()]),
            (Chain::Arc, vec![]),
            (Chain::Tron, vec![TRON_USDT_ASSET_ID.clone()]),
        ])
    }

    pub fn supported_asset_for_token(chain: Chain, token: Address) -> Option<AssetId> {
        let token_address = |asset: &AssetId| asset.token_id.as_deref().and_then(|token_id| token_id.parse::<Address>().ok());
        let native = chain.as_asset_id();
        if AcrossAsset::from_asset(&native).and_then(|routed| token_address(&routed.asset_id)) == Some(token) {
            return Some(native);
        }
        let mapped_assets = Self::asset_mappings().into_iter().flat_map(|mapping| mapping.set).filter(|asset| asset.chain == chain);
        Self::supported_assets().remove(&chain).unwrap_or_default().into_iter().chain(mapped_assets).find(|asset| token_address(asset) == Some(token))
    }

    pub fn deposit_addresses() -> Vec<String> {
        let mut addresses: HashSet<String> = HashSet::default();
        for chain in Chain::all() {
            if let Some(deployment) = Self::deployment_by_chain(&chain) {
                addresses.insert(deployment.spoke_pool.to_string());
            }
        }
        addresses.into_iter().collect()
    }

    pub fn send_addresses() -> Vec<String> {
        let mut addresses: HashSet<String> = HashSet::default();
        for chain in Chain::all() {
            if let Some(deployment) = Self::deployment_by_chain(&chain) {
                addresses.insert(deployment.spoke_pool.to_string());
                addresses.insert(deployment.multicall_handler().to_string());
            }
        }
        addresses.into_iter().collect()
    }

    pub fn asset_mappings() -> Vec<AssetMapping> {
        vec![
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.000075"),
                    cutoff: EtherConv::parse_ether("0.3"),
                    decimals: 18,
                },
                price: Some(ETHER_PRICE),
                set: HashSet::from_iter([
                    ARBITRUM_WETH_ASSET_ID.clone(),
                    BASE_WETH_ASSET_ID.clone(),
                    ETHEREUM_WETH_ASSET_ID.clone(),
                    LINEA_WETH_ASSET_ID.clone(),
                    OPTIMISM_WETH_ASSET_ID.clone(),
                    POLYGON_WETH_ASSET_ID.clone(),
                    ZKSYNC_WETH_ASSET_ID.clone(),
                    WORLD_WETH_ASSET_ID.clone(),
                    INK_WETH_ASSET_ID.clone(),
                    UNICHAIN_WETH_ASSET_ID.clone(),
                    SMARTCHAIN_ETH_ASSET_ID.clone(),
                    ROBINHOOD_WETH_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 6,
                },
                price: Some(TokenPrice::Dollar),
                set: HashSet::from_iter([
                    ARBITRUM_USDC_ASSET_ID.clone(),
                    AVALANCHE_USDC_ASSET_ID.clone(),
                    BASE_USDC_ASSET_ID.clone(),
                    ETHEREUM_USDC_ASSET_ID.clone(),
                    OPTIMISM_USDC_ASSET_ID.clone(),
                    POLYGON_USDC_ASSET_ID.clone(),
                    UNICHAIN_USDC_ASSET_ID.clone(),
                    HYPEREVM_USDC_ASSET_ID.clone(),
                    MONAD_USDC_ASSET_ID.clone(),
                    INK_USDC_ASSET_ID.clone(),
                    ROBINHOOD_USDG_ASSET_ID.clone(),
                    ARC_USDC_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 18,
                },
                price: Some(TokenPrice::Dollar),
                set: HashSet::from_iter([ETHEREUM_USDC_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 6,
                },
                price: Some(TokenPrice::Dollar),
                set: HashSet::from_iter([
                    ARBITRUM_USDT_ASSET_ID.clone(),
                    AVALANCHE_USDT_ASSET_ID.clone(),
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    LINEA_USDT_ASSET_ID.clone(),
                    OPTIMISM_USDT_ASSET_ID.clone(),
                    POLYGON_USDT_ASSET_ID.clone(),
                    ZKSYNC_USDT_ASSET_ID.clone(),
                    INK_USDT0_ASSET_ID.clone(),
                    HYPEREVM_USDT_ASSET_ID.clone(),
                    PLASMA_USDT_ASSET_ID.clone(),
                    MONAD_USDT_ASSET_ID.clone(),
                    UNICHAIN_USDT0_ASSET_ID.clone(),
                    TRON_USDT_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 18,
                },
                price: Some(TokenPrice::Dollar),
                set: HashSet::from_iter([ETHEREUM_USDT_ASSET_ID.clone(), SMARTCHAIN_USDT_ASSET_ID.clone()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 6,
                },
                price: Some(TokenPrice::Dollar),
                set: HashSet::from_iter([
                    ARBITRUM_USDC_E_ASSET_ID.clone(),
                    BASE_USDC_E_ASSET_ID.clone(),
                    ETHEREUM_USDC_E_ASSET_ID.clone(),
                    LINEA_USDC_E_ASSET_ID.clone(),
                    OPTIMISM_USDC_E_ASSET_ID.clone(),
                    POLYGON_USDC_E_ASSET_ID.clone(),
                    WORLD_USDC_E_ASSET_ID.clone(),
                    ZKSYNC_USDC_E_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0003"),
                    upper_bound: EtherConv::parse_ether("0.0025"),
                    cutoff: EtherConv::parse_ether("10"),
                    decimals: 8,
                },
                price: None,
                set: HashSet::from_iter([
                    ARBITRUM_WBTC_ASSET_ID.clone(),
                    ETHEREUM_WBTC_ASSET_ID.clone(),
                    LINEA_WBTC_ASSET_ID.clone(),
                    OPTIMISM_WBTC_ASSET_ID.clone(),
                    POLYGON_WBTC_ASSET_ID.clone(),
                    WORLD_WBTC_ASSET_ID.clone(),
                    ZKSYNC_WBTC_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.001"),
                    cutoff: EtherConv::parse_ether("1000000"),
                    decimals: 18,
                },
                price: None,
                set: HashSet::from_iter([ARBITRUM_ACX_ASSET_ID.clone(), ETHEREUM_ACX_ASSET_ID.clone(), OPTIMISM_ACX_ASSET_ID.clone(), POLYGON_ACX_ASSET_ID.clone()]),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::AcrossDeployment;
    use crate::across::asset::AcrossAsset;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_USDC_TOKEN_ID, LINEA_USDC_E_ASSET_ID, LINEA_USDC_E_TOKEN_ID, ROBINHOOD_USDG_ASSET_ID, ROBINHOOD_WETH_ASSET_ID},
        contract_constants::{ARC_ACROSS_MULTICALL_HANDLER_CONTRACT, ARC_ACROSS_SPOKE_POOL_CONTRACT, ROBINHOOD_ACROSS_MULTICALL_HANDLER_CONTRACT, ROBINHOOD_ACROSS_SPOKE_POOL_CONTRACT},
    };

    #[test]
    fn test_robinhood_deployment() {
        let deployment = AcrossDeployment::deployment_by_chain(&Chain::Robinhood).unwrap();
        assert_eq!(deployment.chain_id, 4663);
        assert_eq!(deployment.spoke_pool, ROBINHOOD_ACROSS_SPOKE_POOL_CONTRACT);
        assert_eq!(deployment.multicall_handler(), ROBINHOOD_ACROSS_MULTICALL_HANDLER_CONTRACT);
        assert_eq!(AcrossDeployment::supported_assets().get(&Chain::Robinhood), Some(&vec![ROBINHOOD_WETH_ASSET_ID.clone(), ROBINHOOD_USDG_ASSET_ID.clone()]));
    }

    #[test]
    fn test_fill_cost_covers_every_destination() {
        for chain in Chain::all().into_iter().filter(|chain| AcrossDeployment::deployment_by_chain(chain).is_some()) {
            assert_eq!(AcrossDeployment::fill_cost(&chain).is_some(), chain != Chain::Tron, "{chain:?}");
        }
    }

    #[test]
    fn test_routed_assets_are_priced() {
        let mappings = AcrossDeployment::asset_mappings();
        let mapping = |asset: &AssetId| mappings.iter().find(|mapping| mapping.set.contains(asset));
        let routed_natives = Chain::all().into_iter().filter_map(|chain| AcrossAsset::from_asset(&chain.as_asset_id())).map(|asset| asset.asset_id);

        for asset in AcrossDeployment::supported_assets().into_values().flatten() {
            assert!(mapping(&asset).and_then(|mapping| mapping.price).is_some(), "{asset}");
        }
        for asset in routed_natives.filter(|asset| mapping(asset).is_some()) {
            assert!(mapping(&asset).and_then(|mapping| mapping.price).is_some(), "{asset}");
        }
    }

    #[test]
    fn test_arc_deployment() {
        let deployment = AcrossDeployment::deployment_by_chain(&Chain::Arc).unwrap();
        assert_eq!(deployment.chain_id, 5042);
        assert_eq!(deployment.spoke_pool, ARC_ACROSS_SPOKE_POOL_CONTRACT);
        assert_eq!(deployment.multicall_handler(), ARC_ACROSS_MULTICALL_HANDLER_CONTRACT);
        assert_eq!(AcrossDeployment::supported_assets().get(&Chain::Arc), Some(&vec![]));
        assert_eq!(AcrossDeployment::supported_asset_for_token(Chain::Arc, ARC_USDC_TOKEN_ID.parse().unwrap()), Some(Chain::Arc.as_asset_id()));
        assert_eq!(AcrossDeployment::supported_asset_for_token(Chain::Linea, LINEA_USDC_E_TOKEN_ID.parse().unwrap()), Some(LINEA_USDC_E_ASSET_ID.clone()));
    }
}
