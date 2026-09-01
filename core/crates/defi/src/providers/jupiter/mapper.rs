use std::collections::HashMap;
use std::error::Error;

use ::jupiter::{PortfolioAsset, PortfolioElement, PositionsResponse, TokenInfo};
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, Chain, DefiPosition, DefiPositionAsset, DefiPositionMetadata, DefiPositionType, DefiProtocol, DefiProvider, contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS,
};

type TokenInfoByAddress = HashMap<String, TokenInfo>;

pub fn map_positions(response: PositionsResponse) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
    if response.elements.is_empty() {
        return Ok(Vec::new());
    }

    let token_info = response
        .token_info
        .as_ref()
        .and_then(|token_info| token_info.get(Chain::Solana.as_ref()))
        .ok_or("Missing Jupiter Solana token info")?;

    response.elements.into_iter().enumerate().try_fold(Vec::new(), |mut positions, (index, element)| {
        positions.extend(map_element_positions(element, token_info, index)?);
        Ok(positions)
    })
}

fn map_element_positions(element: PortfolioElement, token_info: &TokenInfoByAddress, index: usize) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
    let mut positions = Vec::new();

    match element {
        PortfolioElement::Multiple(element) => {
            let assets = map_assets(element.data.assets.iter(), token_info)?;
            let position_type = map_position_type(&element.label);
            push_position(
                &mut positions,
                position_id(&element.platform_id, &element.label, element.name.as_deref(), element.data.reference.as_deref(), index),
                &element.platform_id,
                element.name.unwrap_or(element.label),
                position_type,
                assets,
            );
        }
        PortfolioElement::Liquidity(element) => {
            for (liquidity_index, liquidity) in element.data.liquidities.iter().enumerate() {
                let assets = map_assets(liquidity.assets.iter().chain(liquidity.reward_assets.iter()), token_info)?;
                let name = liquidity.name.clone().or_else(|| element.name.clone()).unwrap_or_else(|| element.label.clone());
                push_position(
                    &mut positions,
                    position_id(&element.platform_id, &element.label, Some(&name), liquidity.reference.as_deref(), index + liquidity_index),
                    &element.platform_id,
                    name,
                    DefiPositionType::LiquidityPool,
                    assets,
                );
            }
        }
        PortfolioElement::BorrowLend(element) => {
            let unsettled_assets = element.data.unsettled.as_ref().map(|unsettled| unsettled.assets.as_slice()).unwrap_or_default();
            let assets = map_assets(
                element
                    .data
                    .supplied_assets
                    .iter()
                    .chain(element.data.borrowed_assets.iter())
                    .chain(element.data.reward_assets.iter())
                    .chain(unsettled_assets.iter()),
                token_info,
            )?;
            push_position(
                &mut positions,
                position_id(&element.platform_id, &element.label, element.name.as_deref(), element.data.reference.as_deref(), index),
                &element.platform_id,
                element.name.unwrap_or(element.label),
                DefiPositionType::Lending,
                assets,
            );
        }
        PortfolioElement::Trade(element) => {
            let assets = [element.data.assets.input.as_ref(), element.data.assets.output.as_ref()]
                .into_iter()
                .flatten()
                .map(|asset| map_asset(asset, token_info))
                .collect::<Result<Vec<_>, _>>()?;
            push_position(
                &mut positions,
                position_id(&element.platform_id, &element.label, element.name.as_deref(), element.data.reference.as_deref(), index),
                &element.platform_id,
                element.name.unwrap_or(element.label),
                DefiPositionType::Other,
                assets,
            );
        }
        PortfolioElement::Leverage(element) => {
            let assets = match element.data.cross.as_ref().and_then(|cross| cross.collateral_assets.as_ref()) {
                Some(collateral_assets) => map_assets(collateral_assets.iter(), token_info)?,
                None => Vec::new(),
            };
            push_position(
                &mut positions,
                position_id(&element.platform_id, &element.label, element.name.as_deref(), element.data.reference.as_deref(), index),
                &element.platform_id,
                element.name.unwrap_or(element.label),
                DefiPositionType::Other,
                assets,
            );
        }
    }

    Ok(positions)
}

fn map_assets<'a>(assets: impl Iterator<Item = &'a PortfolioAsset>, token_info: &TokenInfoByAddress) -> Result<Vec<DefiPositionAsset>, Box<dyn Error + Send + Sync>> {
    assets.map(|asset| map_asset(asset, token_info)).collect()
}

fn map_asset(asset: &PortfolioAsset, token_info: &TokenInfoByAddress) -> Result<DefiPositionAsset, Box<dyn Error + Send + Sync>> {
    let address = asset.data.address.as_ref().ok_or("Missing Jupiter asset address")?;
    let amount = asset.data.amount.as_ref().ok_or("Missing Jupiter asset amount")?;
    let decimals = token_info
        .get(address)
        .map(|token_info| token_info.decimals)
        .ok_or_else(|| format!("Missing Jupiter token info for asset {address}"))?;

    Ok(DefiPositionAsset {
        asset_id: map_asset_id(address),
        value: BigNumberFormatter::value_from_amount_biguint(&amount.to_string(), decimals)?,
    })
}

fn map_asset_id(address: &str) -> AssetId {
    match address {
        SOLANA_WRAPPED_SOL_TOKEN_ADDRESS => AssetId::from_chain(Chain::Solana),
        _ => AssetId::from_token(Chain::Solana, address),
    }
}

fn push_position(positions: &mut Vec<DefiPosition>, id: String, platform_id: &str, name: String, position_type: DefiPositionType, assets: Vec<DefiPositionAsset>) {
    if assets.is_empty() {
        return;
    }

    positions.push(DefiPosition {
        id,
        provider: DefiProvider::Jupiter,
        chain: Chain::Solana,
        protocol_info: DefiProtocol {
            name: platform_id.to_string(),
            url: None,
        },
        name,
        position_type,
        metadata: DefiPositionMetadata { apy: None },
        assets,
    });
}

fn position_id(platform_id: &str, label: &str, name: Option<&str>, reference: Option<&str>, index: usize) -> String {
    let value = reference
        .or(name)
        .map(|value| format!("{platform_id}-{label}-{value}"))
        .unwrap_or_else(|| format!("{platform_id}-{label}-{index}"));
    value.replace(' ', "-").to_lowercase()
}

fn map_position_type(label: &str) -> DefiPositionType {
    match label {
        "Staked" | "Farming" | "Vault" => DefiPositionType::Staking,
        "LiquidityPool" => DefiPositionType::LiquidityPool,
        "Lending" => DefiPositionType::Lending,
        "Rewards" => DefiPositionType::Rewards,
        _ => DefiPositionType::Other,
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain, DefiPositionType, DefiProvider, asset_constants::SOLANA_USDC_TOKEN_ID};

    use super::{PositionsResponse, map_positions};

    #[test]
    fn test_map_positions() {
        let response: PositionsResponse = serde_json::from_str(include_str!("../../../testdata/jupiter/positions.json")).unwrap();

        let positions = map_positions(response).unwrap();

        assert_eq!(positions.len(), 4);
        assert_eq!(positions[0].id, "jupiter-governance-staked-stake-account");
        assert_eq!(positions[0].provider, DefiProvider::Jupiter);
        assert_eq!(positions[0].chain, Chain::Solana);
        assert_eq!(positions[0].protocol_info.name, "jupiter-governance");
        assert_eq!(positions[0].name, "JUP Staked");
        assert_eq!(positions[0].position_type, DefiPositionType::Staking);
        assert_eq!(
            positions[0].assets[0].asset_id,
            AssetId::from_token(Chain::Solana, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN")
        );
        assert_eq!(positions[0].assets[0].value, BigUint::from(1250000000u64));

        assert_eq!(positions[1].id, "jupiter-exchange-liquiditypool-liquidity-position");
        assert_eq!(positions[1].position_type, DefiPositionType::LiquidityPool);
        assert_eq!(positions[1].assets.len(), 2);
        assert_eq!(positions[1].assets[0].asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(positions[1].assets[0].value, BigUint::from(2500000000u64));
        assert_eq!(positions[1].assets[1].asset_id, AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID));
        assert_eq!(positions[1].assets[1].value, BigUint::from(1500750u64));

        assert_eq!(positions[2].id, "jupiter-exchange-lending-lending-position");
        assert_eq!(positions[2].position_type, DefiPositionType::Lending);
        assert_eq!(positions[2].assets.len(), 3);

        assert_eq!(positions[3].id, "jupiter-exchange-limitorder-order-account");
        assert_eq!(positions[3].position_type, DefiPositionType::Other);
        assert_eq!(positions[3].assets.len(), 2);
    }

    #[test]
    fn test_map_positions_empty() {
        let response: PositionsResponse = serde_json::from_str(r#"{"elements":[],"tokenInfo":{}}"#).unwrap();

        assert!(map_positions(response).unwrap().is_empty());
    }
}
