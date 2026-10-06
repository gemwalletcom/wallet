use async_trait::async_trait;
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use futures::try_join;
use num_bigint::BigInt;
use std::error::Error;
use std::sync::Arc;

use gem_client::Client;
use primitives::transaction_load_metadata::AgentPrivateKey;
use primitives::{
    FeeOption, FeePriority, FeeRate, GasPriceType, HyperliquidOrder, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
    asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID, perpetual::PerpetualType,
};

use crate::constants::{NEW_ACCOUNT_FEE, TRANSACTION_FEE_UNITS, WITHDRAWAL_FEE};
use crate::is_spot_swap;
use crate::models::user::UserRole;
use crate::provider::fee_calculator::{calculate_perpetual_fee_amount, calculate_spot_fee_amount};
use crate::provider::preload_cache::{HyperCoreCache, UserFeeRates};
use crate::rpc::client::HyperCoreClient;

impl<C: Client> HyperCoreClient<C> {
    async fn get_order(&self, sender_address: &str) -> Result<(HyperliquidOrder, UserFeeRates), Box<dyn Error + Sync + Send>> {
        let cache = HyperCoreCache::new(self.preferences.clone(), self.config.clone());
        let (agent, referral_required, builder_required, fee_rates) = try_join!(
            cache.manage_agent(sender_address, self.secure_preferences.clone(), self.get_extra_agents(sender_address)),
            cache.needs_referral_approval(sender_address, self.get_referral(sender_address)),
            cache.needs_builder_fee_approval(sender_address, self.get_builder_fee(sender_address, &self.config.builder_address)),
            cache.get_user_fee_rates(sender_address, self.get_user_fees(sender_address)),
        )?;
        let fee_rates = if referral_required { cache.activate_referral_fee_rates(sender_address, &fee_rates)? } else { fee_rates.current() };

        Ok((
            HyperliquidOrder {
                approve_agent_required: agent.approval_required,
                approve_referral_required: referral_required,
                approve_builder_required: builder_required,
                builder_fee_bps: self.config.max_builder_fee_bps,
                agent_name: agent.name,
                agent_address: agent.address,
                agent_private_key: Arc::new(AgentPrivateKey::new(agent.private_key)),
            },
            fee_rates,
        ))
    }
}

fn perpetual_fee_fiat_value(perpetual_type: &PerpetualType) -> Result<f64, Box<dyn Error + Send + Sync>> {
    let data = match perpetual_type {
        PerpetualType::Open { data } | PerpetualType::Close { data } | PerpetualType::Increase { data } => data,
        PerpetualType::Reduce { data } => &data.data,
        PerpetualType::Modify { .. } => return Ok(0.0),
    };

    Ok(data.size.parse::<f64>()?.abs() * data.market_price)
}

fn transfer_fee(destination_role: &UserRole) -> TransactionFee {
    match destination_role {
        UserRole::Missing => TransactionFee::new_from_fee_with_option(BigInt::from(0), FeeOption::TokenAccountCreation, BigInt::from(NEW_ACCOUNT_FEE), HYPERCORE_SPOT_USDC_ASSET_ID.clone()),
        UserRole::Agent { .. } | UserRole::Other => TransactionFee::new_from_fee(BigInt::from(0), HYPERCORE_SPOT_USDC_ASSET_ID.clone()),
    }
}

#[async_trait]
impl<C: Client> ChainTransactionLoad for HyperCoreClient<C> {
    fn transaction_fee_estimate_units(&self, _operation: TransactionFeeOperation) -> Option<u64> {
        Some(TRANSACTION_FEE_UNITS)
    }

    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        match &input.input_type {
            TransactionInputType::Transfer { .. } => Ok(TransactionLoadData {
                fee: transfer_fee(&self.get_user_role(&input.destination_address).await?),
                metadata: TransactionLoadMetadata::Hyperliquid { order: None },
            }),
            TransactionInputType::Deposit { .. } | TransactionInputType::TransferNft { .. } | TransactionInputType::Account { .. } | TransactionInputType::Stake { .. } => Ok(TransactionLoadData {
                fee: TransactionFee::new_from_fee(BigInt::from(0), HYPERCORE_SPOT_USDC_ASSET_ID.clone()),
                metadata: TransactionLoadMetadata::Hyperliquid { order: None },
            }),
            TransactionInputType::Withdrawal { asset } => Ok(TransactionLoadData {
                fee: TransactionFee::new_from_fee(BigInt::from(WITHDRAWAL_FEE), asset.id.clone()),
                metadata: TransactionLoadMetadata::Hyperliquid { order: None },
            }),
            TransactionInputType::Swap { from_asset, to_asset, .. } => {
                let (fee_amount, order) = if is_spot_swap(from_asset.chain(), to_asset.chain()) {
                    let (order, fee_rates) = self.get_order(&input.sender_address).await?;
                    let swap_data = input.input_type.get_swap_data().map_err(ToString::to_string)?;
                    let fee_amount = calculate_spot_fee_amount(swap_data, from_asset, to_asset, fee_rates.spot_cross, order.builder_fee_bps)?;

                    (fee_amount, Some(order))
                } else {
                    (BigInt::from(0), None)
                };

                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(fee_amount, HYPERCORE_SPOT_USDC_ASSET_ID.clone()),
                    metadata: TransactionLoadMetadata::Hyperliquid { order },
                })
            }
            TransactionInputType::Perpetual { perpetual_type, .. } => {
                let fiat_value = perpetual_fee_fiat_value(perpetual_type)?;
                let fee_asset = perpetual_type.base_asset().id.clone();
                let (order, fee_rates) = self.get_order(&input.sender_address).await?;
                let fee_amount = calculate_perpetual_fee_amount(fiat_value, fee_rates.perpetual_cross, order.builder_fee_bps);

                Ok(TransactionLoadData {
                    fee: TransactionFee::new_from_fee(fee_amount, fee_asset),
                    metadata: TransactionLoadMetadata::Hyperliquid { order: Some(order) },
                })
            }
            _ => Err("Unsupported input type".to_string().into()),
        }
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{PerpetualConfirmData, PerpetualDirection, known_assets::HYPERCORE_PERPETUAL_USDC};

    #[test]
    fn test_perpetual_fee_fiat_value_uses_signed_size_at_market_price() {
        let data = PerpetualConfirmData {
            size: "1.259".to_string(),
            market_price: 794.14,
            fiat_value: 1_000.0,
            ..PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)
        };

        let fiat_value = perpetual_fee_fiat_value(&PerpetualType::Close { data }).unwrap();

        assert!((fiat_value - 999.82226).abs() < 1e-9);
        assert_eq!(calculate_perpetual_fee_amount(fiat_value, 0.000432, 45), BigInt::from(881_843));
    }

    #[test]
    fn test_transfer_fee() {
        let new_account = transfer_fee(&UserRole::Missing);
        assert_eq!(new_account.fee, BigInt::from(NEW_ACCOUNT_FEE));
        assert_eq!(new_account.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(NEW_ACCOUNT_FEE)));
        assert_eq!(new_account.fee_asset, *HYPERCORE_SPOT_USDC_ASSET_ID);

        let existing = transfer_fee(&UserRole::Other);
        assert_eq!(existing.fee, BigInt::from(0));
        assert!(existing.options.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction_load_withdrawal() {
        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Withdrawal { asset: HYPERCORE_PERPETUAL_USDC.clone() });

        let load = HyperCoreClient::mock().get_transaction_load(input).await.unwrap();

        assert_eq!(load.fee.fee, BigInt::from(WITHDRAWAL_FEE));
        assert_eq!(load.fee.fee_asset, HYPERCORE_PERPETUAL_USDC.id);
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_hypercore_test_client;
    use primitives::{Asset, Chain, TransactionLoadInput};

    #[tokio::test]
    async fn test_get_transaction_load_transfer() {
        let client = create_hypercore_test_client();
        let input = TransactionLoadInput {
            destination_address: "0x1085c5f70f7f7591d97da281a64688385455c2bd".to_string(),
            ..TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::HyperCore) })
        };

        let result = client.get_transaction_load(input).await.unwrap();

        assert_eq!(result.fee.fee, BigInt::from(0));
        let TransactionLoadMetadata::Hyperliquid { order } = result.metadata else {
            panic!("invalid metadata");
        };
        assert!(order.is_none());
    }
}
