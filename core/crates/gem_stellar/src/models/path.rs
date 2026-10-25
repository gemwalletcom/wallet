use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::AssetId;
use serde::{Deserialize, Serialize};

use crate::constants::{ASSET_TYPE_CREDIT_ALPHANUM4, ASSET_TYPE_CREDIT_ALPHANUM12, ASSET_TYPE_NATIVE, STELLAR_DECIMALS};

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
            Some((issuer, code)) if code.len() <= 4 => (ASSET_TYPE_CREDIT_ALPHANUM4, Some(code), Some(issuer)),
            Some((issuer, code)) => (ASSET_TYPE_CREDIT_ALPHANUM12, Some(code), Some(issuer)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, asset_constants::STELLAR_USDC_ASSET_ID};

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
}
