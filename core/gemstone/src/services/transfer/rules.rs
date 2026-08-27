use chrono::Utc;
use num_bigint::BigInt;
use primitives::SwapProvider;
use primitives::known_assets::wallet_default_assets;
use primitives::swap::ApprovalData;
use primitives::{
    ApplicationMetadataSource, Asset, AssetId, AssetType, Chain, EarnType, PerpetualType, StakeType, Transaction, TransactionDirection, TransactionInputType,
    TransactionNFTTransferMetadata, TransactionPerpetualMetadata, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionType,
    TransactionWalletConnectMetadata, TransferDataOutputAction, TransferDataOutputType,
};

use super::model::{GemPendingTransactionInput, GemTransferBalance, GemTransferData, GemTransferOutput};
use crate::models::transaction::{GemTransactionInputType, transaction_metadata_block_number, transaction_metadata_sequence};

pub fn transaction_type(input_type: &GemTransactionInputType) -> TransactionType {
    TransactionInputType::from(input_type.clone()).transaction_type()
}

pub fn asset(input_type: &GemTransactionInputType) -> Asset {
    match input_type {
        GemTransactionInputType::TransferNft { asset, .. } => Asset::from_chain(asset.chain()),
        _ => input_type.asset().clone(),
    }
}

pub fn asset_ids(input_type: &GemTransactionInputType) -> Vec<AssetId> {
    match input_type {
        GemTransactionInputType::Swap { from_asset, to_asset, .. } => vec![from_asset.id.clone(), to_asset.id.clone()],
        GemTransactionInputType::TransferNft { .. } => vec![],
        _ => vec![input_type.asset().id.clone()],
    }
}

pub fn fee_asset(input_type: &GemTransactionInputType) -> Asset {
    let asset = asset(input_type);
    let chain = asset.chain();
    if matches!(input_type, GemTransactionInputType::Perpetual { .. }) && chain == Chain::HyperCore {
        return default_asset(chain, AssetType::PERPETUAL).unwrap_or(asset);
    }
    match chain {
        Chain::Tempo => asset,
        Chain::HyperCore => default_asset(chain, AssetType::TOKEN).unwrap_or(asset),
        _ if asset.id.is_token() => Asset::from_chain(chain),
        _ => asset,
    }
}

pub fn output(input_type: &GemTransactionInputType) -> GemTransferOutput {
    match input_type {
        GemTransactionInputType::Generic { extra, .. } => GemTransferOutput {
            output_type: extra.output_type.clone(),
            output_action: extra.output_action.clone(),
        },
        _ => GemTransferOutput {
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Send,
        },
    }
}

pub fn approval(input_type: &GemTransactionInputType, transaction_type: TransactionType) -> Result<Option<ApprovalData>, String> {
    if transaction_type != TransactionType::TokenApproval {
        return Ok(None);
    }
    match input_type {
        GemTransactionInputType::Swap { swap_data, .. } => swap_data.data.approval.clone().map(Some).ok_or("Missing swap approval data".to_string()),
        GemTransactionInputType::Earn { data, .. } => data.approval.clone().map(Some).ok_or("Missing earn approval data".to_string()),
        GemTransactionInputType::TokenApprove { approval_data, .. } => Ok(Some(approval_data.clone())),
        GemTransactionInputType::Generic { extra, .. } => Ok(extra.approval.clone()),
        _ => Err("Token approval transaction type does not match transfer data".to_string()),
    }
}

pub fn spends_balance(input_type: &GemTransactionInputType) -> bool {
    match input_type {
        GemTransactionInputType::Withdrawal { .. } => true,
        _ => TransactionInputType::from(input_type.clone()).spends_balance(),
    }
}

pub fn metadata(input_type: &GemTransactionInputType) -> Result<Option<serde_json::Value>, serde_json::Error> {
    let value = match input_type {
        GemTransactionInputType::Swap { from_asset, to_asset, swap_data } => Some(serde_json::to_value(TransactionSwapMetadata {
            from_asset: from_asset.id.clone(),
            from_value: swap_data.quote.from_value.clone(),
            to_asset: to_asset.id.clone(),
            to_value: swap_data.quote.to_value.clone(),
            provider: Some(swap_data.quote.provider_data.provider.as_ref().to_string()),
        })?),
        GemTransactionInputType::TransferNft { nft_asset, .. } => Some(serde_json::to_value(TransactionNFTTransferMetadata::new(
            nft_asset.id.clone(),
            Some(nft_asset.name.clone()),
        ))?),
        GemTransactionInputType::Perpetual { perpetual_type, .. } => match perpetual_type {
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
        GemTransactionInputType::Stake { stake_type, .. } => match stake_type {
            StakeType::Freeze(data) | StakeType::Unfreeze(data) => Some(serde_json::to_value(TransactionResourceTypeMetadata::new(*data))?),
            StakeType::Stake(_) | StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => None,
        },
        GemTransactionInputType::Generic { metadata, extra, .. } => match metadata.source {
            ApplicationMetadataSource::WalletConnect => Some(serde_json::to_value(TransactionWalletConnectMetadata {
                output_action: extra.output_action.clone(),
            })?),
            ApplicationMetadataSource::Payment => None,
        },
        GemTransactionInputType::Transfer { .. }
        | GemTransactionInputType::Deposit { .. }
        | GemTransactionInputType::Withdrawal { .. }
        | GemTransactionInputType::TokenApprove { .. }
        | GemTransactionInputType::Account { .. }
        | GemTransactionInputType::Earn { .. } => None,
    };
    Ok(value)
}

pub fn tron_stake_available(asset: &Asset, balance: &GemTransferBalance) -> BigInt {
    let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
    let staked = BigInt::from(balance.votes) * BigInt::from(10u32).pow(asset.decimals.max(0) as u32);
    (parse(&balance.frozen) + parse(&balance.locked) - staked).max(BigInt::from(0))
}

pub fn unfreeze_available(resource: &primitives::Resource, balance: &GemTransferBalance) -> BigInt {
    let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
    match resource {
        primitives::Resource::Bandwidth => parse(&balance.frozen),
        primitives::Resource::Energy => parse(&balance.locked),
    }
}

pub fn available_value(transfer: &GemTransferData, balance: &GemTransferBalance) -> BigInt {
    let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
    let asset = transfer.input_type.asset();
    match &transfer.input_type {
        GemTransactionInputType::Withdrawal { .. } => parse(&balance.withdrawable),
        GemTransactionInputType::Stake { stake_type, .. } => match stake_type {
            StakeType::Unstake(delegation) | StakeType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
            StakeType::Redelegate(data) => BigInt::from(data.delegation.base.balance.clone()),
            StakeType::Rewards(_) => parse(&transfer.value),
            StakeType::Unfreeze(resource) => unfreeze_available(resource, balance),
            StakeType::Stake(_) if asset.chain() == Chain::Tron => tron_stake_available(asset, balance),
            StakeType::Stake(_) | StakeType::Freeze(_) => parse(&balance.available),
        },
        GemTransactionInputType::Earn { earn_type, .. } => match earn_type {
            EarnType::Withdraw(delegation) => BigInt::from(delegation.base.balance.clone()),
            EarnType::Deposit(_) => parse(&balance.available),
        },
        _ => parse(&balance.available),
    }
}

pub fn pending_transaction(input: GemPendingTransactionInput) -> Result<Option<Transaction>, String> {
    if !is_tracked(
        &input.transfer.input_type,
        &input.transaction_type,
        &input.hash,
        input.transaction_index,
        input.transaction_count,
    ) {
        return Ok(None);
    }
    let transfer = input.transfer;
    let chain = transfer.input_type.asset().chain();
    let approval = approval(&transfer.input_type, input.transaction_type.clone())?;
    let simulation_header = match transfer.input_type {
        GemTransactionInputType::Generic { .. } => input.simulation.and_then(|simulation| simulation.header),
        _ => None,
    };
    let (recipient, value, memo) = match &approval {
        Some(approval) => (approval.spender.clone(), approval.value.clone(), String::new()),
        None => {
            let recipient = match &transfer.input_type {
                GemTransactionInputType::Swap { swap_data, .. } => swap_data.data.to.clone(),
                _ => transfer.recipient.address.clone(),
            };
            let value = simulation_header.as_ref().map(|header| header.value.clone()).unwrap_or(input.value);
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
        .unwrap_or_else(|| asset(&transfer.input_type).id);
    let direction = if input.sender == recipient {
        TransactionDirection::SelfTransfer
    } else {
        TransactionDirection::Outgoing
    };
    let metadata = match transfer.input_type {
        GemTransactionInputType::Swap { .. } | GemTransactionInputType::Earn { .. } if approval.is_some() => None,
        _ => metadata(&transfer.input_type).map_err(|error| error.to_string())?,
    };
    let mut transaction = Transaction::new(
        input.hash,
        asset_id,
        input.sender,
        recipient,
        None,
        input.transaction_type,
        TransactionState::Pending,
        input.network_fee,
        input.fee.fee_asset,
        value,
        Some(memo),
        metadata,
        Utc::now(),
    );
    transaction.block_number = Some(transaction_metadata_block_number(&input.metadata));
    transaction.sequence = Some(transaction_metadata_sequence(&input.metadata));
    transaction.direction = direction;
    Ok(Some(transaction))
}

fn is_tracked(input_type: &GemTransactionInputType, transaction_type: &TransactionType, hash: &str, index: u32, count: u32) -> bool {
    if *transaction_type == TransactionType::PerpetualModifyPosition {
        return false;
    }
    if input_type.asset().chain() != Chain::HyperCore {
        return true;
    }
    let is_intermediate = index + 1 < count;
    match input_type {
        GemTransactionInputType::Stake { .. } => !is_intermediate,
        GemTransactionInputType::Perpetual { .. } => hash.starts_with(HYPERCORE_ORDER_PREFIX),
        GemTransactionInputType::Swap { to_asset, swap_data, .. } => {
            !(to_asset.chain() == Chain::HyperCore && swap_data.quote.provider_data.provider == SwapProvider::Hyperliquid && is_intermediate)
        }
        _ => true,
    }
}

const HYPERCORE_ORDER_PREFIX: &str = "order:";

fn default_asset(chain: Chain, asset_type: AssetType) -> Option<Asset> {
    wallet_default_assets(chain).into_iter().find(|asset| asset.asset_type == asset_type)
}

#[cfg(test)]
mod tests {
    use super::super::model::GemRecipient;
    use super::*;
    use crate::models::gateway::GemGasPriceType;
    use crate::models::transaction::{GemFeeOptions, GemTransactionLoadFee, GemTransactionLoadMetadata};
    use num_bigint::BigUint;
    use primitives::{
        Delegation, DelegationBase, DelegationState, DelegationValidator, Resource, StakeProviderType, SwapProvider, TransactionType,
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
                    value: "100".into(),
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

    fn transfer(input_type: GemTransactionInputType, value: &str) -> GemTransferData {
        GemTransferData {
            input_type,
            recipient: GemRecipient {
                address: "recipient".into(),
                name: None,
                memo: Some("memo".into()),
                references: vec![],
            },
            value: value.into(),
            use_max_amount: false,
            minimum_value: None,
        }
    }

    fn balance(available: u64, frozen: u64, locked: u64) -> GemTransferBalance {
        GemTransferBalance {
            available: available.to_string(),
            frozen: frozen.to_string(),
            locked: locked.to_string(),
            withdrawable: "0".to_string(),
            votes: 0,
        }
    }

    fn pending_input(input_type: GemTransactionInputType, transaction_type: TransactionType, hash: &str, index: u32, count: u32) -> GemPendingTransactionInput {
        GemPendingTransactionInput {
            sender: "sender".into(),
            transfer: transfer(input_type, "100"),
            value: "99".into(),
            transaction_type,
            hash: hash.into(),
            fee: GemTransactionLoadFee {
                fee: "1".into(),
                gas_price_type: GemGasPriceType::Regular { gas_price: "1".into() },
                gas_limit: "21000".into(),
                options: GemFeeOptions { options: HashMap::new() },
                fee_asset: AssetId::from_chain(Chain::Ethereum),
            },
            network_fee: "1".into(),
            metadata: GemTransactionLoadMetadata::None,
            simulation: None,
            transaction_index: index,
            transaction_count: count,
        }
    }

    #[test]
    fn test_swap_rules() {
        let input = swap_input(asset(Chain::Ethereum), asset(Chain::Bitcoin), SwapProvider::Thorchain, None);
        assert_eq!(transaction_type(&input), TransactionType::Swap);
        assert_eq!(asset_ids(&input), vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Bitcoin)]);
        assert_eq!(fee_asset(&input).id, AssetId::from_chain(Chain::Ethereum));
        assert!(spends_balance(&input));
        let metadata = metadata(&input).unwrap().unwrap();
        assert_eq!(metadata["provider"], "thorchain");
        assert_eq!(metadata["toValue"], "90");
        assert!(approval(&input, TransactionType::TokenApproval).is_err());
        assert_eq!(approval(&input, TransactionType::Swap).unwrap(), None);
    }

    #[test]
    fn test_fee_asset_for_tokens_and_hypercore() {
        let token_transfer = GemTransactionInputType::Transfer {
            asset: token(Chain::Ethereum, "0xusdc"),
        };
        assert_eq!(fee_asset(&token_transfer).id, AssetId::from_chain(Chain::Ethereum));
        let tempo_token = GemTransactionInputType::Transfer {
            asset: token(Chain::Tempo, "0xusdc"),
        };
        assert!(fee_asset(&tempo_token).id.is_token());
        let hypercore = GemTransactionInputType::Transfer { asset: asset(Chain::HyperCore) };
        assert_eq!(fee_asset(&hypercore).asset_type, AssetType::TOKEN);
    }

    #[test]
    fn test_stake_metadata_and_available_value() {
        let unfreeze = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Unfreeze(Resource::Bandwidth),
        };
        assert_eq!(metadata(&unfreeze).unwrap().unwrap()["resourceType"], "bandwidth");
        assert_eq!(available_value(&transfer(unfreeze, "1"), &balance(10, 20, 30)), BigInt::from(20));

        let unstake = GemTransactionInputType::Stake {
            asset: asset(Chain::Cosmos),
            stake_type: StakeType::Unstake(delegation(700)),
        };
        assert_eq!(available_value(&transfer(unstake, "1"), &balance(10, 0, 0)), BigInt::from(700));
        let rewards = GemTransactionInputType::Stake {
            asset: asset(Chain::Cosmos),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert_eq!(available_value(&transfer(rewards, "42"), &balance(10, 0, 0)), BigInt::from(42));
        let tron_stake = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Stake(delegation(0).validator),
        };
        assert_eq!(
            available_value(
                &transfer(tron_stake, "1"),
                &GemTransferBalance {
                    votes: 2,
                    ..balance(1, 5_000_000, 3_000_000)
                }
            ),
            BigInt::from(6_000_000)
        );
        let overvoted = GemTransactionInputType::Stake {
            asset: asset(Chain::Tron),
            stake_type: StakeType::Stake(delegation(0).validator),
        };
        assert_eq!(
            available_value(
                &transfer(overvoted, "1"),
                &GemTransferBalance {
                    votes: 9,
                    ..balance(1, 5_000_000, 3_000_000)
                }
            ),
            BigInt::from(0)
        );
        let withdrawal = GemTransactionInputType::Withdrawal { asset: asset(Chain::HyperCore) };
        assert_eq!(
            available_value(
                &transfer(withdrawal, "1"),
                &GemTransferBalance {
                    withdrawable: "9".to_string(),
                    ..balance(10, 0, 0)
                }
            ),
            BigInt::from(9)
        );
    }

    #[test]
    fn test_pending_transaction_uses_swap_router_and_hypercore_tracking() {
        let swap = swap_input(asset(Chain::Ethereum), asset(Chain::Bitcoin), SwapProvider::Thorchain, None);
        let transaction = pending_transaction(pending_input(swap, TransactionType::Swap, "0xhash", 0, 1)).unwrap().unwrap();
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
                value: "100".into(),
                is_unlimited: false,
            }),
        );
        let transaction = pending_transaction(pending_input(approval_leg, TransactionType::TokenApproval, "0xhash", 0, 2))
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
                value: "19000000".into(),
                is_unlimited: false,
            }),
        });
        let transaction = pending_transaction(generic_input).unwrap().unwrap();
        assert_eq!(transaction.asset_id, AssetId::from(Chain::Solana, Some("usdc".into())));
        assert_eq!(transaction.value, "19000000");
        assert_eq!(transaction.to, "recipient");

        let hypercore_swap = swap_input(asset(Chain::Ethereum), asset(Chain::HyperCore), SwapProvider::Hyperliquid, None);
        assert!(
            pending_transaction(pending_input(hypercore_swap.clone(), TransactionType::Swap, "0xhash", 0, 2))
                .unwrap()
                .is_some()
        );
        let hypercore_stake = GemTransactionInputType::Stake {
            asset: asset(Chain::HyperCore),
            stake_type: StakeType::Rewards(vec![]),
        };
        assert!(
            pending_transaction(pending_input(hypercore_stake.clone(), TransactionType::StakeRewards, "h", 0, 2))
                .unwrap()
                .is_none()
        );
        assert!(
            pending_transaction(pending_input(hypercore_stake, TransactionType::StakeRewards, "h", 1, 2))
                .unwrap()
                .is_some()
        );
        assert!(
            pending_transaction(pending_input(
                GemTransactionInputType::Transfer { asset: asset(Chain::Ethereum) },
                TransactionType::PerpetualModifyPosition,
                "h",
                0,
                1
            ))
            .unwrap()
            .is_none()
        );
    }
}
