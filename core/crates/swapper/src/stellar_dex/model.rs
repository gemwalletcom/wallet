use gem_stellar::{
    constants::STELLAR_DECIMALS,
    models::{StellarPathAsset, StellarPaymentPath, StellarSwapData, StellarSwapFee},
};
use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, swap::STELLAR_REFERRAL_TRUSTED_ASSETS};
use serde::{Deserialize, Serialize};

use crate::{
    Quote, SwapperError,
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

    pub fn swap_data(&self, quote: &Quote, change_trust: bool, max_time: u64) -> StellarSwapData {
        let fee_asset = match self.fee.side {
            FeeSide::Input => quote.request.from_asset.asset_id(),
            FeeSide::Output => quote.request.to_asset.asset_id(),
        };
        StellarSwapData {
            send_amount: self.swap_value,
            destination_min: self.fee.minimum_output(self.output_value, quote.data.slippage_bps),
            path: self.path.clone(),
            change_trust,
            fee: (self.fee.value > 0).then(|| StellarSwapFee {
                destination: quote.request.on_chain_referral_fee().address,
                asset: fee_asset,
                amount: self.fee.value,
            }),
            max_time,
        }
    }
}

pub(super) fn fee_side(from_asset: &AssetId, to_asset: &AssetId) -> Option<FeeSide> {
    FeeSide::from_available(from_asset.is_native() || STELLAR_REFERRAL_TRUSTED_ASSETS.contains(from_asset), to_asset.is_native())
}

fn path_output(path: &StellarPaymentPath) -> Option<(u64, Vec<AssetId>)> {
    let output_value = BigNumberFormatter::value_from_amount_exact(&path.destination_amount, STELLAR_DECIMALS).ok()?.to_u64()?;
    let assets = path.path.iter().map(StellarPathAsset::asset_id).collect::<Option<Vec<_>>>()?;
    Some((output_value, assets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderType, QuoteRequest, SwapperProvider, SwapperQuoteAsset, models::Options};
    use num_bigint::BigUint;
    use primitives::{Chain, asset_constants::STELLAR_USDC_ASSET_ID, swap::STELLAR_REFERRAL_ADDRESS};

    const EURC_TOKEN_ID: &str = "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC";

    fn referral_fee() -> ReferralFee {
        ReferralFee {
            address: STELLAR_REFERRAL_ADDRESS.to_string(),
            bps: 50,
        }
    }

    fn path(destination_amount: &str, path: Vec<StellarPathAsset>) -> StellarPaymentPath {
        StellarPaymentPath {
            destination_amount: destination_amount.to_string(),
            path,
        }
    }

    fn quote(from_asset: AssetId, to_asset: AssetId, route: &StellarRoute) -> Quote {
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(from_asset),
            to_asset: SwapperQuoteAsset::from(to_asset),
            wallet_address: String::new(),
            destination_address: String::new(),
            value: BigUint::from(route.fee.input_value(route.swap_value)),
            options: Options::new_with_slippage(100.into()),
        };
        Quote::from_route(&request, &ProviderType::new(SwapperProvider::StellarDex), BigUint::from(route.received_value()), 100, route, None).unwrap()
    }

    #[test]
    fn test_fee_is_taken_in_xlm_or_in_usdc_sold() {
        let xlm = AssetId::from_chain(Chain::Stellar);
        let usdc = STELLAR_USDC_ASSET_ID.clone();
        let eurc = AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID);
        let named_usdc = AssetId::from_token(Chain::Stellar, "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::USDC");

        assert_eq!(fee_side(&xlm, &usdc), Some(FeeSide::Input));
        assert_eq!(fee_side(&xlm, &eurc), Some(FeeSide::Input));
        assert_eq!(fee_side(&usdc, &xlm), Some(FeeSide::Input));
        assert_eq!(fee_side(&usdc, &eurc), Some(FeeSide::Input));
        assert_eq!(fee_side(&eurc, &xlm), Some(FeeSide::Output));
        assert_eq!(fee_side(&eurc, &usdc), None, "the fee account trusts only Circle's USDC, and only a sold one is taken");
        assert_eq!(fee_side(&named_usdc, &eurc), None, "a USDC from another issuer is not trusted");
    }

    #[test]
    fn test_best_route_takes_the_path_that_receives_the_most() {
        let usdc = StellarPathAsset {
            asset_type: "credit_alphanum4".to_string(),
            asset_code: Some("USDC".to_string()),
            asset_issuer: Some("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string()),
        };
        let unknown = StellarPathAsset {
            asset_type: "liquidity_pool_shares".to_string(),
            asset_code: None,
            asset_issuer: None,
        };
        let paths = [path("7671.3771345", vec![]), path("7693.2168543", vec![usdc]), path("9999.0000000", vec![unknown]), path("1.00000001", vec![])];

        let route = StellarRoute::best(&paths, 502_512_562_814, &referral_fee(), FeeSide::Input).unwrap();

        assert_eq!(
            route,
            StellarRoute {
                path: vec![STELLAR_USDC_ASSET_ID.clone()],
                swap_value: 500_000_000_000,
                output_value: 76_932_168_543,
                fee: ReferralTransfer { side: FeeSide::Input, value: 2_512_562_814 },
            }
        );
        assert_eq!(route.received_value(), 76_932_168_543);
        assert_eq!(StellarRoute::best(&[], 1_000, &referral_fee(), FeeSide::Input), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(StellarRoute::best(&[path("0.0000000", vec![])], 1_000, &referral_fee(), FeeSide::Input), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_swap_data_pays_the_fee_in_the_asset_of_its_side() {
        let xlm = AssetId::from_chain(Chain::Stellar);
        let eurc = AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID);
        let selling = StellarRoute {
            path: vec![],
            swap_value: 995_000_000,
            output_value: 170_000_000,
            fee: ReferralTransfer { side: FeeSide::Input, value: 5_000_000 },
        };
        let buying_xlm = StellarRoute {
            path: vec![],
            swap_value: 1_000_000_000,
            output_value: 5_838_000_000,
            fee: ReferralTransfer { side: FeeSide::Output, value: 29_190_000 },
        };
        let fee = |asset: &AssetId, amount: u64| {
            Some(StellarSwapFee {
                destination: STELLAR_REFERRAL_ADDRESS.to_string(),
                asset: asset.clone(),
                amount,
            })
        };

        assert_eq!(
            selling.swap_data(&quote(xlm.clone(), STELLAR_USDC_ASSET_ID.clone(), &selling), true, 1_792_902_087),
            StellarSwapData {
                send_amount: 995_000_000,
                destination_min: 168_300_000,
                path: vec![],
                change_trust: true,
                fee: fee(&xlm, 5_000_000),
                max_time: 1_792_902_087,
            }
        );
        assert_eq!(selling.swap_data(&quote(STELLAR_USDC_ASSET_ID.clone(), xlm.clone(), &selling), false, 0).fee, fee(&STELLAR_USDC_ASSET_ID, 5_000_000));

        let buying = buying_xlm.swap_data(&quote(eurc, xlm.clone(), &buying_xlm), false, 0);
        assert_eq!(buying.fee, fee(&xlm, 29_190_000));
        assert_eq!(buying.destination_min, 5_779_911_900);
        assert_eq!(buying.destination_min - 29_190_000, 5_750_721_900, "what is left after Gem's fee is the minimum shown: 5,808,810,000 less 1%");

        let dust = StellarRoute {
            fee: ReferralTransfer { side: FeeSide::Input, value: 0 },
            ..selling
        };
        assert_eq!(
            dust.swap_data(&quote(xlm, STELLAR_USDC_ASSET_ID.clone(), &dust), false, 0).fee,
            None,
            "a zero payment is malformed on Stellar, so a dust swap pays no fee"
        );
    }
}
