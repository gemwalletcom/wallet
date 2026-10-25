use gem_xrp::{
    currency::truncate_to_iou_precision,
    models::{Amount, PathAlternative, XrpSwap},
};
use num_bigint::BigUint;
use primitives::TransactionSwapReferralFee;

use crate::{
    QuoteRequest, SwapperError, SwapperQuoteAsset,
    fees::{FeeSide, ReferralFee, ReferralTransfer},
};

pub(super) fn ledger_value(asset: &SwapperQuoteAsset, value: &BigUint) -> BigUint {
    match asset.is_native() {
        true => value.clone(),
        false => truncate_to_iou_precision(value),
    }
}

pub(super) fn send_max(request: &QuoteRequest, fee_side: FeeSide, referral_fee: &ReferralFee) -> BigUint {
    ledger_value(&request.from_asset, &referral_fee.swap_value(fee_side, request.value.clone()))
}

pub(super) fn path_destination(request: &QuoteRequest) -> String {
    request.to_asset.asset_id().token_id.unwrap_or_else(|| request.wallet_address.clone())
}

pub(super) fn map_swap(request: &QuoteRequest, fee_side: FeeSide, referral_fee: &ReferralFee, send_max: BigUint, alternatives: &[PathAlternative]) -> Result<(BigUint, XrpSwap), SwapperError> {
    let (amount, alternative) = alternatives
        .iter()
        .filter(|alternative| alternative.source_amount.value().as_ref() == Some(&send_max))
        .filter_map(|alternative| Some((alternative.destination_amount.as_ref().and_then(Amount::value)?, alternative)))
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .ok_or(SwapperError::NoQuoteAvailable)?;
    let (fee_asset, transfer) = match fee_side {
        FeeSide::Input => (
            &request.from_asset,
            ReferralTransfer {
                side: fee_side,
                value: &request.value - &send_max,
            },
        ),
        FeeSide::Output => (&request.to_asset, referral_fee.transfer(fee_side, request.value.clone(), amount.clone())),
    };
    let swap = XrpSwap {
        send_max,
        amount: amount.clone(),
        deliver_min: ledger_value(&request.to_asset, &transfer.minimum_output(amount.clone(), request.options.slippage.bps)),
        paths: alternative.paths_computed.clone(),
        fee: (transfer.value > BigUint::ZERO).then(|| TransactionSwapReferralFee {
            asset_id: fee_asset.asset_id(),
            value: transfer.value.clone(),
        }),
        trust_line: false,
    };
    Ok((transfer.received_value(amount), swap))
}

#[cfg(test)]
mod tests {
    use gem_xrp::models::AmountCurrency;
    use primitives::{Asset, Chain, asset_constants::XRP_USDC_TOKEN_ID};

    use super::*;
    use crate::{fees::default_referral_fees, xrp_ledger::testkit::mock_request};

    #[test]
    fn test_ledger_value() {
        let (xrp, rlusd) = (SwapperQuoteAsset::from(&Asset::from_chain(Chain::Xrp)), SwapperQuoteAsset::from(&Asset::mock_xrp_rlusd()));
        let value = BigUint::from(12_345_678_901_234_567u64);

        assert_eq!(ledger_value(&xrp, &value), value);
        assert_eq!(ledger_value(&rlusd, &value), BigUint::from(12_345_678_901_234_560u64));
    }

    #[test]
    fn test_map_swap() {
        let request = mock_request(&Asset::mock_xrp_usdc(), &Asset::from_chain(Chain::Xrp), 1_000_000_000_000_000_000);
        let referral_fee = default_referral_fees().xrp;
        let send_max = send_max(&request, FeeSide::Output, &referral_fee);
        let alternative = |source: u64, destination: u64| PathAlternative {
            source_amount: Amount::Amount(AmountCurrency {
                value: BigUint::from(source),
                issuer: Some(XRP_USDC_TOKEN_ID.to_string()),
                currency: Some("5553444300000000000000000000000000000000".to_string()),
                mpt_issuance_id: None,
            }),
            destination_amount: Some(Amount::Str(BigUint::from(destination))),
            paths_computed: vec![],
        };
        let whole_input = 1_000_000_000_000_000_000;
        let map = |alternatives: &[PathAlternative]| map_swap(&request, FeeSide::Output, &referral_fee, send_max.clone(), alternatives).map(|(to_value, swap)| (to_value, swap.amount, swap.fee.map(|fee| fee.value)));

        assert_eq!(
            map(&[alternative(999_000_000_000_000_000, 800_000_000), alternative(whole_input, 750_000_000), alternative(whole_input, 760_000_000)]),
            Ok((BigUint::from(756_200_000u64), BigUint::from(760_000_000u64), Some(BigUint::from(3_800_000u64))))
        );
        assert_eq!(map(&[alternative(999_000_000_000_000_000, 800_000_000)]), Err(SwapperError::NoQuoteAvailable));
        assert_eq!(map(&[alternative(whole_input, 199)]), Ok((BigUint::from(199u64), BigUint::from(199u64), None)));
    }
}
