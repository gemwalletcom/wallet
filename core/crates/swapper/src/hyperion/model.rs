use gem_aptos::{
    APTOS_NATIVE_METADATA_ADDRESS, AccountAddress, ENTRY_FUNCTION_PAYLOAD_TYPE, HYPERION_PARTNER_SWAP_FUNCTION, TransactionPayload,
    models::{Event, HyperionThirdPartySwapData, Transaction, fungible_store_change},
};
use num_bigint::{BigInt, BigUint};
use num_traits::ToPrimitive;
use primitives::{Address, AssetId, Chain, asset_constants::APTOS_USDC_TOKEN_ID, chain_aptos::is_fungible_asset_token_id};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::constants::POOL_CREATOR;
use crate::{
    Quote, SwapperError,
    fees::{DEFAULT_REFERRER, FeeSide, ReferralFee, subtract_bps},
};

pub(super) const INTERMEDIATE_TOKENS: [&str; 2] = [APTOS_NATIVE_METADATA_ADDRESS, APTOS_USDC_TOKEN_ID];

pub(super) fn token_metadata(asset_id: &AssetId) -> Result<AccountAddress, SwapperError> {
    if asset_id.chain != Chain::Aptos {
        return Err(SwapperError::NotSupportedAsset);
    }
    match asset_id.token_id.as_deref() {
        None => AccountAddress::from_hex(APTOS_NATIVE_METADATA_ADDRESS).map_err(|_| SwapperError::NotSupportedAsset),
        Some(token_id) if is_fungible_asset_token_id(token_id) => AccountAddress::from_hex(token_id).map_err(|_| SwapperError::NotSupportedAsset),
        Some(_) => Err(SwapperError::NotSupportedAsset),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct PoolRoute {
    pub pools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PoolQuote {
    pub pool: String,
    pub amount_out: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Pool {
    pub address: String,
    pub token_a: String,
}

impl Pool {
    pub fn new(token_x: &str, token_y: &str, fee_tier: u8) -> Result<Self, SwapperError> {
        let parse = |token: &str| AccountAddress::from_hex(token).map_err(|_| SwapperError::NotSupportedAsset);
        let (token_x, token_y) = (parse(token_x)?, parse(token_y)?);
        let (token_a, token_b) = if token_x <= token_y { (token_x, token_y) } else { (token_y, token_x) };
        let address = POOL_CREATOR.named_object(&[token_a.as_bytes(), token_b.as_bytes(), &[fee_tier]].concat());
        Ok(Self {
            address: address.to_string(),
            token_a: token_a.to_string(),
        })
    }

    pub fn quote(&self, from: &str, amount_out: u64, reserves: (u64, u64)) -> Option<PoolQuote> {
        let output_reserve = if self.token_a == from { reserves.1 } else { reserves.0 };
        (amount_out > 0 && amount_out < output_reserve).then(|| PoolQuote { pool: self.address.clone(), amount_out })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PartnerSwap {
    pub pools: Vec<String>,
    pub token_in: AccountAddress,
    pub token_out: AccountAddress,
    pub amount_in: u64,
    pub amount_out_min: u64,
    pub fee: u64,
    pub fee_receiver: AccountAddress,
}

impl PartnerSwap {
    pub fn new(quote: &Quote, route: &PoolRoute, referral_fee: &ReferralFee) -> Result<Self, SwapperError> {
        let amount_in = quote.from_value_u64()?;
        let amount_out = quote.to_value.to_u128().ok_or_else(|| SwapperError::ComputeQuoteError("quote amount is too large".into()))?;
        let amount_out_min = u64::try_from(subtract_bps(&amount_out, quote.data.slippage_bps)).map_err(SwapperError::compute_quote_error)?;
        Ok(Self {
            pools: route.pools.clone(),
            token_in: token_metadata(&quote.request.from_asset.asset_id())?,
            token_out: token_metadata(&quote.request.to_asset.asset_id())?,
            amount_in,
            amount_out_min,
            fee: referral_fee.transfer(FeeSide::Input, amount_in, 0).value,
            fee_receiver: AccountAddress::from_hex(&referral_fee.address).map_err(SwapperError::transaction_error)?,
        })
    }

    pub fn payload(&self) -> TransactionPayload {
        TransactionPayload {
            function: Some(HYPERION_PARTNER_SWAP_FUNCTION.to_string()),
            type_arguments: vec![],
            arguments: vec![
                json!(self.pools),
                json!(self.token_in.to_string()),
                json!(self.token_out.to_string()),
                json!(self.amount_in.to_string()),
                json!(self.amount_out_min.to_string()),
                json!(DEFAULT_REFERRER),
                json!(self.fee.to_string()),
                json!(self.fee_receiver.to_string()),
            ],
            payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
        }
    }

    pub fn verify_simulation(&self, wallet: &AccountAddress, simulation: &Transaction) -> Result<u64, SwapperError> {
        let gas_used = simulation.simulated_gas().map_err(|error| SwapperError::TransactionError(format!("Hyperion swap simulation failed: {error}")))?;
        let events = simulation.events.as_deref().unwrap_or_default();
        let fee_recorded = events.iter().filter_map(Event::hyperion_third_party_swap).any(|event| self.is_recorded_by(&event));
        let fee_received = fungible_store_change(events, &self.fee_receiver.primary_store(&self.token_in));
        let input_change = fungible_store_change(events, &wallet.primary_store(&self.token_in));
        let output_change = fungible_store_change(events, &wallet.primary_store(&self.token_out));

        if !fee_recorded || fee_received != BigInt::from(self.fee) {
            return Err(SwapperError::TransactionError("Hyperion swap simulation did not pay the referral fee".into()));
        }
        if input_change < -BigInt::from(self.amount_in) {
            return Err(SwapperError::TransactionError("Hyperion swap simulation spent more than the swap amount".into()));
        }
        if output_change < BigInt::from(self.amount_out_min) {
            return Err(SwapperError::TransactionError("Hyperion swap simulation paid less than the minimum output".into()));
        }
        Ok(gas_used)
    }

    fn is_recorded_by(&self, event: &HyperionThirdPartySwapData) -> bool {
        event.amount_in == BigUint::from(self.amount_in)
            && event.third_party_fee == BigUint::from(self.fee)
            && AccountAddress::from_hex(&event.token_in.inner).ok().as_ref() == Some(&self.token_in)
            && AccountAddress::from_hex(&event.token_out.inner).ok().as_ref() == Some(&self.token_out)
            && AccountAddress::from_hex(&event.third_party_receiver).ok().as_ref() == Some(&self.fee_receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderData, ProviderType, QuoteRequest, SwapperProvider, SwapperQuoteAsset, fees::default_referral_fees};
    use primitives::asset_constants::{APTOS_USDC_ASSET_ID, APTOS_USDT_TOKEN_ID};

    const TEST_WALLET: &str = "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef";
    const SIMULATION: &str = include_str!("../../testdata/hyperion/partner_swap_simulation.json");

    #[test]
    fn test_token_metadata() {
        assert_eq!(token_metadata(&AssetId::from_chain(Chain::Aptos)).unwrap(), AccountAddress::from_hex("0xa").unwrap());
        assert_eq!(token_metadata(&APTOS_USDC_ASSET_ID).unwrap(), AccountAddress::from_hex(APTOS_USDC_TOKEN_ID).unwrap());
        assert_eq!(token_metadata(&AssetId::from_chain(Chain::Sui)), Err(SwapperError::NotSupportedAsset));
        assert_eq!(
            token_metadata(&AssetId::from_token(Chain::Aptos, "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC")),
            Err(SwapperError::NotSupportedAsset)
        );
    }

    #[test]
    fn test_pool_new() {
        let apt = AccountAddress::from_hex("0xa").unwrap().to_string();
        let apt_usdc = Pool {
            address: "0x925660b8618394809f89f8002e2926600c775221f43bf1919782b297a79400d8".to_string(),
            token_a: apt.clone(),
        };

        assert_eq!(Pool::new(&apt, APTOS_USDC_TOKEN_ID, 1), Ok(apt_usdc.clone()));
        assert_eq!(Pool::new(APTOS_USDC_TOKEN_ID, &apt, 1), Ok(apt_usdc));
        assert_eq!(
            Pool::new(APTOS_USDC_TOKEN_ID, APTOS_USDT_TOKEN_ID, 5),
            Ok(Pool {
                address: "0xe914b87896574534e95722004df09d012aa985ed451d72a9df4b1343eb9691c1".to_string(),
                token_a: APTOS_USDT_TOKEN_ID.to_string(),
            })
        );
    }

    #[test]
    fn test_pool_quote() {
        let pool = Pool {
            address: "0xd3894a".to_string(),
            token_a: APTOS_USDT_TOKEN_ID.to_string(),
        };
        let reserves = (130_064_896_770, 17_353);

        assert_eq!(pool.quote(APTOS_USDT_TOKEN_ID, 29_810, reserves), None);
        assert_eq!(pool.quote(APTOS_USDT_TOKEN_ID, 17_353, reserves), None, "a pool cannot pay out its whole reserve");
        assert_eq!(
            pool.quote(APTOS_USDT_TOKEN_ID, 17_352, reserves),
            Some(PoolQuote {
                pool: "0xd3894a".to_string(),
                amount_out: 17_352
            })
        );
        assert_eq!(
            pool.quote(APTOS_USDC_TOKEN_ID, 29_810, reserves),
            Some(PoolQuote {
                pool: "0xd3894a".to_string(),
                amount_out: 29_810
            })
        );
        assert_eq!(pool.quote(APTOS_USDC_TOKEN_ID, 130_064_896_771, reserves), None);
        assert_eq!(pool.quote(APTOS_USDT_TOKEN_ID, 0, reserves), None);
    }

    #[test]
    fn test_partner_swap_new() {
        let quote = Quote {
            from_value: BigUint::from(100_000_000u64),
            to_value: BigUint::from(838_590u64),
            data: ProviderData {
                provider: ProviderType::new(SwapperProvider::Hyperion),
                routes: vec![],
                slippage_bps: 50,
            },
            request: QuoteRequest {
                from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Aptos)),
                to_asset: SwapperQuoteAsset::from(APTOS_USDC_ASSET_ID.clone()),
                wallet_address: TEST_WALLET.to_string(),
                destination_address: TEST_WALLET.to_string(),
                ..QuoteRequest::mock(Chain::Aptos, None)
            },
            ..Quote::mock(Chain::Aptos, None)
        };
        let route = PoolRoute { pools: PartnerSwap::mock().pools };

        assert_eq!(PartnerSwap::new(&quote, &route, &default_referral_fees().aptos), Ok(PartnerSwap::mock()));
    }

    #[test]
    fn test_partner_swap_payload() {
        let payload = PartnerSwap::mock().payload();

        assert_eq!(payload.function.as_deref(), Some(HYPERION_PARTNER_SWAP_FUNCTION));
        assert_eq!(payload.payload_type, ENTRY_FUNCTION_PAYLOAD_TYPE);
        assert_eq!(
            payload.arguments,
            vec![
                json!(["0x925660b8618394809f89f8002e2926600c775221f43bf1919782b297a79400d8"]),
                json!("0x000000000000000000000000000000000000000000000000000000000000000a"),
                json!(APTOS_USDC_TOKEN_ID),
                json!("100000000"),
                json!("834397"),
                json!("gemwallet"),
                json!("500000"),
                json!(default_referral_fees().aptos.address),
            ]
        );
    }

    #[test]
    fn test_verify_simulation() {
        let simulation: Transaction = serde_json::from_str(SIMULATION).unwrap();
        let wallet = AccountAddress::from_hex(TEST_WALLET).unwrap();
        let other = AccountAddress::from_hex("0x1").unwrap();
        let fee_error = Err(SwapperError::TransactionError("Hyperion swap simulation did not pay the referral fee".into()));
        let output_error = Err(SwapperError::TransactionError("Hyperion swap simulation paid less than the minimum output".into()));
        let abort = "Move abort in 0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c::router_v3: EOUT_AMOUNT_TOO_LESS";

        assert_eq!(PartnerSwap::mock().verify_simulation(&wallet, &simulation), Ok(691));
        assert_eq!(PartnerSwap { fee: 499_999, ..PartnerSwap::mock() }.verify_simulation(&wallet, &simulation), fee_error);
        assert_eq!(
            PartnerSwap {
                fee_receiver: other.clone(),
                ..PartnerSwap::mock()
            }
            .verify_simulation(&wallet, &simulation),
            fee_error
        );
        assert_eq!(
            PartnerSwap {
                amount_out_min: 834_398,
                ..PartnerSwap::mock()
            }
            .verify_simulation(&wallet, &simulation),
            output_error
        );
        assert_eq!(PartnerSwap::mock().verify_simulation(&other, &simulation), output_error);
        assert_eq!(
            PartnerSwap::mock().verify_simulation(&wallet, &simulation.with_event(Event::mock_withdraw(&wallet.primary_store(&PartnerSwap::mock().token_out), 1))),
            output_error
        );
        assert_eq!(
            PartnerSwap::mock().verify_simulation(&wallet, &simulation.with_event(Event::mock_withdraw(&wallet.primary_store(&PartnerSwap::mock().token_in), 1))),
            Err(SwapperError::TransactionError("Hyperion swap simulation spent more than the swap amount".into()))
        );
        assert_eq!(
            PartnerSwap::mock().verify_simulation(
                &wallet,
                &Transaction {
                    success: false,
                    vm_status: Some(abort.into()),
                    ..simulation
                }
            ),
            Err(SwapperError::TransactionError(format!("Hyperion swap simulation failed: {abort}")))
        );
    }
}
