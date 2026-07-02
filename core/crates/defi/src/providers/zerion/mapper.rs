use std::error::Error;

use primitives::{AssetId, Chain, DefiPosition, DefiPositionAsset, DefiPositionType, DefiProtocol, DefiProvider};

use super::client::ZerionClient;
use super::model::{ZerionFungibleInfo, ZerionPosition, ZerionPositionAttributes, ZerionPositionsResponse};

pub fn map_positions(response: ZerionPositionsResponse, chain: Chain) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
    response.data.into_iter().try_fold(Vec::new(), |mut positions, x| {
        let id = position_id(&x.id, chain)?;
        let asset = map_position_asset(&x.attributes, chain)?;

        if let Some(position) = positions.iter_mut().find(|position: &&mut DefiPosition| position.id == id) {
            position.assets.push(asset);
            return Ok(positions);
        }

        let protocol_info = map_protocol(&x);
        let position_type = map_position_type(&x.attributes);

        positions.push(DefiPosition {
            id,
            provider: DefiProvider::Zerion,
            chain,
            protocol_info,
            name: x.attributes.name,
            position_type,
            assets: vec![asset],
        });

        Ok(positions)
    })
}

fn position_id(id: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
    let chain_id = ZerionClient::chain_id(&chain)?;
    let marker = format!("-{}-", chain_id);
    let id = id.split_once(&marker).map(|(_, suffix)| format!("{chain_id}-{suffix}")).unwrap_or_else(|| id.to_string());

    Ok(id.replace(' ', "-"))
}

fn map_protocol(position: &ZerionPosition) -> DefiProtocol {
    let metadata = position.attributes.application_metadata.as_ref();
    let name = metadata
        .map(|metadata| metadata.name.clone())
        .or_else(|| position.attributes.protocol.clone())
        .or_else(|| {
            position
                .relationships
                .dapp
                .as_ref()
                .and_then(|relationship| relationship.data.as_ref())
                .map(|data| data.id.clone())
        })
        .unwrap_or_default();
    let url = metadata.and_then(|metadata| metadata.url.clone()).filter(|url| !url.is_empty());

    DefiProtocol { name, url }
}

fn map_position_asset(attributes: &ZerionPositionAttributes, chain: Chain) -> Result<DefiPositionAsset, Box<dyn Error + Send + Sync>> {
    let asset_id = map_asset_id(&attributes.fungible_info, chain)?;

    Ok(DefiPositionAsset {
        asset_id,
        value: attributes.quantity.int.clone(),
    })
}

fn map_position_type(attributes: &ZerionPositionAttributes) -> DefiPositionType {
    let value = attributes.protocol_module.as_deref().or(attributes.position_type.as_deref()).unwrap_or_default();

    match value {
        "lending" | "loan" | "borrowing" | "debt" | "deposit" => DefiPositionType::Lending,
        "staked" | "staking" => DefiPositionType::Staking,
        "liquidity_pool" | "liquidity-pool" | "pool" => DefiPositionType::LiquidityPool,
        "farm" | "farming" => DefiPositionType::Staking,
        "reward" | "rewards" => DefiPositionType::Rewards,
        _ => DefiPositionType::Other,
    }
}

fn map_asset_id(fungible_info: &ZerionFungibleInfo, chain: Chain) -> Result<AssetId, Box<dyn Error + Send + Sync>> {
    let chain_id = ZerionClient::chain_id(&chain)?;
    let implementation = fungible_info
        .implementations
        .iter()
        .find(|implementation| implementation.chain_id == chain_id)
        .ok_or_else(|| format!("Missing Zerion fungible implementation for chain {chain}"))?;

    match implementation.address.as_deref() {
        Some(address) if !is_native_address(address) => Ok(AssetId::from_token(chain, address)),
        _ => Ok(AssetId::from_chain(chain)),
    }
}

fn is_native_address(address: &str) -> bool {
    address.eq_ignore_ascii_case("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, DefiPositionType, DefiProvider};

    use super::{ZerionPositionsResponse, map_positions, position_id};

    #[test]
    fn test_map_positions() {
        let response: ZerionPositionsResponse = serde_json::from_str(include_str!("../../../testdata/zerion/positions.json")).unwrap();

        let positions = map_positions(response, Chain::Polygon).unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].id, "polygon-asset-none-");
        assert_eq!(positions[0].provider, DefiProvider::Zerion);
        assert_eq!(positions[0].chain, Chain::Polygon);
        assert_eq!(positions[0].protocol_info.name, "AAVE");
        assert_eq!(positions[0].protocol_info.url.as_deref(), Some("https://app.aave.com/"));
        assert_eq!(positions[0].name, "Asset");
        assert_eq!(positions[0].position_type, DefiPositionType::Lending);
        assert_eq!(positions[0].assets.len(), 1);
        assert_eq!(
            positions[0].assets[0].asset_id,
            AssetId::from_token(Chain::Polygon, "0x0391d2021f89dc339f60fff84546ea23e337750f")
        );
        assert_eq!(positions[0].assets[0].value, "123456780000000000000");
    }

    #[test]
    fn test_position_id() {
        assert_eq!(
            position_id("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2-ethereum-aave v3 lending-deposit", Chain::Ethereum).unwrap(),
            "ethereum-aave-v3-lending-deposit"
        );
    }

    #[test]
    fn test_map_positions_groups_pool_legs() {
        let response: ZerionPositionsResponse = serde_json::from_str(include_str!("../../../testdata/zerion/positions_pool.json")).unwrap();

        let positions = map_positions(response, Chain::SmartChain).unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].id, "binance-smart-chain-uniswap-v3-usdt/wbnb-pool-(#2419399)-deposit");
        assert_eq!(positions[0].position_type, DefiPositionType::LiquidityPool);
        assert_eq!(positions[0].assets.len(), 2);
        assert_eq!(
            positions[0].assets[0].asset_id,
            AssetId::from_token(Chain::SmartChain, "0x55d398326f99059ff775485246999027b3197955")
        );
        assert_eq!(
            positions[0].assets[1].asset_id,
            AssetId::from_token(Chain::SmartChain, "0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c")
        );
    }
}
