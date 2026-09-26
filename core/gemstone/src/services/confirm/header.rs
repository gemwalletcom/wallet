use std::str::FromStr;

use chrono::Utc;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, AssetPrice, Currency, PerpetualProvider, SimulationResult, TransactionInputType};

use super::model::{GemConfirmLoad, GemConfirmPhase, GemConfirmScreen, GemSimulationValue, GemTransferAmountResult};
use super::rules;
use crate::config::image::GemImage;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::perpetual::GemPerpetual;
use crate::services::assets::icon::asset_icon;
use crate::services::transactions::model::{GemAmountSign, GemTransactionAmount, GemTransactionHeader, GemTransactionHeaderKind};
use crate::services::transactions::rules::header_amount;
use crate::services::transfer::model::GemTransferData;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemConfirmHeader {
    Placeholder { icon: crate::services::assets::icon::GemAssetIcon },
    Reserved { header: GemTransactionHeader },
    Value { value: GemSimulationValue },
    Transaction { header: GemTransactionHeader },
}

pub fn header(transfer: &GemTransferData, requested: Option<&SimulationResult>, load: Option<&GemConfirmLoad>, currency: Currency, screen: &GemConfirmScreen) -> GemConfirmHeader {
    if let Some(value) = load.and_then(|load| load.simulation.simulation.as_ref()).and_then(|simulation| simulation.header.clone()) {
        return GemConfirmHeader::Value { value };
    }
    if let TransactionInputType::TokenApprove { asset, approval_data } = &transfer.input_type {
        return GemConfirmHeader::Value {
            value: GemSimulationValue::new(asset.clone(), rules::approval_value_from(Some(&approval_data.value), approval_data.is_unlimited)),
        };
    }
    if let (TransactionInputType::Generic { .. }, Some(header)) = (&transfer.input_type, requested.and_then(|result| result.valid_header())) {
        return GemConfirmHeader::Placeholder { icon: asset_icon(&header.asset_id) };
    }
    let header = transaction_header(transfer, load, currency);
    if screen.phase == GemConfirmPhase::Loading && is_amountless_payment(transfer) {
        return GemConfirmHeader::Reserved { header };
    }
    GemConfirmHeader::Transaction { header }
}

fn is_amountless_payment(transfer: &GemTransferData) -> bool {
    matches!(transfer.input_type, TransactionInputType::Payment { .. }) && transfer.value == GemBigInt::ZERO
}

fn transaction_header(transfer: &GemTransferData, load: Option<&GemConfirmLoad>, currency: Currency) -> GemTransactionHeader {
    let prices = load.map(|load| load.metadata.prices.as_slice()).unwrap_or_default();
    let amount = |shows_fiat: bool| GemTransactionHeader::Amount {
        amount: header_amount(amount(transfer, load, prices, &currency), &currency, shows_fiat),
    };
    match transfer.header_kind() {
        GemTransactionHeaderKind::Amount { shows_fiat } => amount(shows_fiat),
        GemTransactionHeaderKind::Swap => match &transfer.input_type {
            TransactionInputType::Swap { from_asset, to_asset, swap_data } => GemTransactionHeader::Swap {
                from: header_amount(leg(from_asset.clone(), swap_data.quote.from_value.clone(), prices), &currency, true),
                to: header_amount(leg(to_asset.clone(), swap_data.quote.to_value.clone(), prices), &currency, true),
            },
            _ => amount(true),
        },
        GemTransactionHeaderKind::Nft => match &transfer.input_type {
            TransactionInputType::TransferNft { nft_asset, .. } => GemTransactionHeader::Nft {
                image_url: GemImage::NftAsset { asset_id: nft_asset.id.to_string() }.url(),
                asset_id: nft_asset.id.clone(),
                name: Some(nft_asset.name.clone()),
            },
            _ => amount(false),
        },
        GemTransactionHeaderKind::Symbol => GemTransactionHeader::symbol(header_asset(transfer)),
        GemTransactionHeaderKind::AssetImage => GemTransactionHeader::asset_image(&header_asset(transfer)),
    }
}

fn header_asset(transfer: &GemTransferData) -> Asset {
    match &transfer.input_type {
        TransactionInputType::Withdrawal { .. } => GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset(),
        _ => transfer.input_type.get_asset().clone(),
    }
}

fn amount(transfer: &GemTransferData, load: Option<&GemConfirmLoad>, prices: &[AssetPrice], currency: &Currency) -> GemTransactionAmount {
    let asset = header_asset(transfer);
    let value = match load.and_then(|load| load.fee.as_ref()).map(|fee| &fee.amount) {
        Some(GemTransferAmountResult::Amount { amount }) => amount.value.to_biguint().unwrap_or_default(),
        _ => transfer.value.to_biguint().unwrap_or_default(),
    };
    if let TransactionInputType::Payment { invoice, .. } = &transfer.input_type
        && let Some(price) = &invoice.price
        && Currency::from_str(&price.currency).is_ok_and(|price_currency| price_currency == *currency)
        && value != GemBigUint::ZERO
    {
        return GemTransactionAmount {
            price: Some(AssetPrice::new(asset.id.clone(), price.amount / BigNumberFormatter::f64_value(&value, asset.decimals as u32), 0.0, Utc::now())),
            asset,
            value,
            sign: GemAmountSign::None,
        };
    }
    leg(asset, value, prices)
}

fn leg(asset: Asset, value: GemBigUint, prices: &[AssetPrice]) -> GemTransactionAmount {
    GemTransactionAmount {
        price: prices.iter().find(|price| price.asset_id == asset.id).cloned(),
        asset,
        value,
        sign: GemAmountSign::None,
    }
}

#[cfg(test)]
mod tests {
    use primitives::{ApprovalData, AssetId, Chain, NFTAsset, PaymentInvoice, PerpetualConfirmData, PerpetualDirection, PerpetualType, TransferDataExtra};

    use super::super::error::GemConfirmError;
    use super::super::model::GemApprovalValue;
    use crate::formatted_number::GemFormattedNumber;
    use crate::precision::GemValueStyle;
    use crate::services::localization::GemLocalizedText;

    use super::*;

    fn transfer(input_type: TransactionInputType) -> GemTransferData {
        GemTransferData::mock(input_type)
    }

    fn ready() -> GemConfirmScreen {
        GemConfirmScreen {
            phase: GemConfirmPhase::Ready,
            ..GemConfirmScreen::initial(None)
        }
    }

    #[test]
    fn test_a_token_approval_shows_the_asset_and_what_it_approves() {
        let asset = Asset::mock();
        let unlimited = header(
            &transfer(TransactionInputType::TokenApprove {
                asset: asset.clone(),
                approval_data: ApprovalData {
                    token: asset.id.to_string(),
                    spender: "0xspender".to_string(),
                    value: GemBigUint::from(1u32),
                    is_unlimited: true,
                },
            }),
            None,
            None,
            Currency::USD,
            &ready(),
        );

        assert_eq!(
            unlimited,
            GemConfirmHeader::Value {
                value: GemSimulationValue::new(asset, GemApprovalValue::Unlimited)
            },
            "both apps show the approved asset, not one of them a bare symbol"
        );
    }

    #[test]
    fn test_a_withdrawal_shows_the_asset_it_moves_rather_than_the_one_it_is_priced_in() {
        let header = header(&transfer(TransactionInputType::Withdrawal { asset: Asset::mock() }), None, None, Currency::USD, &ready());

        let GemConfirmHeader::Transaction {
            header: GemTransactionHeader::Amount { amount, .. },
        } = header
        else {
            panic!("a withdrawal reads as an amount")
        };
        assert_eq!(amount.asset, GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset());
    }

    #[test]
    fn test_a_perpetual_shows_the_market_asset_rather_than_its_collateral() {
        let market = Asset {
            id: AssetId::from_token(Chain::HyperCore, "perpetual::BTC"),
            symbol: "BTC".to_string(),
            ..Asset::from_chain(Chain::HyperCore)
        };
        let header = header(
            &transfer(TransactionInputType::Perpetual {
                asset: market.clone(),
                perpetual_type: PerpetualType::Open {
                    data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
                },
            }),
            None,
            None,
            Currency::USD,
            &ready(),
        );

        assert_eq!(
            header,
            GemConfirmHeader::Transaction {
                header: GemTransactionHeader::symbol(market)
            }
        );
    }

    #[test]
    fn test_a_generic_request_waits_on_the_asset_its_simulation_named() {
        let asset = Asset::mock();
        let simulation = SimulationResult {
            header: Some(primitives::SimulationHeader {
                asset_id: asset.id.clone(),
                value: Some(1u32.into()),
                is_unlimited: false,
            }),
            ..SimulationResult::new(vec![], vec![])
        };

        assert_eq!(
            header(
                &transfer(TransactionInputType::Generic {
                    asset: asset.clone(),
                    metadata: primitives::ApplicationMetadata::mock(),
                    extra: primitives::TransferDataExtra::default(),
                }),
                Some(&simulation),
                None,
                Currency::USD,
                &ready(),
            ),
            GemConfirmHeader::Placeholder { icon: asset_icon(&asset.id) },
            "the head keeps its place until the simulation answers"
        );
    }

    #[test]
    fn test_the_head_shows_the_requested_amount_until_the_fee_has_been_worked_out() {
        let sent = GemTransferData {
            value: 150_000u32.into(),
            ..transfer(TransactionInputType::Transfer { asset: Asset::mock() })
        };

        let GemConfirmHeader::Transaction {
            header: GemTransactionHeader::Amount { amount },
        } = header(&sent, None, None, Currency::USD, &ready())
        else {
            panic!("a transfer reads as an amount")
        };
        assert_eq!(
            amount.amount,
            GemFormattedNumber::asset_amount(&150_000u32.into(), &Asset::mock(), GemValueStyle::Auto),
            "the head is not empty while the fee loads"
        );
    }

    #[test]
    fn test_an_nft_transfer_carries_its_collectible_rather_than_an_amount() {
        let nft = NFTAsset::mock();
        let header = header(
            &transfer(TransactionInputType::TransferNft {
                asset: Asset::mock(),
                nft_asset: nft.clone(),
            }),
            None,
            None,
            Currency::USD,
            &ready(),
        );

        assert!(matches!(header, GemConfirmHeader::Transaction { header: GemTransactionHeader::Nft { ref asset_id, .. } } if *asset_id == nft.id));
    }

    #[test]
    fn test_a_payment_is_priced_by_its_invoice_in_the_invoice_currency_only() {
        use primitives::PaymentPrice;
        let asset = Asset::mock();
        let payment = GemTransferData {
            value: 250_000u32.into(),
            ..transfer(TransactionInputType::Payment {
                asset: asset.clone(),
                invoice: PaymentInvoice {
                    price: Some(PaymentPrice { currency: "USD".to_string(), amount: 0.1 }),
                    ..PaymentInvoice::mock()
                },
                extra: TransferDataExtra::mock(),
            })
        };

        let GemConfirmHeader::Transaction {
            header: GemTransactionHeader::Amount { amount },
        } = header(&payment, None, None, Currency::USD, &ready())
        else {
            panic!("a payment reads as an amount with its fiat");
        };
        let fiat = amount.fiat.unwrap().value;
        assert!((fiat - 0.1).abs() < 1e-9, "the fiat is the invoice price, not the market price: {fiat}");

        let GemConfirmHeader::Transaction {
            header: GemTransactionHeader::Amount { amount, .. },
        } = header(&payment, None, None, Currency::EUR, &ready())
        else {
            panic!("a payment reads as an amount");
        };
        assert_eq!(amount.fiat, None, "another wallet currency falls back to the market price");
    }

    #[test]
    fn test_an_amountless_payment_reserves_its_head_until_the_load_ends() {
        let payment = GemTransferData {
            value: GemBigInt::ZERO,
            ..transfer(TransactionInputType::Payment {
                asset: Asset::mock(),
                invoice: PaymentInvoice::mock(),
                extra: TransferDataExtra::mock(),
            })
        };
        let loading = GemConfirmScreen::initial(None);

        assert!(matches!(header(&payment, None, None, Currency::USD, &loading), GemConfirmHeader::Reserved { .. }));
        assert!(matches!(header(&payment, None, None, Currency::USD, &ready()), GemConfirmHeader::Transaction { .. }));
        assert!(
            matches!(
                header(&payment, None, None, Currency::USD, &loading.on_load_failed(GemConfirmError::Load { msg: "offline".into() })),
                GemConfirmHeader::Transaction { .. }
            ),
            "a failed load shows the head"
        );
        assert!(
            matches!(
                header(&transfer(TransactionInputType::Transfer { asset: Asset::mock() }), None, None, Currency::USD, &loading),
                GemConfirmHeader::Transaction { .. }
            ),
            "only an amountless payment waits"
        );
    }

    #[test]
    fn test_an_approval_header_names_an_unlimited_amount_and_shows_an_exact_one_in_full() {
        let usdt = Asset::mock_ethereum_usdc();
        let unlimited = GemSimulationValue::new(usdt.clone(), GemApprovalValue::Unlimited);
        let exact = GemSimulationValue::new(usdt.clone(), GemApprovalValue::Exact { value: 1_234_567u32.into() });

        assert_eq!(unlimited.header.title, GemLocalizedText::UnlimitedAsset { symbol: usdt.symbol.clone() });
        assert_eq!(
            exact.header.title,
            GemLocalizedText::Number {
                number: GemFormattedNumber::asset_amount(&1_234_567u32.into(), &usdt, GemValueStyle::Full)
            }
        );
    }
}
