use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};

use crate::constants::STELLAR_DECIMALS;

const ASSET_CODE_ALPHANUM4_MAX_LENGTH: usize = 4;
const ASSET_TYPE_NATIVE: &str = "native";
const ASSET_TYPE_ALPHANUM4: &str = "credit_alphanum4";
const ASSET_TYPE_ALPHANUM12: &str = "credit_alphanum12";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StrictSendPathsQuery {
    pub source_asset_type: &'static str,
    pub source_asset_code: Option<String>,
    pub source_asset_issuer: Option<String>,
    pub source_amount: String,
    pub destination_assets: String,
}

impl StrictSendPathsQuery {
    pub fn new(source: &AssetId, source_amount: u64, destination: &AssetId) -> Result<Self, NumberFormatterError> {
        let (source_asset_type, source_asset_code, source_asset_issuer) = match source.split_sub_token_parts() {
            Some((issuer, code)) if code.len() <= ASSET_CODE_ALPHANUM4_MAX_LENGTH => (ASSET_TYPE_ALPHANUM4, Some(code), Some(issuer)),
            Some((issuer, code)) => (ASSET_TYPE_ALPHANUM12, Some(code), Some(issuer)),
            None => (ASSET_TYPE_NATIVE, None, None),
        };
        Ok(Self {
            source_asset_type,
            source_asset_code,
            source_asset_issuer,
            source_amount: BigNumberFormatter::plain_value(source_amount, STELLAR_DECIMALS)?,
            destination_assets: destination.contract().unwrap_or_else(|| ASSET_TYPE_NATIVE.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarPaymentPath {
    pub destination_amount: String,
    pub path: Vec<StellarPathAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarPathAsset {
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

impl StellarPathAsset {
    pub fn asset_id(&self) -> Option<AssetId> {
        stellar_asset_id(&self.asset_type, self.asset_code.as_ref(), self.asset_issuer.as_ref())
    }
}

pub(crate) fn stellar_asset_id(asset_type: &str, code: Option<&String>, issuer: Option<&String>) -> Option<AssetId> {
    match (asset_type, code, issuer) {
        (ASSET_TYPE_NATIVE, None, None) => Some(AssetId::from_chain(Chain::Stellar)),
        (ASSET_TYPE_ALPHANUM4 | ASSET_TYPE_ALPHANUM12, Some(code), Some(issuer)) => Some(AssetId::from_token(Chain::Stellar, &AssetId::sub_token_id(&[issuer.clone(), code.clone()]))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::STELLAR_USDC_ASSET_ID;

    #[test]
    fn test_strict_send_paths_query() {
        let xlm = AssetId::from_chain(Chain::Stellar);
        let long_code = AssetId::from_token(Chain::Stellar, "GCRYUGD5NVARGXT56XEZI5CIFCQETYHAPQQTHO2O3IQZTHDH4LATMYWC::CETES");

        assert_eq!(
            StrictSendPathsQuery::new(&xlm, 58_380_000_001, &STELLAR_USDC_ASSET_ID).unwrap(),
            StrictSendPathsQuery {
                source_asset_type: "native",
                source_asset_code: None,
                source_asset_issuer: None,
                source_amount: "5838.0000001".to_string(),
                destination_assets: "USDC:GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string(),
            }
        );
        assert_eq!(
            StrictSendPathsQuery::new(&STELLAR_USDC_ASSET_ID, 1, &xlm).unwrap(),
            StrictSendPathsQuery {
                source_asset_type: "credit_alphanum4",
                source_asset_code: Some("USDC".to_string()),
                source_asset_issuer: Some("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string()),
                source_amount: "0.0000001".to_string(),
                destination_assets: "native".to_string(),
            }
        );
        assert_eq!(StrictSendPathsQuery::new(&long_code, 10_000_000, &xlm).unwrap().source_asset_type, "credit_alphanum12");
    }

    #[test]
    fn test_path_asset_id() {
        let asset = |asset_type: &str, code: Option<&str>, issuer: Option<&str>| StellarPathAsset {
            asset_type: asset_type.to_string(),
            asset_code: code.map(str::to_string),
            asset_issuer: issuer.map(str::to_string),
        };
        let issuer = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";

        assert_eq!(asset("native", None, None).asset_id(), Some(AssetId::from_chain(Chain::Stellar)));
        assert_eq!(asset("credit_alphanum4", Some("USDC"), Some(issuer)).asset_id(), Some(STELLAR_USDC_ASSET_ID.clone()));
        assert_eq!(asset("liquidity_pool_shares", None, None).asset_id(), None);
        assert_eq!(asset("credit_alphanum4", Some("USDC"), None).asset_id(), None);
    }
}
