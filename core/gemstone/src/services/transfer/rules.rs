use chrono::Utc;
use num_bigint::BigInt;
use primitives::SwapProvider;
use primitives::swap::{ApprovalData, SwapQuoteDataType};
use primitives::{
    AccountDataType, AddressName, Asset, AssetId, AssetType, Chain, ContractCallData, DelegationValidator, EarnType, FeePriority, PaymentVerification, PerpetualType, RecentActivityType, StakeType, Transaction, TransactionDirection,
    TransactionInputType, TransactionNFTTransferMetadata, TransactionPaymentMetadata, TransactionPerpetualMetadata, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionType,
    TransactionWalletConnectMetadata, TransferDataOutputAction, TransferDataOutputType,
};

use super::model::{GemConfirmDestination, GemConfirmRow, GemConfirmTitle, GemPendingTransactionInput, GemRecentActivity, GemRecipient, GemTransferData, GemTransferOutput};
use crate::config::chain::is_memo_supported;
use crate::models::transaction::{GemTransactionLoadInput, transaction_metadata_block_number, transaction_metadata_sequence};
use crate::services::amount::model::GemAmountError;
use crate::services::assets::rules as asset_rules;
use crate::services::balance::GemAssetBalance;
use crate::services::transactions::GemTransactionHeaderKind;

pub(crate) trait TransferInput {
    fn input_asset(&self) -> Asset;
    fn transaction_asset(&self) -> Asset;
    fn balance_asset(&self) -> Asset;
    fn header_kind(&self) -> GemTransactionHeaderKind;
    fn title(&self) -> GemConfirmTitle;
    fn shows_memo(&self) -> bool;
    fn fee_asset(&self) -> Asset;
    fn default_fee_priority(&self) -> FeePriority;
    fn application_short_name(&self) -> Option<String>;
    fn output(&self) -> GemTransferOutput;
    fn asset_ids(&self) -> Vec<AssetId>;
    fn recent_activity(&self) -> Option<GemRecentActivity>;
    fn approval(&self, transaction_type: TransactionType) -> Result<Option<ApprovalData>, String>;
    fn metadata(&self) -> Result<Option<serde_json::Value>, serde_json::Error>;
    fn is_tracked(&self, transaction_type: &TransactionType, hash: &str, index: u32, count: u32) -> bool;
}

#[uniffi::export]
impl GemTransferData {
    pub fn transaction_type(&self) -> TransactionType {
        self.input_type.transaction_type()
    }

    pub fn input_asset(&self) -> Asset {
        self.input_type.input_asset()
    }

    pub fn title(&self) -> GemConfirmTitle {
        self.input_type.title()
    }

    pub fn verification(&self) -> Option<PaymentVerification> {
        let TransactionInputType::Payment { invoice, .. } = &self.input_type else {
            return None;
        };
        invoice.verification.clone()
    }

    pub fn fee_asset(&self) -> Asset {
        self.input_type.fee_asset()
    }
}

impl GemTransferData {
    pub fn header_kind(&self) -> GemTransactionHeaderKind {
        self.input_type.header_kind()
    }

    pub fn default_fee_priority(&self) -> FeePriority {
        self.input_type.default_fee_priority()
    }
}

#[uniffi::export]
impl GemTransactionLoadInput {
    pub fn chain(&self) -> Chain {
        self.input_type.transaction_asset().chain()
    }
}

impl TransferInput for TransactionInputType {
    fn input_asset(&self) -> Asset {
        self.get_asset().clone()
    }

    fn transaction_asset(&self) -> Asset {
        match self {
            Self::TransferNft { asset, .. } => Asset::from_chain(asset.chain()),
            _ => self.get_asset().clone(),
        }
    }

    fn balance_asset(&self) -> Asset {
        match self {
            Self::Perpetual { perpetual_type, .. } => perpetual_type.base_asset().clone(),
            _ => self.transaction_asset(),
        }
    }

    fn header_kind(&self) -> GemTransactionHeaderKind {
        match self {
            Self::Transfer { .. } | Self::Deposit { .. } | Self::Withdrawal { .. } | Self::Stake { .. } | Self::Earn { .. } | Self::Generic { .. } => GemTransactionHeaderKind::Amount { shows_fiat: true },
            Self::Payment { .. } => GemTransactionHeaderKind::Amount { shows_fiat: true },
            Self::Account { .. } => GemTransactionHeaderKind::Amount { shows_fiat: false },
            Self::TokenApprove { .. } => GemTransactionHeaderKind::AssetImage,
            Self::TransferNft { .. } => GemTransactionHeaderKind::Nft,
            Self::Swap { .. } => GemTransactionHeaderKind::Swap,
            Self::Perpetual { .. } => GemTransactionHeaderKind::Symbol,
        }
    }

    fn title(&self) -> GemConfirmTitle {
        match self {
            Self::Transfer { .. } | Self::TransferNft { .. } => GemConfirmTitle::Send,
            Self::Deposit { .. } => GemConfirmTitle::Deposit,
            Self::Withdrawal { .. } => GemConfirmTitle::Withdraw,
            Self::Swap { .. } => GemConfirmTitle::Swap,
            Self::TokenApprove { .. } => GemConfirmTitle::Approve,
            Self::Generic { .. } => GemConfirmTitle::Request,
            Self::Payment { .. } => GemConfirmTitle::Payment,
            Self::Account { account_type: AccountDataType::Activate, .. } => GemConfirmTitle::ActivateAsset,
            Self::Stake { stake_type, .. } => match stake_type {
                StakeType::Stake(_) => GemConfirmTitle::Stake,
                StakeType::Unstake(_) => GemConfirmTitle::Unstake,
                StakeType::Redelegate(_) => GemConfirmTitle::Redelegate,
                StakeType::Rewards(_) => GemConfirmTitle::ClaimRewards,
                StakeType::Withdraw(_) => GemConfirmTitle::Withdraw,
                StakeType::Freeze(_) => GemConfirmTitle::Freeze,
                StakeType::Unfreeze(_) => GemConfirmTitle::Unfreeze,
            },
            Self::Earn { earn_type, .. } => match earn_type {
                EarnType::Deposit(_) => GemConfirmTitle::Deposit,
                EarnType::Withdraw(_) => GemConfirmTitle::Withdraw,
            },
            Self::Perpetual { perpetual_type, .. } => match perpetual_type {
                PerpetualType::Open { data } => GemConfirmTitle::PerpetualOpen { direction: data.direction.clone() },
                PerpetualType::Increase { data } => GemConfirmTitle::PerpetualIncrease { direction: data.direction.clone() },
                PerpetualType::Reduce { data } => GemConfirmTitle::PerpetualReduce { direction: data.position_direction.clone() },
                PerpetualType::Close { .. } => GemConfirmTitle::PerpetualClose,
                PerpetualType::Modify { .. } => GemConfirmTitle::PerpetualModify,
            },
        }
    }

    fn shows_memo(&self) -> bool {
        match self {
            Self::Transfer { .. } | Self::Deposit { .. } | Self::Withdrawal { .. } => is_memo_supported(self.get_asset().chain()),
            _ => false,
        }
    }

    fn fee_asset(&self) -> Asset {
        if let Self::Perpetual { perpetual_type, .. } = self {
            return perpetual_type.base_asset().clone();
        }
        let asset = self.transaction_asset();
        let chain = asset.chain();
        match chain {
            Chain::Tempo => asset,
            Chain::HyperCore => asset_rules::default_asset(chain, AssetType::TOKEN).unwrap_or(asset),
            _ if asset.id.is_token() => Asset::from_chain(chain),
            _ => asset,
        }
    }

    fn default_fee_priority(&self) -> FeePriority {
        match self {
            Self::Swap { from_asset, .. } if from_asset.chain() == Chain::Bitcoin => FeePriority::Fast,
            Self::Transfer { .. }
            | Self::Deposit { .. }
            | Self::Swap { .. }
            | Self::Stake { .. }
            | Self::TokenApprove { .. }
            | Self::Generic { .. }
            | Self::Payment { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => FeePriority::Normal,
        }
    }

    fn application_short_name(&self) -> Option<String> {
        match self {
            Self::Generic { metadata, .. } => Some(metadata.short_name()),
            Self::Payment { invoice, .. } => Some(invoice.merchant.name.clone()),
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

    fn output(&self) -> GemTransferOutput {
        match self {
            Self::Generic { extra, .. } | Self::Payment { extra, .. } => GemTransferOutput {
                output_type: extra.output_type.clone(),
                output_action: extra.output_action.clone(),
            },
            _ => GemTransferOutput {
                output_type: TransferDataOutputType::EncodedTransaction,
                output_action: TransferDataOutputAction::Send,
            },
        }
    }

    fn asset_ids(&self) -> Vec<AssetId> {
        match self {
            Self::Swap { from_asset, to_asset, .. } => vec![from_asset.id.clone(), to_asset.id.clone()],
            Self::TransferNft { .. } => vec![],
            _ => vec![self.get_asset().id.clone()],
        }
    }
    fn recent_activity(&self) -> Option<GemRecentActivity> {
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
            | Self::Payment { .. }
            | Self::TransferNft { .. }
            | Self::Account { .. }
            | Self::Perpetual { .. }
            | Self::Earn { .. }
            | Self::Withdrawal { .. } => None,
        }
    }

    fn approval(&self, transaction_type: TransactionType) -> Result<Option<ApprovalData>, String> {
        if transaction_type != TransactionType::TokenApproval {
            return Ok(None);
        }
        match self {
            Self::Swap { swap_data, .. } => swap_data.data.approval.clone().map(Some).ok_or("Missing swap approval data".to_string()),
            Self::Earn { data, .. } => data.approval.clone().map(Some).ok_or("Missing earn approval data".to_string()),
            Self::TokenApprove { approval_data, .. } => Ok(Some(approval_data.clone())),
            Self::Generic { extra, .. } | Self::Payment { extra, .. } => Ok(extra.approval.clone()),
            Self::Transfer { .. } | Self::Deposit { .. } | Self::Stake { .. } | Self::TransferNft { .. } | Self::Account { .. } | Self::Perpetual { .. } | Self::Withdrawal { .. } => {
                Err("Token approval transaction type does not match transfer data".to_string())
            }
        }
    }

    fn metadata(&self) -> Result<Option<serde_json::Value>, serde_json::Error> {
        let value = match self {
            Self::Swap { from_asset, to_asset, swap_data } => Some(serde_json::to_value(TransactionSwapMetadata {
                from_asset: from_asset.id.clone(),
                from_value: swap_data.quote.from_value.clone(),
                to_asset: to_asset.id.clone(),
                to_value: swap_data.quote.to_value.clone(),
                provider: Some(swap_data.quote.provider_data.provider.as_ref().to_string()),
            })?),
            Self::TransferNft { nft_asset, .. } => Some(serde_json::to_value(TransactionNFTTransferMetadata::new(nft_asset.id.clone(), Some(nft_asset.name.clone())))?),
            Self::Perpetual { perpetual_type, .. } => match perpetual_type {
                PerpetualType::Open { data } | PerpetualType::Close { data } | PerpetualType::Increase { data } => Some(serde_json::to_value(TransactionPerpetualMetadata {
                    pnl: 0.0,
                    price: 0.0,
                    direction: data.direction.clone(),
                    is_liquidation: None,
                    provider: None,
                })?),
                PerpetualType::Reduce { data } => Some(serde_json::to_value(TransactionPerpetualMetadata {
                    pnl: 0.0,
                    price: 0.0,
                    direction: data.position_direction.clone(),
                    is_liquidation: None,
                    provider: None,
                })?),
                PerpetualType::Modify { .. } => None,
            },
            Self::Stake { stake_type, .. } => match stake_type {
                StakeType::Freeze(data) | StakeType::Unfreeze(data) => Some(serde_json::to_value(TransactionResourceTypeMetadata::new(*data))?),
                StakeType::Stake(_) | StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => None,
            },
            Self::Generic { extra, .. } => Some(serde_json::to_value(TransactionWalletConnectMetadata { output_action: extra.output_action.clone() })?),
            Self::Payment { invoice, .. } => Some(serde_json::to_value(TransactionPaymentMetadata {
                link: invoice.link.clone(),
                merchant: invoice.merchant.clone(),
            })?),
            Self::Transfer { .. } | Self::Deposit { .. } | Self::Withdrawal { .. } | Self::TokenApprove { .. } | Self::Account { .. } | Self::Earn { .. } => None,
        };
        Ok(value)
    }

    fn is_tracked(&self, transaction_type: &TransactionType, hash: &str, index: u32, count: u32) -> bool {
        if *transaction_type == TransactionType::PerpetualModifyPosition {
            return false;
        }
        if self.get_asset().chain() != Chain::HyperCore {
            return true;
        }
        let is_intermediate = index + 1 < count;
        match self {
            Self::Stake { .. } => !is_intermediate,
            Self::Perpetual { .. } => hash.starts_with(HYPERCORE_ORDER_PREFIX),
            Self::Swap { to_asset, swap_data, .. } => !(to_asset.chain() == Chain::HyperCore && swap_data.quote.provider_data.provider == SwapProvider::Hyperliquid && is_intermediate),
            Self::Transfer { .. } | Self::Deposit { .. } | Self::TokenApprove { .. } | Self::Generic { .. } | Self::Payment { .. } | Self::TransferNft { .. } | Self::Account { .. } | Self::Earn { .. } | Self::Withdrawal { .. } => true,
        }
    }
}

pub(crate) fn tron_stake_available(asset: &Asset, balance: &GemAssetBalance) -> BigInt {
    let staked = BigInt::from(balance.votes()) * BigInt::from(10u32).pow(asset.decimals.max(0) as u32);
    (BigInt::from(&balance.frozen + &balance.locked) - staked).max(BigInt::from(0))
}

pub(crate) fn unfreeze_available(resource: &primitives::Resource, balance: &GemAssetBalance) -> BigInt {
    match resource {
        primitives::Resource::Bandwidth => BigInt::from(balance.frozen.clone()),
        primitives::Resource::Energy => BigInt::from(balance.locked.clone()),
    }
}

impl GemConfirmDestination {
    pub fn with_address_name(&self, address_name: Option<AddressName>) -> GemConfirmDestination {
        let named = address_name.filter(|address_name| !address_name.name.is_empty());
        match (self, named) {
            (Self::Recipient { address, .. }, Some(address_name)) => Self::Recipient {
                name: Some(address_name.name),
                address: address.clone(),
            },
            (Self::Contract { address, .. }, Some(address_name)) => Self::Contract {
                name: Some(address_name.name),
                address: address.clone(),
            },
            _ => self.clone(),
        }
    }

    pub fn address(&self) -> String {
        match self {
            Self::Recipient { address, .. } | Self::Contract { address, .. } | Self::Validator { address, .. } | Self::Provider { address, .. } => address.clone(),
            Self::Resource { .. } => String::new(),
        }
    }

    pub fn shows_address_beside_name(&self) -> bool {
        match self {
            Self::Recipient { .. } | Self::Contract { .. } => true,
            Self::Validator { .. } | Self::Provider { .. } | Self::Resource { .. } => false,
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            Self::Recipient { name, .. } | Self::Contract { name, .. } => name.clone(),
            Self::Validator { name, .. } | Self::Provider { name, .. } => Some(name.clone()),
            Self::Resource { .. } => None,
        }
    }
}

#[uniffi::export]
impl GemTransferData {
    pub fn destination(&self) -> Option<GemConfirmDestination> {
        let recipient = || {
            (!self.recipient.address.is_empty()).then(|| GemConfirmDestination::Recipient {
                name: self.recipient.name.clone(),
                address: self.recipient.address.clone(),
            })
        };
        let validator = |validator: &DelegationValidator| GemConfirmDestination::Validator {
            name: validator.name.clone(),
            address: validator.id.clone(),
        };
        match &self.input_type {
            TransactionInputType::Transfer { .. } | TransactionInputType::TransferNft { .. } | TransactionInputType::Deposit { .. } | TransactionInputType::Withdrawal { .. } => recipient(),
            TransactionInputType::TokenApprove { .. } => Some(GemConfirmDestination::Contract {
                name: None,
                address: self.recipient.address.clone(),
            }),
            TransactionInputType::Generic { extra, .. } => match extra.output_action {
                TransferDataOutputAction::Send => recipient(),
                TransferDataOutputAction::Sign => None,
            },
            TransactionInputType::Payment { invoice, .. } => Some(GemConfirmDestination::Recipient {
                name: Some(invoice.merchant.name.clone()),
                address: self.recipient.address.clone(),
            }),
            TransactionInputType::Stake { stake_type, .. } => match stake_type {
                StakeType::Stake(target) => Some(validator(target)),
                StakeType::Redelegate(data) => Some(validator(&data.to_validator)),
                StakeType::Unstake(delegation) | StakeType::Withdraw(delegation) => Some(validator(&delegation.validator)),
                StakeType::Rewards(validators) => match validators.as_slice() {
                    [target] => Some(validator(target)),
                    _ => None,
                },
                StakeType::Freeze(resource) | StakeType::Unfreeze(resource) => Some(GemConfirmDestination::Resource { resource: *resource }),
            },
            TransactionInputType::Earn { earn_type, .. } => {
                let provider = match earn_type {
                    EarnType::Deposit(provider) => provider,
                    EarnType::Withdraw(delegation) => &delegation.validator,
                };
                Some(GemConfirmDestination::Provider {
                    name: provider.name.clone(),
                    address: provider.id.clone(),
                })
            }
            TransactionInputType::Swap { .. } | TransactionInputType::Account { .. } | TransactionInputType::Perpetual { .. } => None,
        }
    }
}

pub fn stake_transfer_data(asset: Asset, stake_type: StakeType, value: BigInt, use_max_amount: bool) -> GemTransferData {
    let recipient = match &stake_type {
        StakeType::Stake(validator) => GemRecipient::named(validator.id.clone(), validator.name.clone()),
        StakeType::Redelegate(data) => GemRecipient::named(data.to_validator.id.clone(), data.to_validator.name.clone()),
        StakeType::Unstake(delegation) | StakeType::Withdraw(delegation) => GemRecipient::named(delegation.validator.id.clone(), delegation.validator.name.clone()),
        StakeType::Rewards(validators) => match validators.first() {
            Some(validator) => GemRecipient::named(validator.id.clone(), validator.name.clone()),
            None => GemRecipient::address(String::new()),
        },
        StakeType::Freeze(resource) | StakeType::Unfreeze(resource) => GemRecipient::address(resource.as_ref().to_string()),
    };
    let use_max_amount = use_max_amount && matches!(stake_type, StakeType::Stake(_) | StakeType::Freeze(_));
    GemTransferData {
        input_type: TransactionInputType::Stake { asset, stake_type },
        recipient,
        value,
        use_max_amount,
    }
}

pub fn activate_asset_transfer_data(asset: Asset) -> GemTransferData {
    GemTransferData {
        input_type: TransactionInputType::Account {
            asset,
            account_type: AccountDataType::Activate,
        },
        recipient: GemRecipient::address(String::new()),
        value: BigInt::from(0),
        use_max_amount: false,
    }
}

pub fn earn_transfer_data(asset: Asset, earn_type: EarnType, data: ContractCallData, value: BigInt, use_max_amount: bool) -> GemTransferData {
    let provider = match &earn_type {
        EarnType::Deposit(provider) => provider,
        EarnType::Withdraw(delegation) => &delegation.validator,
    };
    GemTransferData {
        recipient: GemRecipient::named(data.contract_address.clone(), provider.name.clone()),
        input_type: TransactionInputType::Earn { asset, earn_type, data },
        value,
        use_max_amount,
    }
}

impl GemTransferData {
    pub fn shows_memo(&self) -> bool {
        self.input_type.shows_memo()
    }

    pub fn confirm_rows(&self) -> Vec<GemConfirmRow> {
        if let TransactionInputType::Payment { invoice, .. } = &self.input_type {
            return [
                Some(GemConfirmRow::Recipient),
                Some(GemConfirmRow::Sender),
                Some(GemConfirmRow::Network),
                (!invoice.quotes.is_empty()).then_some(GemConfirmRow::PaymentAsset),
            ]
            .into_iter()
            .flatten()
            .collect();
        }
        let is_generic = matches!(self.input_type, TransactionInputType::Generic { .. });
        [
            self.input_type.application_short_name().is_some().then_some(GemConfirmRow::App),
            Some(GemConfirmRow::Sender),
            (!is_generic).then_some(GemConfirmRow::Recipient),
            Some(GemConfirmRow::Network),
            self.shows_memo().then_some(GemConfirmRow::Memo),
            (!is_generic).then_some(GemConfirmRow::Details),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn application_short_name(&self) -> Option<String> {
        self.input_type.application_short_name()
    }

    #[allow(clippy::result_large_err)]
    pub(crate) fn available_value(&self, balance: &GemAssetBalance) -> Result<BigInt, GemAmountError> {
        let asset = self.input_type.get_asset();
        Ok(match &self.input_type {
            TransactionInputType::Withdrawal { .. } => BigInt::from(balance.withdrawable.clone()),
            TransactionInputType::Stake { stake_type, .. } => match stake_type {
                StakeType::Unstake(delegation) | StakeType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
                StakeType::Redelegate(data) => BigInt::from(data.delegation.base.balance.clone()),
                StakeType::Rewards(_) => self.value.clone(),
                StakeType::Unfreeze(resource) => unfreeze_available(resource, balance),
                StakeType::Stake(_) if asset.chain() == Chain::Tron => tron_stake_available(asset, balance),
                StakeType::Stake(_) | StakeType::Freeze(_) => BigInt::from(balance.available.clone()),
            },
            TransactionInputType::Earn { earn_type, .. } => match earn_type {
                EarnType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
                EarnType::Deposit(_) => BigInt::from(balance.available.clone()),
            },
            TransactionInputType::Transfer { .. }
            | TransactionInputType::Deposit { .. }
            | TransactionInputType::Swap { .. }
            | TransactionInputType::TokenApprove { .. }
            | TransactionInputType::Generic { .. }
            | TransactionInputType::Payment { .. }
            | TransactionInputType::TransferNft { .. }
            | TransactionInputType::Account { .. }
            | TransactionInputType::Perpetual { .. } => BigInt::from(balance.available.clone()),
        })
    }
}

impl GemPendingTransactionInput {
    pub(crate) fn pending_transaction(self) -> Result<Option<Transaction>, String> {
        if !self.transfer.input_type.is_tracked(&self.transaction_type, &self.hash, self.transaction_index, self.transaction_count) {
            return Ok(None);
        }
        let transfer = self.transfer;
        let chain = transfer.input_type.get_asset().chain();
        let approval = transfer.input_type.approval(self.transaction_type.clone())?;
        let simulation_header = match transfer.input_type {
            TransactionInputType::Generic { .. } | TransactionInputType::Payment { .. } => self.simulation.and_then(|simulation| simulation.header),
            _ => None,
        };
        let transfer_value = self.value.to_biguint().ok_or_else(|| "negative transfer value".to_string())?;
        let (recipient, value, memo) = match &approval {
            Some(approval) => (approval.spender.clone(), approval.value.clone(), None),
            None => {
                let recipient = match &transfer.input_type {
                    TransactionInputType::Swap { swap_data, .. } => swap_data.data.to.clone(),
                    _ => transfer.recipient.address.clone(),
                };
                let value = simulation_header.as_ref().and_then(|header| header.value.clone()).unwrap_or(transfer_value);
                let memo = match &transfer.input_type {
                    TransactionInputType::Swap { .. } => None,
                    _ => transfer.recipient.memo.clone().filter(|memo| !memo.is_empty()),
                };
                (recipient, value, memo)
            }
        };
        let asset_id = simulation_header
            .as_ref()
            .map(|header| header.asset_id.clone())
            .or_else(|| approval.as_ref().map(|approval| AssetId::from(chain, Some(approval.token.clone()))))
            .unwrap_or_else(|| transfer.input_type.transaction_asset().id);
        let direction = if self.sender == recipient { TransactionDirection::SelfTransfer } else { TransactionDirection::Outgoing };
        let contract = match (&transfer.input_type, &self.transaction_type) {
            (TransactionInputType::Swap { swap_data, .. }, TransactionType::Swap) if swap_data.data.data_type == SwapQuoteDataType::Contract => Some(swap_data.data.to.clone()).filter(|contract| !contract.is_empty()),
            _ => None,
        };
        let metadata = match transfer.input_type {
            TransactionInputType::Swap { .. } | TransactionInputType::Earn { .. } | TransactionInputType::Payment { .. } if approval.is_some() => None,
            _ => transfer.input_type.metadata().map_err(|error| error.to_string())?,
        };
        let mut transaction = Transaction::new(
            self.hash,
            asset_id,
            self.sender,
            recipient,
            contract,
            self.transaction_type,
            TransactionState::Pending,
            self.network_fee.to_biguint().ok_or_else(|| "negative network fee".to_string())?,
            self.fee.fee_asset,
            value,
            memo,
            metadata,
            Utc::now(),
        );
        transaction.block_number = transaction_metadata_block_number(&self.metadata);
        transaction.sequence = transaction_metadata_sequence(&self.metadata);
        transaction.direction = direction;
        Ok(Some(transaction))
    }
}

const HYPERCORE_ORDER_PREFIX: &str = "order:";

#[cfg(test)]
mod tests {
    #[test]
    fn test_only_a_transfer_and_a_swap_leave_recent_activity() {
        let from = Asset::from_chain(Chain::Ethereum);
        let to = Asset::from_chain(Chain::Solana);

        let transfer = (TransactionInputType::Transfer { asset: from.clone() }).recent_activity().unwrap();
        assert_eq!(transfer.activity_type, RecentActivityType::Transfer);
        assert_eq!(transfer.asset_id, from.id);
        assert_eq!(transfer.to_asset_id, None);

        let swap = (TransactionInputType::Swap {
            from_asset: from.clone(),
            to_asset: to.clone(),
            swap_data: SwapData::mock(),
        })
        .recent_activity()
        .unwrap();
        assert_eq!(swap.activity_type, RecentActivityType::Swap);
        assert_eq!(swap.asset_id, from.id);
        assert_eq!(swap.to_asset_id, Some(to.id));

        assert!((TransactionInputType::Deposit { asset: from.clone() }).recent_activity().is_none());
        assert!((TransactionInputType::Withdrawal { asset: from }).recent_activity().is_none());
    }

    use super::*;
    use num_bigint::BigUint;
    use primitives::asset_balance::BalanceMetadata;
    use primitives::{
        Delegation, DelegationBase, DelegationValidator, NFTAsset, PaymentInvoice, PaymentMerchant, PaymentPrice, PerpetualConfirmData, PerpetualDirection, PerpetualModifyConfirmData, PerpetualReduceData, Resource, SwapProvider,
        TransactionType, TransferDataExtra,
        known_assets::HYPERCORE_PERPETUAL_USDC,
        swap::{SwapData, SwapQuote, SwapQuoteData},
    };

    #[test]
    fn test_swap_rules() {
        let input = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Ethereum),
            to_asset: Asset::from_chain(Chain::Bitcoin),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Thorchain, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: None,
                    ..SwapQuoteData::mock()
                },
            },
        };
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
    fn test_header_kind_by_input_type() {
        assert_eq!(
            TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }.header_kind(),
            GemTransactionHeaderKind::Amount { shows_fiat: true }
        );
        let payment = |price| TransactionInputType::Payment {
            asset: Asset::from_chain(Chain::Ethereum),
            invoice: PaymentInvoice { price, ..PaymentInvoice::mock() },
            extra: TransferDataExtra::mock(),
        };
        assert_eq!(payment(Some(PaymentPrice::mock())).header_kind(), GemTransactionHeaderKind::Amount { shows_fiat: true });
        assert_eq!(payment(None).header_kind(), GemTransactionHeaderKind::Amount { shows_fiat: true });
        assert_eq!(
            TransactionInputType::TokenApprove {
                asset: Asset::from_chain(Chain::Ethereum),
                approval_data: primitives::swap::ApprovalData::mock(),
            }
            .header_kind(),
            GemTransactionHeaderKind::AssetImage
        );
        assert_eq!(
            TransactionInputType::Perpetual {
                asset: Asset::from_chain(Chain::HyperCore),
                perpetual_type: PerpetualType::Open {
                    data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
                },
            }
            .header_kind(),
            GemTransactionHeaderKind::Symbol
        );
    }

    #[test]
    fn test_balance_asset_is_the_perpetual_collateral_not_the_market() {
        let market = Asset {
            id: AssetId::from_token(Chain::HyperCore, "perpetual::ETH"),
            ..Asset::from_chain(Chain::HyperCore)
        };
        let collateral = HYPERCORE_PERPETUAL_USDC.clone();
        let data = PerpetualConfirmData {
            base_asset: collateral.clone(),
            ..PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)
        };
        let open = TransactionInputType::Perpetual {
            asset: market.clone(),
            perpetual_type: PerpetualType::Open { data: data.clone() },
        };
        let modify = TransactionInputType::Perpetual {
            asset: market.clone(),
            perpetual_type: PerpetualType::Modify {
                data: PerpetualModifyConfirmData {
                    base_asset: collateral.clone(),
                    ..PerpetualModifyConfirmData::mock(vec![], None, None)
                },
            },
        };
        let reduce = TransactionInputType::Perpetual {
            asset: market.clone(),
            perpetual_type: PerpetualType::Reduce {
                data: PerpetualReduceData {
                    data,
                    position_direction: PerpetualDirection::Long,
                },
            },
        };

        for input_type in [&open, &modify, &reduce] {
            assert_eq!(input_type.transaction_asset().id, market.id, "the market asset stays the transaction's own asset");
            assert_eq!(
                input_type.balance_asset().id,
                collateral.id,
                "a perpetual is funded from the collateral balance; the market asset never has a stored balance row"
            );
            assert_eq!(input_type.balance_asset().id, input_type.fee_asset().id);
        }
    }

    #[test]
    fn test_balance_asset_follows_the_transaction_asset_outside_perpetuals() {
        let nft = TransactionInputType::TransferNft {
            asset: Asset::from_chain(Chain::Ethereum),
            nft_asset: NFTAsset::mock(),
        };
        assert_eq!(nft.balance_asset().id, nft.transaction_asset().id);

        let transfer = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) };
        assert_eq!(transfer.balance_asset().id, transfer.transaction_asset().id);
    }

    #[test]
    fn test_title_by_input_type() {
        assert_eq!(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }.title(), GemConfirmTitle::Send);
        assert_eq!(TransactionInputType::Deposit { asset: Asset::from_chain(Chain::HyperCore) }.title(), GemConfirmTitle::Deposit);
        assert_eq!(TransactionInputType::Withdrawal { asset: Asset::from_chain(Chain::HyperCore) }.title(), GemConfirmTitle::Withdraw);
        assert_eq!(TransactionInputType::mock_payment(Asset::from_chain(Chain::Ethereum), TransferDataExtra::mock()).title(), GemConfirmTitle::Payment);
        assert_eq!(
            TransactionInputType::TokenApprove {
                asset: Asset::from_chain(Chain::Ethereum),
                approval_data: primitives::swap::ApprovalData::mock(),
            }
            .title(),
            GemConfirmTitle::Approve
        );
        assert_eq!(
            TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Cosmos),
                stake_type: StakeType::Rewards(vec![]),
            }
            .title(),
            GemConfirmTitle::ClaimRewards
        );
        assert_eq!(
            TransactionInputType::Perpetual {
                asset: Asset::from_chain(Chain::HyperCore),
                perpetual_type: PerpetualType::Open {
                    data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
                },
            }
            .title(),
            GemConfirmTitle::PerpetualOpen { direction: PerpetualDirection::Long }
        );
    }

    #[test]
    fn test_stake_transfer_data_names_the_validator_and_keeps_max_for_new_stake_only() {
        let validator = primitives::DelegationValidator::mock();
        let staked = stake_transfer_data(Asset::from_chain(Chain::Cosmos), StakeType::Stake(validator.clone()), BigInt::from(5), true);
        assert_eq!(staked.recipient.address, validator.id);
        assert_eq!(staked.recipient.name.as_deref(), Some(validator.name.as_str()));
        assert!(staked.use_max_amount);
        let unstaked = stake_transfer_data(Asset::from_chain(Chain::Cosmos), StakeType::Unstake(primitives::Delegation::mock()), BigInt::from(5), true);
        assert!(!unstaked.use_max_amount);
        assert_eq!(unstaked.value, BigInt::from(5));
    }

    #[test]
    fn test_earn_transfer_data_sends_to_the_contract_under_the_provider_name() {
        let provider = primitives::DelegationValidator::mock();
        let data = ContractCallData {
            contract_address: "0xvault".to_string(),
            call_data: "0x".to_string(),
            approval: None,
            gas_limit: None,
        };
        let transfer = earn_transfer_data(Asset::from_chain(Chain::Ethereum), EarnType::Deposit(provider.clone()), data, BigInt::from(1), false);
        assert_eq!(transfer.recipient.address, "0xvault");
        assert_eq!(transfer.recipient.name.as_deref(), Some(provider.name.as_str()));
        assert!(matches!(transfer.input_type, TransactionInputType::Earn { .. }));
    }

    #[test]
    fn test_a_generic_request_shows_the_app_instead_of_a_recipient() {
        let send = GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Cosmos) });
        assert_eq!(send.confirm_rows(), vec![GemConfirmRow::Sender, GemConfirmRow::Recipient, GemConfirmRow::Network, GemConfirmRow::Memo, GemConfirmRow::Details]);

        let ethereum = GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) });
        assert!(!ethereum.confirm_rows().contains(&GemConfirmRow::Memo), "a chain without memos has no memo row");

        let generic = GemTransferData::mock(TransactionInputType::Generic {
            asset: Asset::from_chain(Chain::Ethereum),
            metadata: primitives::ApplicationMetadata {
                name: "App".into(),
                description: String::new(),
                url: String::new(),
                icon: String::new(),
                source: primitives::ApplicationMetadataSource::WalletConnect,
            },
            extra: primitives::TransferDataExtra::default(),
        });
        assert_eq!(generic.confirm_rows(), vec![GemConfirmRow::App, GemConfirmRow::Sender, GemConfirmRow::Network]);

        let quoted = GemTransferData::mock(TransactionInputType::mock_payment(Asset::from_chain(Chain::Ethereum), TransferDataExtra::mock()));
        assert_eq!(quoted.confirm_rows(), vec![GemConfirmRow::Recipient, GemConfirmRow::Sender, GemConfirmRow::Network, GemConfirmRow::PaymentAsset]);
        let solana_pay = GemTransferData::mock(TransactionInputType::Payment {
            asset: Asset::from_chain(Chain::Solana),
            invoice: PaymentInvoice { quotes: vec![], ..PaymentInvoice::mock() },
            extra: TransferDataExtra::mock(),
        });
        assert_eq!(
            solana_pay.confirm_rows(),
            vec![GemConfirmRow::Recipient, GemConfirmRow::Sender, GemConfirmRow::Network],
            "a rail without quotes offers nothing to pay with"
        );
    }

    #[test]
    fn test_memo_row_only_for_sends_on_memo_chains() {
        assert!(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Cosmos) }.shows_memo());
        assert!(!TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }.shows_memo());
        assert!(
            !TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Cosmos),
                stake_type: StakeType::Rewards(vec![]),
            }
            .shows_memo()
        );
    }

    #[test]
    fn test_fee_asset_for_tokens_and_hypercore() {
        let token = Asset::mock_with_params(Chain::Ethereum, Some("0xusdc".to_string()), "USDC".to_string(), "USDC".to_string(), 6, AssetType::ERC20);
        let tempo = Asset::mock_with_params(Chain::Tempo, Some("0xusdc".to_string()), "USDC".to_string(), "USDC".to_string(), 6, AssetType::ERC20);
        let token_transfer = TransactionInputType::Transfer { asset: token.clone() };
        assert_eq!(token_transfer.fee_asset().id, AssetId::from_chain(Chain::Ethereum));
        let tempo_token = TransactionInputType::Transfer { asset: tempo.clone() };
        assert!(tempo_token.fee_asset().id.is_token());
        let hypercore = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::HyperCore) };
        assert_eq!(hypercore.fee_asset().asset_type, AssetType::TOKEN);
        assert_eq!(tempo_token.fee_asset().id, tempo.id);

        let perpetual = TransactionInputType::Perpetual {
            asset: Asset::from_chain(Chain::HyperCore),
            perpetual_type: PerpetualType::Open {
                data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
            },
        };
        assert_eq!(perpetual.fee_asset().id, HYPERCORE_PERPETUAL_USDC.id);
        assert_eq!(perpetual.fee_asset().asset_type, AssetType::PERPETUAL);

        let other_collateral = Asset::mock_with_params(Chain::HyperCore, Some("perpetual::EURC".to_string()), "EURC".to_string(), "EURC".to_string(), 6, AssetType::PERPETUAL);
        let other = TransactionInputType::Perpetual {
            asset: Asset::from_chain(Chain::HyperCore),
            perpetual_type: PerpetualType::Open {
                data: PerpetualConfirmData {
                    base_asset: other_collateral.clone(),
                    ..PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)
                },
            },
        };
        assert_eq!(
            other.fee_asset().id,
            other_collateral.id,
            "the fee asset is the collateral the transaction itself carries, not a chain-wide default, so it can never disagree with the balance the load reads"
        );
        let nft = TransactionInputType::TransferNft { asset: token, nft_asset: NFTAsset::mock() };
        assert_eq!(nft.fee_asset().id, AssetId::from_chain(Chain::Ethereum));
        let spl = TransactionInputType::Transfer { asset: Asset::mock_spl_token() };
        assert_eq!(spl.fee_asset().id, AssetId::from_chain(Chain::Solana));
    }

    #[test]
    fn test_a_known_address_name_replaces_the_recipient_name_only() {
        use primitives::{AddressType, VerificationStatus};
        let recipient = GemConfirmDestination::Recipient {
            name: Some("typed.eth".into()),
            address: "0x1".into(),
        };
        let known = AddressName::mock("0x1", "Vitalik", AddressType::Contact, VerificationStatus::Verified);
        let unnamed = AddressName::mock("0x1", "", AddressType::Address, VerificationStatus::Verified);
        assert_eq!(
            recipient.with_address_name(Some(known.clone())),
            GemConfirmDestination::Recipient {
                name: Some("Vitalik".into()),
                address: "0x1".into()
            }
        );
        assert_eq!(recipient.with_address_name(Some(unnamed)), recipient);
        assert_eq!(recipient.with_address_name(None), recipient);
        let validator = GemConfirmDestination::Validator {
            name: "Allnodes".into(),
            address: "validator1".into(),
        };
        assert_eq!(validator.with_address_name(Some(known)), validator);
    }

    #[test]
    fn test_destination_is_the_row_the_confirm_screen_shows() {
        let eth = Asset::from_chain(Chain::Ethereum);
        let paid = GemTransferData::mock(TransactionInputType::mock_payment(eth.clone(), TransferDataExtra::mock()));
        assert_eq!(
            paid.destination(),
            Some(GemConfirmDestination::Recipient {
                name: Some(PaymentMerchant::mock().name),
                address: "recipient".into()
            })
        );
        let sent = GemTransferData::mock(TransactionInputType::Transfer { asset: eth.clone() });
        assert_eq!(sent.destination(), Some(GemConfirmDestination::Recipient { name: None, address: "recipient".into() }));
        let mut unaddressed = sent.clone();
        unaddressed.recipient.address = String::new();
        assert_eq!(unaddressed.destination(), None);

        let signature = GemTransferData::mock(TransactionInputType::Generic {
            asset: eth.clone(),
            metadata: primitives::ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_action: TransferDataOutputAction::Sign,
                ..TransferDataExtra::mock()
            },
        });
        assert_eq!(signature.destination(), None);

        let validator = DelegationValidator::mock();
        assert_eq!(
            GemTransferData::mock(TransactionInputType::Stake {
                asset: eth.clone(),
                stake_type: StakeType::Rewards(vec![validator.clone()]),
            })
            .destination(),
            Some(GemConfirmDestination::Validator {
                name: validator.name.clone(),
                address: validator.id.clone()
            })
        );
        assert_eq!(
            GemTransferData::mock(TransactionInputType::Stake {
                asset: eth.clone(),
                stake_type: StakeType::Rewards(vec![validator.clone(), validator.clone()]),
            })
            .destination(),
            None
        );
        assert_eq!(
            GemTransferData::mock(TransactionInputType::Stake {
                asset: eth.clone(),
                stake_type: StakeType::Freeze(primitives::Resource::Energy),
            })
            .destination(),
            Some(GemConfirmDestination::Resource { resource: primitives::Resource::Energy })
        );
        assert_eq!(
            GemTransferData::mock(TransactionInputType::TokenApprove {
                asset: eth.clone(),
                approval_data: primitives::swap::ApprovalData::mock(),
            })
            .destination(),
            Some(GemConfirmDestination::Contract { name: None, address: "recipient".into() })
        );
    }

    #[test]
    fn test_output() {
        let signature = TransactionInputType::Generic {
            asset: Asset::from_chain(Chain::Ethereum),
            metadata: primitives::ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_type: TransferDataOutputType::Signature,
                output_action: TransferDataOutputAction::Sign,
                ..TransferDataExtra::mock()
            },
        };
        let signed = signature.output();
        assert_eq!(signed.output_type, TransferDataOutputType::Signature);
        assert_eq!(signed.output_action, TransferDataOutputAction::Sign);
        assert_eq!(signature.application_short_name().as_deref(), Some("Test Dapp"));

        let withdrawal = TransactionInputType::Withdrawal { asset: Asset::from_chain(Chain::HyperCore) };
        let sent = withdrawal.output();
        assert_eq!(sent.output_type, TransferDataOutputType::EncodedTransaction);
        assert_eq!(sent.output_action, TransferDataOutputAction::Send);
        assert_eq!(withdrawal.application_short_name(), None);
        assert_eq!(
            TransactionInputType::Perpetual {
                asset: Asset::from_chain(Chain::HyperCore),
                perpetual_type: PerpetualType::Open {
                    data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
                },
            }
            .output()
            .output_action,
            TransferDataOutputAction::Send
        );
    }

    #[test]
    fn test_hypercore_tracking_skips_intermediate_legs_and_non_orders() {
        let intermediate = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::HyperCore),
            to_asset: Asset::from_chain(Chain::HyperCore),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Hyperliquid, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: None,
                    ..SwapQuoteData::mock()
                },
            },
        };
        assert!(GemPendingTransactionInput::mock(intermediate.clone(), TransactionType::Swap, "h", 0, 2).pending_transaction().unwrap().is_none());
        assert!(GemPendingTransactionInput::mock(intermediate, TransactionType::Swap, "h", 1, 2).pending_transaction().unwrap().is_some());
        let other_provider = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::HyperCore),
            to_asset: Asset::from_chain(Chain::HyperCore),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Thorchain, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: None,
                    ..SwapQuoteData::mock()
                },
            },
        };
        assert!(GemPendingTransactionInput::mock(other_provider, TransactionType::Swap, "h", 0, 2).pending_transaction().unwrap().is_some());

        let perpetual = TransactionInputType::Perpetual {
            asset: Asset::from_chain(Chain::HyperCore),
            perpetual_type: PerpetualType::Open {
                data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
            },
        };
        assert!(
            GemPendingTransactionInput::mock(perpetual.clone(), TransactionType::PerpetualOpenPosition, "order:1", 0, 1)
                .pending_transaction()
                .unwrap()
                .is_some()
        );
        assert!(GemPendingTransactionInput::mock(perpetual, TransactionType::PerpetualOpenPosition, "0xabc", 0, 1).pending_transaction().unwrap().is_none());
    }

    #[test]
    fn test_stake_metadata_and_available_value() {
        let unfreeze = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Tron),
            stake_type: StakeType::Unfreeze(Resource::Bandwidth),
        };
        assert_eq!(unfreeze.metadata().unwrap().unwrap()["resourceType"], "bandwidth");
        assert_eq!(
            GemTransferData::mock(unfreeze)
                .available_value(&GemAssetBalance {
                    frozen: BigUint::from(20u64),
                    locked: BigUint::from(30u64),
                    ..GemAssetBalance::mock_with_available(10)
                })
                .unwrap(),
            BigInt::from(20)
        );

        let unstake = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Cosmos),
            stake_type: StakeType::Unstake(Delegation::mock_base(DelegationBase::mock_with_balance(700, 5))),
        };
        assert_eq!(GemTransferData::mock(unstake).available_value(&GemAssetBalance::mock_with_available(10)).unwrap(), BigInt::from(700));
        let rewards = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Cosmos),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert_eq!(
            GemTransferData {
                value: BigInt::from(42),
                ..GemTransferData::mock(rewards)
            }
            .available_value(&GemAssetBalance::mock_with_available(10))
            .unwrap(),
            BigInt::from(42)
        );
        let tron_stake = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Tron),
            stake_type: StakeType::Stake(Delegation::mock_base(DelegationBase::mock_with_balance(0, 5)).validator),
        };
        assert_eq!(
            GemTransferData::mock(tron_stake)
                .available_value(&GemAssetBalance {
                    metadata: Some(BalanceMetadata { votes: 2, ..BalanceMetadata::default() }),
                    ..GemAssetBalance {
                        frozen: BigUint::from(5000000u64),
                        locked: BigUint::from(3000000u64),
                        ..GemAssetBalance::mock_with_available(1)
                    }
                })
                .unwrap(),
            BigInt::from(6_000_000)
        );
        let overvoted = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Tron),
            stake_type: StakeType::Stake(Delegation::mock_base(DelegationBase::mock_with_balance(0, 5)).validator),
        };
        assert_eq!(
            GemTransferData::mock(overvoted)
                .available_value(&GemAssetBalance {
                    metadata: Some(BalanceMetadata { votes: 9, ..BalanceMetadata::default() }),
                    ..GemAssetBalance {
                        frozen: BigUint::from(5000000u64),
                        locked: BigUint::from(3000000u64),
                        ..GemAssetBalance::mock_with_available(1)
                    }
                })
                .unwrap(),
            BigInt::from(0)
        );
        let withdrawal = TransactionInputType::Withdrawal { asset: Asset::from_chain(Chain::HyperCore) };
        assert_eq!(
            GemTransferData::mock(withdrawal)
                .available_value(&GemAssetBalance {
                    withdrawable: BigUint::from(9u32),
                    ..GemAssetBalance::mock_with_available(10)
                })
                .unwrap(),
            BigInt::from(9)
        );
    }

    #[test]
    fn test_a_transfer_keeps_its_memo_and_reports_a_blank_one_as_absent() {
        let transfer = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Cosmos) };
        let written = GemPendingTransactionInput::mock(transfer.clone(), TransactionType::Transfer, "0xhash", 0, 1).pending_transaction().unwrap().unwrap();

        assert_eq!(written.memo.as_deref(), Some("memo"));

        let mut blank = GemPendingTransactionInput::mock(transfer, TransactionType::Transfer, "0xhash", 0, 1);
        blank.transfer.recipient.memo = Some(String::new());

        assert_eq!(blank.pending_transaction().unwrap().unwrap().memo, None, "a blank memo field is no memo, not an empty one");
    }

    #[test]
    fn test_pending_transaction_uses_swap_router_and_hypercore_tracking() {
        let swap = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Ethereum),
            to_asset: Asset::from_chain(Chain::Bitcoin),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Thorchain, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: None,
                    ..SwapQuoteData::mock()
                },
            },
        };
        let transaction = GemPendingTransactionInput::mock(swap, TransactionType::Swap, "0xhash", 0, 1).pending_transaction().unwrap().unwrap();
        assert_eq!(transaction.to, "0xrouter");
        assert_eq!(transaction.contract.as_deref(), Some("0xrouter"), "a contract swap keeps the router so the provider row can open it");
        assert_eq!(transaction.value, BigUint::from(99u64));
        assert_eq!(transaction.memo, None, "a swap carries no memo, which is absence and not an empty one");
        assert_eq!(transaction.direction, TransactionDirection::Outgoing);
        assert!(transaction.metadata.is_some());

        let approval_leg = TransactionInputType::Swap {
            from_asset: Asset::mock_with_params(Chain::Ethereum, Some("0xusdc".to_string()), "USDC".to_string(), "USDC".to_string(), 6, AssetType::ERC20),
            to_asset: Asset::from_chain(Chain::Bitcoin),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Thorchain, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: Some(ApprovalData {
                        token: "0xusdc".into(),
                        spender: "0xspender".into(),
                        value: BigUint::from(100u64),
                        is_unlimited: false,
                    }),
                    ..SwapQuoteData::mock()
                },
            },
        };
        let transaction = GemPendingTransactionInput::mock(approval_leg, TransactionType::TokenApproval, "0xhash", 0, 2).pending_transaction().unwrap().unwrap();
        assert_eq!(transaction.to, "0xspender");
        assert_eq!(transaction.contract, None, "the approval leg is the token spender, not the swap contract");
        assert_eq!(transaction.asset_id, AssetId::from(Chain::Ethereum, Some("0xusdc".into())));
        assert!(transaction.metadata.is_none());

        let deposit = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Near),
            to_asset: Asset::from_chain(Chain::Ethereum),
            swap_data: SwapData::mock_transfer(SwapProvider::NearIntents, "100", "90", "deposit.near"),
        };
        let transaction = GemPendingTransactionInput::mock(deposit, TransactionType::Swap, "near-hash", 0, 1).pending_transaction().unwrap().unwrap();
        assert_eq!(transaction.to, "deposit.near");
        assert_eq!(transaction.contract, None, "a deposit-address swap such as NEAR Intents has no contract to open");

        let generic = TransactionInputType::mock_payment(
            Asset::from_chain(Chain::Solana),
            TransferDataExtra {
                to: String::new(),
                data: Some(b"encoded".to_vec()),
                output_action: TransferDataOutputAction::Send,
                transaction_type: TransactionType::Transfer,
                ..TransferDataExtra::mock()
            },
        );
        let mut generic_input = GemPendingTransactionInput::mock(generic, TransactionType::Transfer, "hash", 0, 1);
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
        assert_eq!(transaction.value, BigUint::from(19_000_000u64));
        assert_eq!(transaction.to, "recipient");
        assert!(transaction.payment_metadata().is_some(), "a broadcast payment carries the merchant like a signature payment");

        let token_payment = TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock_signature(b"typed data".to_vec(), Some(ApprovalData::mock())));
        let approve_leg = GemPendingTransactionInput::mock(token_payment.clone(), TransactionType::TokenApproval, "0xapprove", 0, 2)
            .pending_transaction()
            .unwrap()
            .unwrap();
        assert_eq!(approve_leg.to, ApprovalData::mock().spender);
        assert!(approve_leg.metadata.is_none(), "the approve leg must not look like the payment to the tracker");
        let payment_leg = GemPendingTransactionInput::mock(token_payment, TransactionType::Transfer, "pay_1", 0, 1).pending_transaction().unwrap().unwrap();
        assert_eq!(payment_leg.to, "recipient");
        assert!(payment_leg.payment_metadata().is_some());

        let hypercore_swap = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Ethereum),
            to_asset: Asset::from_chain(Chain::HyperCore),
            swap_data: SwapData {
                quote: SwapQuote::mock_with_values(SwapProvider::Hyperliquid, "100", "90"),
                data: SwapQuoteData {
                    to: "0xrouter".into(),
                    approval: None,
                    ..SwapQuoteData::mock()
                },
            },
        };
        assert!(GemPendingTransactionInput::mock(hypercore_swap.clone(), TransactionType::Swap, "0xhash", 0, 2).pending_transaction().unwrap().is_some());
        let hypercore_stake = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::HyperCore),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert!(GemPendingTransactionInput::mock(hypercore_stake.clone(), TransactionType::StakeRewards, "h", 0, 2).pending_transaction().unwrap().is_none());
        assert!(GemPendingTransactionInput::mock(hypercore_stake, TransactionType::StakeRewards, "h", 1, 2).pending_transaction().unwrap().is_some());
        assert!(
            GemPendingTransactionInput::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) }, TransactionType::PerpetualModifyPosition, "h", 0, 1)
                .pending_transaction()
                .unwrap()
                .is_none()
        );
    }
}
