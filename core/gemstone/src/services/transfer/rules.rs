use chrono::Utc;
use num_bigint::BigInt;
use primitives::SwapProvider;
use primitives::known_assets::wallet_default_assets;
use primitives::swap::ApprovalData;
use primitives::{
    ApplicationMetadataSource, Asset, AssetId, AssetType, Chain, EarnType, FeePriority, PerpetualType, RecentActivityType, StakeType, Transaction, TransactionDirection,
    TransactionInputType, TransactionNFTTransferMetadata, TransactionPerpetualMetadata, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata,
    TransactionType, TransactionWalletConnectMetadata, TransferDataOutputAction, TransferDataOutputType,
};

use super::model::{GemPendingTransactionInput, GemRecentActivity, GemTransferBalance, GemTransferData, GemTransferOutput};
use crate::models::transaction::{GemTransactionInputType, transaction_metadata_block_number, transaction_metadata_sequence};
use crate::services::amount::model::GemAmountError;

#[uniffi::export]
impl GemTransactionInputType {
    pub fn transaction_type(&self) -> TransactionType {
        TransactionInputType::from(self.clone()).transaction_type()
    }

    pub fn transaction_asset(&self) -> Asset {
        match self {
            Self::TransferNft { asset, .. } => Asset::from_chain(asset.chain()),
            _ => self.asset().clone(),
        }
    }

    pub fn asset_ids(&self) -> Vec<AssetId> {
        match self {
            Self::Swap { from_asset, to_asset, .. } => vec![from_asset.id.clone(), to_asset.id.clone()],
            Self::TransferNft { .. } => vec![],
            _ => vec![self.asset().id.clone()],
        }
    }

    pub fn fee_asset(&self) -> Asset {
        let asset = self.transaction_asset();
        let chain = asset.chain();
        if let Self::Perpetual { .. } = self
            && chain == Chain::HyperCore
        {
            return default_asset(chain, AssetType::PERPETUAL).unwrap_or(asset);
        }
        match chain {
            Chain::Tempo => asset,
            Chain::HyperCore => default_asset(chain, AssetType::TOKEN).unwrap_or(asset),
            _ if asset.id.is_token() => Asset::from_chain(chain),
            _ => asset,
        }
    }

    pub fn recent_activity(&self) -> Option<GemRecentActivity> {
        match self {
            Self::Transfer { asset } => Some(GemRecentActivity {
                activity_type: RecentActivityType::Transfer,
                asset_id: asset.id.clone(),
                to_asset_id: None,
            }),
            Self::Swap { from_asset, to_asset, .. } => Some(GemRecentActivity {
                activity_type: RecentActivityType::Swap,
                asset_id: from_asset.id.clone(),
                to_asset_id: Some(to_asset.id.clone()),
            }),
            Self::Deposit { .. }
            | Self::Stake { .. }
            | Self::TokenApprove { .. }
            | Self::Generic { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => None,
        }
    }

    pub fn default_fee_priority(&self) -> FeePriority {
        match self {
            Self::Swap { from_asset, .. } if from_asset.chain() == Chain::Bitcoin => FeePriority::Fast,
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::Swap { .. }
            | Self::Stake { .. }
            | Self::TokenApprove { .. }
            | Self::Generic { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => FeePriority::Normal,
        }
    }

    pub fn application_short_name(&self) -> Option<String> {
        match self {
            Self::Generic { metadata, .. } => Some(metadata.short_name()),
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::Swap { .. }
            | Self::Stake { .. }
            | Self::TokenApprove { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => None,
        }
    }

    pub fn output(&self) -> GemTransferOutput {
        match self {
            Self::Generic { extra, .. } => GemTransferOutput {
                output_type: extra.output_type.clone(),
                output_action: extra.output_action.clone(),
            },
            _ => GemTransferOutput {
                output_type: TransferDataOutputType::EncodedTransaction,
                output_action: TransferDataOutputAction::Send,
            },
        }
    }
}

impl GemTransactionInputType {
    pub(crate) fn approval(&self, transaction_type: TransactionType) -> Result<Option<ApprovalData>, String> {
        if transaction_type != TransactionType::TokenApproval {
            return Ok(None);
        }
        match self {
            Self::Swap { swap_data, .. } => swap_data.data.approval.clone().map(Some).ok_or("Missing swap approval data".to_string()),
            Self::Earn { data, .. } => data.approval.clone().map(Some).ok_or("Missing earn approval data".to_string()),
            Self::TokenApprove { approval_data, .. } => Ok(Some(approval_data.clone())),
            Self::Generic { extra, .. } => Ok(extra.approval.clone()),
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::Stake { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Withdrawal { .. } => Err("Token approval transaction type does not match transfer data".to_string()),
        }
    }

    pub(crate) fn metadata(&self) -> Result<Option<serde_json::Value>, serde_json::Error> {
        let value = match self {
            Self::Swap { from_asset, to_asset, swap_data } => Some(serde_json::to_value(TransactionSwapMetadata {
                from_asset: from_asset.id.clone(),
                from_value: swap_data.quote.from_value.clone(),
                to_asset: to_asset.id.clone(),
                to_value: swap_data.quote.to_value.clone(),
                provider: Some(swap_data.quote.provider_data.provider.as_ref().to_string()),
            })?),
            Self::TransferNft { nft_asset, .. } => Some(serde_json::to_value(TransactionNFTTransferMetadata::new(
                nft_asset.id.clone(),
                Some(nft_asset.name.clone()),
            ))?),
            Self::Perpetual { perpetual_type, .. } => match perpetual_type {
                PerpetualType::Open(data) | PerpetualType::Close(data) | PerpetualType::Increase(data) => Some(serde_json::to_value(TransactionPerpetualMetadata {
                    pnl: 0.0,
                    price: 0.0,
                    direction: data.direction.clone(),
                    is_liquidation: None,
                    provider: None,
                })?),
                PerpetualType::Reduce(data) => Some(serde_json::to_value(TransactionPerpetualMetadata {
                    pnl: 0.0,
                    price: 0.0,
                    direction: data.position_direction.clone(),
                    is_liquidation: None,
                    provider: None,
                })?),
                PerpetualType::Modify(_) => None,
            },
            Self::Stake { stake_type, .. } => match stake_type {
                StakeType::Freeze(data) | StakeType::Unfreeze(data) => Some(serde_json::to_value(TransactionResourceTypeMetadata::new(*data))?),
                StakeType::Stake(_) | StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => None,
            },
            Self::Generic { metadata, extra, .. } => match metadata.source {
                ApplicationMetadataSource::WalletConnect => Some(serde_json::to_value(TransactionWalletConnectMetadata {
                    output_action: extra.output_action.clone(),
                })?),
                ApplicationMetadataSource::Payment => None,
            },
            Self::Transfer { .. } | Self::Deposit { .. } | Self::Withdrawal { .. } | Self::TokenApprove { .. } | Self::Account { .. } | Self::Earn { .. } => None,
        };
        Ok(value)
    }

    fn is_tracked(&self, transaction_type: &TransactionType, hash: &str, index: u32, count: u32) -> bool {
        if *transaction_type == TransactionType::PerpetualModifyPosition {
            return false;
        }
        if self.asset().chain() != Chain::HyperCore {
            return true;
        }
        let is_intermediate = index + 1 < count;
        match self {
            Self::Stake { .. } => !is_intermediate,
            Self::Perpetual { .. } => hash.starts_with(HYPERCORE_ORDER_PREFIX),
            Self::Swap { to_asset, swap_data, .. } => {
                !(to_asset.chain() == Chain::HyperCore && swap_data.quote.provider_data.provider == SwapProvider::Hyperliquid && is_intermediate)
            }
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::TokenApprove { .. }
            | Self::Generic { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => true,
        }
    }
}

pub(crate) fn tron_stake_available(asset: &Asset, balance: &GemTransferBalance) -> BigInt {
    let staked = BigInt::from(balance.votes) * BigInt::from(10u32).pow(asset.decimals.max(0) as u32);
    (&balance.frozen + &balance.locked - staked).max(BigInt::from(0))
}

pub(crate) fn unfreeze_available(resource: &primitives::Resource, balance: &GemTransferBalance) -> BigInt {
    match resource {
        primitives::Resource::Bandwidth => balance.frozen.clone(),
        primitives::Resource::Energy => balance.locked.clone(),
    }
}

impl GemTransferData {
    pub(crate) fn available_value(&self, balance: &GemTransferBalance) -> Result<BigInt, GemAmountError> {
        let asset = self.input_type.asset();
        Ok(match &self.input_type {
            GemTransactionInputType::Withdrawal { .. } => balance.withdrawable.clone(),
            GemTransactionInputType::Stake { stake_type, .. } => match stake_type {
                StakeType::Unstake(delegation) | StakeType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
                StakeType::Redelegate(data) => BigInt::from(data.delegation.base.balance.clone()),
                StakeType::Rewards(_) => self.value.clone(),
                StakeType::Unfreeze(resource) => unfreeze_available(resource, balance),
                StakeType::Stake(_) if asset.chain() == Chain::Tron => tron_stake_available(asset, balance),
                StakeType::Stake(_) | StakeType::Freeze(_) => balance.available.clone(),
            },
            GemTransactionInputType::Earn { earn_type, .. } => match earn_type {
                EarnType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
                EarnType::Deposit(_) => balance.available.clone(),
            },
            GemTransactionInputType::Transfer { .. }
            | GemTransactionInputType::Deposit { .. }
            | GemTransactionInputType::Swap { .. }
            | GemTransactionInputType::TokenApprove { .. }
            | GemTransactionInputType::Generic { .. }
            | GemTransactionInputType::TransferNft { .. }
            | GemTransactionInputType::Account { .. }
            | GemTransactionInputType::Perpetual { .. } => balance.available.clone(),
        })
    }
}

impl GemPendingTransactionInput {
    pub(crate) fn pending_transaction(self) -> Result<Option<Transaction>, String> {
        if !self
            .transfer
            .input_type
            .is_tracked(&self.transaction_type, &self.hash, self.transaction_index, self.transaction_count)
        {
            return Ok(None);
        }
        let transfer = self.transfer;
        let chain = transfer.input_type.asset().chain();
        let approval = transfer.input_type.approval(self.transaction_type.clone())?;
        let simulation_header = match transfer.input_type {
            GemTransactionInputType::Generic { .. } => self.simulation.and_then(|simulation| simulation.header),
            _ => None,
        };
        let (recipient, value, memo) = match &approval {
            Some(approval) => (approval.spender.clone(), approval.value.clone(), String::new()),
            None => {
                let recipient = match &transfer.input_type {
                    GemTransactionInputType::Swap { swap_data, .. } => swap_data.data.to.clone(),
                    _ => transfer.recipient.address.clone(),
                };
                let value = simulation_header
                    .as_ref()
                    .and_then(|header| header.value.as_ref())
                    .map(ToString::to_string)
                    .unwrap_or_else(|| self.value.to_string());
                let memo = match &transfer.input_type {
                    GemTransactionInputType::Swap { .. } => String::new(),
                    _ => transfer.recipient.memo.clone().unwrap_or_default(),
                };
                (recipient, value, memo)
            }
        };
        let asset_id = simulation_header
            .as_ref()
            .map(|header| header.asset_id.clone())
            .or_else(|| approval.as_ref().map(|approval| AssetId::from(chain, Some(approval.token.clone()))))
            .unwrap_or_else(|| transfer.input_type.transaction_asset().id);
        let direction = if self.sender == recipient {
            TransactionDirection::SelfTransfer
        } else {
            TransactionDirection::Outgoing
        };
        let metadata = match transfer.input_type {
            GemTransactionInputType::Swap { .. } | GemTransactionInputType::Earn { .. } if approval.is_some() => None,
            _ => transfer.input_type.metadata().map_err(|error| error.to_string())?,
        };
        let mut transaction = Transaction::new(
            self.hash,
            asset_id,
            self.sender,
            recipient,
            None,
            self.transaction_type,
            TransactionState::Pending,
            self.network_fee.to_string(),
            self.fee.fee_asset,
            value,
            Some(memo),
            metadata,
            Utc::now(),
        );
        transaction.block_number = Some(transaction_metadata_block_number(&self.metadata));
        transaction.sequence = Some(transaction_metadata_sequence(&self.metadata));
        transaction.direction = direction;
        Ok(Some(transaction))
    }
}

const HYPERCORE_ORDER_PREFIX: &str = "order:";

fn default_asset(chain: Chain, asset_type: AssetType) -> Option<Asset> {
    wallet_default_assets(chain).into_iter().find(|asset| asset.asset_type == asset_type)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_only_a_transfer_and_a_swap_leave_recent_activity() {
        let from = Asset::from_chain(Chain::Ethereum);
        let to = Asset::from_chain(Chain::Solana);

        let transfer = (GemTransactionInputType::Transfer { asset: from.clone() }).recent_activity().unwrap();
        assert_eq!(transfer.activity_type, RecentActivityType::Transfer);
        assert_eq!(transfer.asset_id, from.id);
        assert_eq!(transfer.to_asset_id, None);

        let swap = (GemTransactionInputType::Swap {
            from_asset: from.clone(),
            to_asset: to.clone(),
            swap_data: crate::models::swap::GemSwapData::mock(),
        })
        .recent_activity()
        .unwrap();
        assert_eq!(swap.activity_type, RecentActivityType::Swap);
        assert_eq!(swap.asset_id, from.id);
        assert_eq!(swap.to_asset_id, Some(to.id));

        assert!((GemTransactionInputType::Deposit { asset: from.clone() }).recent_activity().is_none());
        assert!((GemTransactionInputType::Withdrawal { asset: from }).recent_activity().is_none());
    }

    use super::super::model::GemRecipient;
    use super::*;
    use crate::models::gateway::GemGasPriceType;
    use crate::models::transaction::{GemFeeOptions, GemTransactionLoadFee, GemTransactionLoadMetadata};
    use num_bigint::BigUint;
    use primitives::{
        Delegation, DelegationBase, DelegationState, DelegationValidator, NFTAsset, PerpetualConfirmData, PerpetualDirection, Resource, StakeProviderType, SwapProvider,
        TransactionType, TransferDataExtra,
        known_assets::HYPERCORE_PERPETUAL_USDC,
        swap::{SwapData, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType},
    };
    use std::collections::HashMap;

    fn asset(chain: Chain) -> Asset {
        Asset::from_chain(chain)
    }

    fn token(chain: Chain, token_id: &str) -> Asset {
        Asset::new(AssetId::from(chain, Some(token_id.to_string())), "USDC".into(), "USDC".into(), 6, AssetType::ERC20)
    }

    fn swap_input(from: Asset, to: Asset, provider: SwapProvider, approval: Option<ApprovalData>) -> GemTransactionInputType {
        GemTransactionInputType::Swap {
            from_asset: from.clone(),
            to_asset: to.clone(),
            swap_data: SwapData {
                quote: SwapQuote {
                    from_address: "from".into(),
                    from_value: "100".into(),
                    min_from_value: None,
                    to_address: "to".into(),
                    to_value: "90".into(),
                    provider_data: SwapProviderData {
                        provider,
                        name: "provider".into(),
                        protocol_name: "protocol".into(),
                    },
                    slippage_bps: 50,
                    eta_in_seconds: None,
                    use_max_amount: None,
                },
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    data_type: SwapQuoteDataType::Contract,
                    value: "100".to_string(),
                    data: "0x".into(),
                    memo: None,
                    approval,
                    gas_limit: None,
                },
            },
        }
    }

    fn delegation(balance: u64) -> Delegation {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Cosmos),
                state: DelegationState::Active,
                balance: BigUint::from(balance),
                shares: BigUint::default(),
                rewards: BigUint::from(5u64),
                completion_date: None,
                delegation_id: "delegation".into(),
                validator_id: "validator".into(),
            },
            validator: DelegationValidator {
                chain: Chain::Cosmos,
                id: "validator".into(),
                name: "validator".into(),
                is_active: true,
                commission: 0.0,
                apr: 0.0,
                provider_type: StakeProviderType::Stake,
            },
            price: None,
        }
    }

    fn perpetual_input(asset: Asset) -> GemTransactionInputType {
        GemTransactionInputType::Perpetual {
            asset,
            perpetual_type: PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
        }
    }

    fn transfer(input_type: GemTransactionInputType, value: &str) -> GemTransferData {
        GemTransferData {
            input_type,
            recipient: GemRecipient {
                address: "recipient".into(),
                name: None,
                memo: Some("memo".into()),
                references: vec![],
            },
            value: value.parse().unwrap(),
            use_max_amount: false,
            minimum_value: None,
        }
    }

    fn balance(available: u64, frozen: u64, locked: u64) -> GemTransferBalance {
        GemTransferBalance {
            available: BigInt::from(available),
            frozen: BigInt::from(frozen),
            locked: BigInt::from(locked),
            withdrawable: BigInt::ZERO,
            votes: 0,
        }
    }

    fn pending_input(input_type: GemTransactionInputType, transaction_type: TransactionType, hash: &str, index: u32, count: u32) -> GemPendingTransactionInput {
        GemPendingTransactionInput {
            sender: "sender".into(),
            transfer: transfer(input_type, "100"),
            value: BigInt::from(99),
            transaction_type,
            hash: hash.into(),
            fee: GemTransactionLoadFee {
                fee: BigInt::from(1),
                gas_price_type: GemGasPriceType::Regular { gas_price: BigInt::from(1) },
                gas_limit: BigInt::from(21_000),
                options: GemFeeOptions { options: HashMap::new() },
                fee_asset: AssetId::from_chain(Chain::Ethereum),
            },
            network_fee: BigInt::from(1),
            metadata: GemTransactionLoadMetadata::None,
            simulation: None,
            transaction_index: index,
            transaction_count: count,
        }
    }

    #[test]
    fn test_swap_rules() {
        let input = swap_input(asset(Chain::Ethereum), asset(Chain::Bitcoin), SwapProvider::Thorchain, None);
        assert_eq!(input.transaction_type(), TransactionType::Swap);
        assert_eq!(input.asset_ids(), vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Bitcoin)]);
        assert_eq!(input.fee_asset().id, AssetId::from_chain(Chain::Ethereum));
        let metadata = input.metadata().unwrap().unwrap();
        assert_eq!(metadata["provider"], "thorchain");
        assert_eq!(metadata["toValue"], "90");
        assert!(input.approval(TransactionType::TokenApproval).is_err());
        assert_eq!(input.approval(TransactionType::Swap).unwrap(), None);
    }

    #[test]
    fn test_fee_asset_for_tokens_and_hypercore() {
        let token_transfer = GemTransactionInputType::Transfer {
            asset: token(Chain::Ethereum, "0xusdc"),
        };
        assert_eq!(token_transfer.fee_asset().id, AssetId::from_chain(Chain::Ethereum));
        let tempo_token = GemTransactionInputType::Transfer {
            asset: token(Chain::Tempo, "0xusdc"),
        };
        assert!(tempo_token.fee_asset().id.is_token());
        let hypercore = GemTransactionInputType::Transfer { asset: asset(Chain::HyperCore) };
        assert_eq!(hypercore.fee_asset().asset_type, AssetType::TOKEN);
        assert_eq!(tempo_token.fee_asset().id, token(Chain::Tempo, "0xusdc").id);

        let perpetual = perpetual_input(asset(Chain::HyperCore));
        assert_eq!(perpetual.fee_asset().id, HYPERCORE_PERPETUAL_USDC.id);
        assert_eq!(perpetual.fee_asset().asset_type, AssetType::PERPETUAL);
        let nft = GemTransactionInputType::TransferNft {
            asset: token(Chain::Ethereum, "0xusdc"),
            nft_asset: NFTAsset::mock(),
        };
        assert_eq!(nft.fee_asset().id, AssetId::from_chain(Chain::Ethereum));
        let spl = GemTransactionInputType::Transfer { asset: Asset::mock_spl_token() };
        assert_eq!(spl.fee_asset().id, AssetId::from_chain(Chain::Solana));
    }

    #[test]
    fn test_output() {
        let signature = GemTransactionInputType::Generic {
            asset: asset(Chain::Ethereum),
            metadata: primitives::ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_type: TransferDataOutputType::Signature,
                output_action: TransferDataOutputAction::Sign,
                ..TransferDataExtra::mock()
            }
            .into(),
        };
        let signed = signature.output();
        assert_eq!(signed.output_type, TransferDataOutputType::Signature);
        assert_eq!(signed.output_action, TransferDataOutputAction::Sign);
        assert_eq!(signature.application_short_name().as_deref(), Some("Test Dapp"));

        let withdrawal = GemTransactionInputType::Withdrawal { asset: asset(Chain::HyperCore) };
        let sent = withdrawal.output();
        assert_eq!(sent.output_type, TransferDataOutputType::EncodedTransaction);
        assert_eq!(sent.output_action, TransferDataOutputAction::Send);
        assert_eq!(withdrawal.application_short_name(), None);
        assert_eq!(perpetual_input(asset(Chain::HyperCore)).output().output_action, TransferDataOutputAction::Send);
    }

    #[test]
    fn test_hypercore_tracking_skips_intermediate_legs_and_non_orders() {
        let intermediate = swap_input(asset(Chain::HyperCore), asset(Chain::HyperCore), SwapProvider::Hyperliquid, None);
        assert!(
            pending_input(intermediate.clone(), TransactionType::Swap, "h", 0, 2)
                .pending_transaction()
                .unwrap()
                .is_none()
        );
        assert!(pending_input(intermediate, TransactionType::Swap, "h", 1, 2).pending_transaction().unwrap().is_some());
        let other_provider = swap_input(asset(Chain::HyperCore), asset(Chain::HyperCore), SwapProvider::Thorchain, None);
        assert!(pending_input(other_provider, TransactionType::Swap, "h", 0, 2).pending_transaction().unwrap().is_some());

        let perpetual = perpetual_input(asset(Chain::HyperCore));
        assert!(
            pending_input(perpetual.clone(), TransactionType::PerpetualOpenPosition, "order:1", 0, 1)
                .pending_transaction()
                .unwrap()
                .is_some()
        );
        assert!(
            pending_input(perpetual, TransactionType::PerpetualOpenPosition, "0xabc", 0, 1)
                .pending_transaction()
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_stake_metadata_and_available_value() {
        let unfreeze = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Unfreeze(Resource::Bandwidth),
        };
        assert_eq!(unfreeze.metadata().unwrap().unwrap()["resourceType"], "bandwidth");
        assert_eq!(transfer(unfreeze, "1").available_value(&balance(10, 20, 30)).unwrap(), BigInt::from(20));

        let unstake = GemTransactionInputType::Stake {
            asset: asset(Chain::Cosmos),
            stake_type: StakeType::Unstake(delegation(700)),
        };
        assert_eq!(transfer(unstake, "1").available_value(&balance(10, 0, 0)).unwrap(), BigInt::from(700));
        let rewards = GemTransactionInputType::Stake {
            asset: asset(Chain::Cosmos),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert_eq!(transfer(rewards, "42").available_value(&balance(10, 0, 0)).unwrap(), BigInt::from(42));
        let tron_stake = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Stake(delegation(0).validator),
        };
        assert_eq!(
            transfer(tron_stake, "1")
                .available_value(&GemTransferBalance {
                    votes: 2,
                    ..balance(1, 5_000_000, 3_000_000)
                })
                .unwrap(),
            BigInt::from(6_000_000)
        );
        let overvoted = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Stake(delegation(0).validator),
        };
        assert_eq!(
            transfer(overvoted, "1")
                .available_value(&GemTransferBalance {
                    votes: 9,
                    ..balance(1, 5_000_000, 3_000_000)
                })
                .unwrap(),
            BigInt::from(0)
        );
        let withdrawal = GemTransactionInputType::Withdrawal { asset: asset(Chain::HyperCore) };
        assert_eq!(
            transfer(withdrawal, "1")
                .available_value(&GemTransferBalance {
                    withdrawable: BigInt::from(9),
                    ..balance(10, 0, 0)
                })
                .unwrap(),
            BigInt::from(9)
        );
    }

    #[test]
    fn test_pending_transaction_uses_swap_router_and_hypercore_tracking() {
        let swap = swap_input(asset(Chain::Ethereum), asset(Chain::Bitcoin), SwapProvider::Thorchain, None);
        let transaction = pending_input(swap, TransactionType::Swap, "0xhash", 0, 1).pending_transaction().unwrap().unwrap();
        assert_eq!(transaction.to, "0xrouter");
        assert_eq!(transaction.value, "99");
        assert_eq!(transaction.memo.as_deref(), Some(""));
        assert_eq!(transaction.direction, TransactionDirection::Outgoing);
        assert!(transaction.metadata.is_some());

        let approval_leg = swap_input(
            token(Chain::Ethereum, "0xusdc"),
            asset(Chain::Bitcoin),
            SwapProvider::Thorchain,
            Some(ApprovalData {
                token: "0xusdc".into(),
                spender: "0xspender".into(),
                value: "100".to_string(),
                is_unlimited: false,
            }),
        );
        let transaction = pending_input(approval_leg, TransactionType::TokenApproval, "0xhash", 0, 2)
            .pending_transaction()
            .unwrap()
            .unwrap();
        assert_eq!(transaction.to, "0xspender");
        assert_eq!(transaction.asset_id, AssetId::from(Chain::Ethereum, Some("0xusdc".into())));
        assert!(transaction.metadata.is_none());

        let generic = GemTransactionInputType::Generic {
            asset: asset(Chain::Solana),
            metadata: primitives::ApplicationMetadata {
                name: "merchant".into(),
                description: String::new(),
                url: "https://merchant.example".into(),
                icon: String::new(),
                source: ApplicationMetadataSource::Payment,
            },
            extra: crate::models::transaction::GemTransferDataExtra {
                to: String::new(),
                gas_limit: None,
                gas_price: None,
                data: Some(b"encoded".to_vec()),
                output_type: TransferDataOutputType::EncodedTransaction,
                output_action: TransferDataOutputAction::Send,
                transaction_type: TransactionType::Transfer,
                approval: None,
            },
        };
        let mut generic_input = pending_input(generic, TransactionType::Transfer, "hash", 0, 1);
        generic_input.simulation = Some(primitives::SimulationResult {
            warnings: vec![],
            balance_changes: vec![],
            payload: vec![],
            header: Some(primitives::SimulationHeader {
                asset_id: AssetId::from(Chain::Solana, Some("usdc".into())),
                value: Some(num_bigint::BigUint::from(19_000_000u32)),
                is_unlimited: false,
            }),
        });
        let transaction = generic_input.pending_transaction().unwrap().unwrap();
        assert_eq!(transaction.asset_id, AssetId::from(Chain::Solana, Some("usdc".into())));
        assert_eq!(transaction.value, "19000000");
        assert_eq!(transaction.to, "recipient");

        let hypercore_swap = swap_input(asset(Chain::Ethereum), asset(Chain::HyperCore), SwapProvider::Hyperliquid, None);
        assert!(
            pending_input(hypercore_swap.clone(), TransactionType::Swap, "0xhash", 0, 2)
                .pending_transaction()
                .unwrap()
                .is_some()
        );
        let hypercore_stake = GemTransactionInputType::Stake {
            asset: asset(Chain::HyperCore),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert!(
            pending_input(hypercore_stake.clone(), TransactionType::StakeRewards, "h", 0, 2)
                .pending_transaction()
                .unwrap()
                .is_none()
        );
        assert!(
            pending_input(hypercore_stake, TransactionType::StakeRewards, "h", 1, 2)
                .pending_transaction()
                .unwrap()
                .is_some()
        );
        assert!(
            pending_input(
                GemTransactionInputType::Transfer { asset: asset(Chain::Ethereum) },
                TransactionType::PerpetualModifyPosition,
                "h",
                0,
                1
            )
            .pending_transaction()
            .unwrap()
            .is_none()
        );
    }
}
