use serde::Deserialize;
use serde_json::Number;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionsResponse {
    pub elements: Vec<PortfolioElement>,
    pub token_info: Option<HashMap<String, HashMap<String, TokenInfo>>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PortfolioElement {
    #[serde(rename = "multiple")]
    Multiple(MultipleElement),
    #[serde(rename = "liquidity")]
    Liquidity(LiquidityElement),
    #[serde(rename = "borrowlend")]
    BorrowLend(BorrowLendElement),
    #[serde(rename = "trade")]
    Trade(TradeElement),
    #[serde(rename = "leverage")]
    Leverage(LeverageElement),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleElement {
    pub platform_id: String,
    pub label: String,
    pub name: Option<String>,
    pub data: MultipleData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleData {
    pub assets: Vec<PortfolioAsset>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityElement {
    pub platform_id: String,
    pub label: String,
    pub name: Option<String>,
    pub data: LiquidityData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityData {
    pub liquidities: Vec<Liquidity>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Liquidity {
    pub assets: Vec<PortfolioAsset>,
    pub reward_assets: Vec<PortfolioAsset>,
    pub name: Option<String>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowLendElement {
    pub platform_id: String,
    pub label: String,
    pub name: Option<String>,
    pub data: BorrowLendData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowLendData {
    pub supplied_assets: Vec<PortfolioAsset>,
    pub borrowed_assets: Vec<PortfolioAsset>,
    pub reward_assets: Vec<PortfolioAsset>,
    pub unsettled: Option<UnsettledAssets>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnsettledAssets {
    pub assets: Vec<PortfolioAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeElement {
    pub platform_id: String,
    pub label: String,
    pub name: Option<String>,
    pub data: TradeData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeData {
    pub assets: TradeAssets,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TradeAssets {
    pub input: Option<PortfolioAsset>,
    pub output: Option<PortfolioAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageElement {
    pub platform_id: String,
    pub label: String,
    pub name: Option<String>,
    pub data: LeverageData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageData {
    pub cross: Option<CrossLeverage>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossLeverage {
    pub collateral_assets: Option<Vec<PortfolioAsset>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAsset {
    pub data: PortfolioAssetData,
}

#[derive(Debug, Deserialize)]
pub struct PortfolioAssetData {
    pub address: Option<String>,
    pub amount: Option<Number>,
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    pub decimals: u32,
}
