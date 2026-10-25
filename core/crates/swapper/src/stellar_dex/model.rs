use gem_stellar::{
    constants::STELLAR_DECIMALS,
    models::{StellarPaymentPath, StellarSwapData},
    provider::balances_mapper::map_asset_id,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, Chain, TransactionSwapReferralFee};
use serde::{Deserialize, Serialize};

use crate::{
    QuoteRequest, SwapperError,
    fees::{FeeSide, ReferralFee, ReferralTransfer},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct StellarRoute {
    pub path: Vec<AssetId>,
    pub swap_value: u64,
    pub output_value: u64,
    pub fee: ReferralTransfer,
}

impl StellarRoute {
    pub fn best(paths: &[StellarPaymentPath], value: u64, referral_fee: &ReferralFee, fee_side: FeeSide) -> Result<Self, SwapperError> {
        let (output_value, path) = paths.iter().filter_map(path_output).max_by_key(|(output_value, _)| *output_value).ok_or(SwapperError::NoQuoteAvailable)?;
        let route = Self {
            path,
            swap_value: referral_fee.swap_value(fee_side, value),
            output_value,
            fee: referral_fee.transfer(fee_side, value, output_value),
        };
        match route.received_value() {
            0 => Err(SwapperError::NoQuoteAvailable),
            _ => Ok(route),
        }
    }

    pub fn received_value(&self) -> u64 {
        self.fee.received_value(self.output_value)
    }

    pub fn swap_data(&self, request: &QuoteRequest, change_trust: bool, max_time: u64) -> StellarSwapData {
        let fee_asset = match self.fee.side {
            FeeSide::Input => request.from_asset.asset_id(),
            FeeSide::Output => request.to_asset.asset_id(),
        };
        StellarSwapData {
            send_amount: self.swap_value,
            destination_min: self.fee.minimum_output(self.output_value, request.options.slippage.bps),
            path: self.path.clone(),
            change_trust,
            fee: (self.fee.value > 0).then(|| TransactionSwapReferralFee {
                asset_id: fee_asset,
                value: BigUint::from(self.fee.value),
            }),
            max_time,
        }
    }
}

fn path_output(path: &StellarPaymentPath) -> Option<(u64, Vec<AssetId>)> {
    let output_value = BigNumberFormatter::value_from_amount_exact(&path.destination_amount, STELLAR_DECIMALS).ok()?.to_u64()?;
    let assets = path
        .path
        .iter()
        .map(|asset| map_asset_id(Chain::Stellar, &asset.asset_type, asset.asset_issuer.as_ref(), asset.asset_code.as_ref()))
        .collect::<Option<Vec<_>>>()?;
    Some((output_value, assets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stellar_dex::testkit::{eurc, mock_request};
    use gem_stellar::models::StellarPathAsset;
    use primitives::swap::STELLAR_REFERRAL_ADDRESS;

    fn path(destination_amount: &str, asset_type: &str) -> StellarPaymentPath {
        StellarPaymentPath {
            destination_amount: destination_amount.to_string(),
            path: vec![StellarPathAsset {
                asset_type: asset_type.to_string(),
                asset_code: None,
                asset_issuer: None,
            }],
        }
    }

    #[test]
    fn test_best_route_needs_a_path_that_receives_something() {
        let referral_fee = ReferralFee {
            address: STELLAR_REFERRAL_ADDRESS.to_string(),
            bps: 50,
        };
        let best = |paths: &[StellarPaymentPath]| StellarRoute::best(paths, 1_000, &referral_fee, FeeSide::Input);

        assert_eq!(best(&[]), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(best(&[path("0.0000000", "native")]), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(best(&[path("9999.0000000", "liquidity_pool_shares")]), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_swap_data_keeps_the_minimum_received_after_an_output_fee() {
        let xlm = AssetId::from_chain(Chain::Stellar);
        let request = mock_request(eurc(), xlm.clone(), 1_000_000_000);
        let buying_xlm = StellarRoute {
            path: vec![],
            swap_value: 1_000_000_000,
            output_value: 5_838_000_000,
            fee: ReferralTransfer { side: FeeSide::Output, value: 29_190_000 },
        };
        let dust = StellarRoute {
            fee: ReferralTransfer { side: FeeSide::Output, value: 0 },
            ..buying_xlm.clone()
        };

        let data = buying_xlm.swap_data(&request, false, 0);

        assert_eq!(
            data.fee,
            Some(TransactionSwapReferralFee {
                asset_id: xlm,
                value: BigUint::from(29_190_000u32)
            })
        );
        assert_eq!(data.destination_min - 29_190_000, 5_750_721_900, "what stays after Gem's fee is the minimum shown: 5,808,810,000 less 1%");
        assert_eq!(dust.swap_data(&request, false, 0).fee, None, "a zero payment is malformed on Stellar, so a dust swap pays no fee");
    }
}
