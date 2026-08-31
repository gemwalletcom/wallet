use num_bigint::BigInt;
use primitives::{
    ApplicationMetadataSource, Asset, AssetId, Chain, ChainType, FeePriority, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, Transaction, TransactionPreloadInput,
    TransferDataOutputAction, Wallet,
};

use super::error::GemConfirmError;
use super::model::{GemAcquireAssetFlow, GemConfirmData, GemConfirmFeeSelection, GemConfirmInput, GemConfirmMetadata, GemFeeAsset, GemSendInput, GemTransferAmountResult};
use crate::fee::custom_gas_price;
use crate::models::gateway::{GemBroadcastOptions, GemFeeRate, GemTransactionPreloadInput};
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadInput};
use crate::services::balance::GemAssetBalance;
use crate::services::price::GemAssetPrice;
use crate::services::transfer::GemTransferBalance;
use crate::services::transfer::{GemPendingTransactionInput, rules as transfer_rules};
use crate::transfer_amount::{GemTransferAmountInput, calculate_transfer_amount};

pub fn signer_input(input: &GemSendInput) -> Result<GemSignerInput, GemConfirmError> {
    let GemConfirmInput { from, transfer } = &input.confirm.input;
    let chain = transfer.input_type.asset().chain();
    let sender_address = signing_address(&input.wallet, chain, &from.address)?;
    Ok(GemSignerInput {
        input: GemTransactionLoadInput {
            input_type: transfer.input_type.clone(),
            sender_address,
            destination_address: transfer.recipient.address.clone(),
            value: input.value.to_string(),
            gas_price: input.confirm.fee.gas_price_type.clone(),
            memo: transfer.recipient.memo.clone(),
            is_max_value: transfer.use_max_amount,
            metadata: input.confirm.metadata.clone(),
        },
        fee: GemTransactionLoadFee {
            fee: input.network_fee.clone(),
            ..input.confirm.fee.clone()
        },
    })
}

fn signing_address(wallet: &Wallet, chain: Chain, from: &str) -> Result<String, GemConfirmError> {
    let signer = wallet.account(chain).ok_or(GemConfirmError::AccountMissing { chain })?;
    if signer.address != from {
        return Err(GemConfirmError::SenderMismatch {
            from: from.to_string(),
            signer: signer.address.clone(),
        });
    }
    Ok(signer.address.clone())
}

pub fn metadata_asset_ids(asset_id: &AssetId, fee_asset_id: &AssetId, extra_asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    let mut asset_ids: Vec<AssetId> = Vec::new();
    for asset_id in [asset_id.clone(), fee_asset_id.clone()].into_iter().chain(extra_asset_ids) {
        if !asset_ids.contains(&asset_id) {
            asset_ids.push(asset_id);
        }
    }
    asset_ids
}

fn transfer_balance(balance: &GemAssetBalance) -> GemTransferBalance {
    GemTransferBalance {
        available: balance.available.clone().into(),
        frozen: balance.frozen.clone().into(),
        locked: balance.locked.clone().into(),
        withdrawable: balance.withdrawable.clone().into(),
        votes: 0,
    }
}

pub fn preload_amount(data: &GemConfirmData, metadata: &GemConfirmMetadata, fee_asset: &Asset) -> Result<GemTransferAmountResult, GemConfirmError> {
    let transfer = &data.input.transfer;
    let balance = transfer_balance(&metadata.asset_balance);
    let available_value = transfer_rules::available_value(transfer, &balance).map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
    let input = GemTransferAmountInput {
        input_type: transfer.input_type.clone(),
        value: transfer.value.clone(),
        available_value,
        fee_asset: fee_asset.id.clone(),
        fee_asset_balance: metadata.fee_asset_balance.available.clone().into(),
        fee: data.fee.fee.clone(),
        is_max_amount: transfer.use_max_amount,
        minimum_value: transfer.minimum_value.clone(),
    };
    Ok(match calculate_transfer_amount(input) {
        Ok(amount) => GemTransferAmountResult::Amount { amount },
        Err(error) => GemTransferAmountResult::Error { error },
    })
}

pub fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<GemAssetPrice>) -> Vec<GemFeeAsset> {
    balances
        .into_iter()
        .filter(|balance| balance.available > num_bigint::BigUint::from(0u32))
        .filter_map(|balance| {
            let asset = assets.iter().find(|asset| asset.id == balance.asset_id)?.clone();
            let price = prices.iter().find(|price| price.asset_id == balance.asset_id).cloned();
            Some(GemFeeAsset { asset, balance, price })
        })
        .collect()
}

pub fn build_metadata(asset_id: AssetId, fee_asset_id: AssetId, balances: Vec<GemAssetBalance>, prices: Vec<GemAssetPrice>) -> Result<GemConfirmMetadata, GemConfirmError> {
    Ok(GemConfirmMetadata {
        asset_balance: asset_balance(&balances, &asset_id)?,
        fee_asset_balance: asset_balance(&balances, &fee_asset_id)?,
        prices,
    })
}

fn asset_balance(balances: &[GemAssetBalance], asset_id: &AssetId) -> Result<GemAssetBalance, GemConfirmError> {
    balances
        .iter()
        .find(|balance| balance.asset_id == *asset_id)
        .cloned()
        .ok_or_else(|| GemConfirmError::BalanceMissing { asset_id: asset_id.clone() })
}

pub fn validate_approvals(input_type: &GemTransactionInputType, transactions: &[GemSignedTransaction]) -> Result<(), GemConfirmError> {
    for transaction in transactions {
        let approval = transfer_rules::approval(input_type, transaction.transaction_type.clone()).map_err(|msg| GemConfirmError::ApprovalInvalid { msg })?;
        if let Some(approval) = approval
            && approval.value.parse::<BigInt>().is_err()
        {
            return Err(GemConfirmError::ApprovalInvalid {
                msg: format!("approval value is not an integer: {}", approval.value),
            });
        }
    }
    Ok(())
}

pub fn output_action(input_type: &GemTransactionInputType) -> TransferDataOutputAction {
    transfer_rules::output(input_type).output_action
}

pub fn acquire_asset_flow(chain: Chain) -> GemAcquireAssetFlow {
    match chain {
        Chain::Tron => GemAcquireAssetFlow::Options,
        _ => GemAcquireAssetFlow::Fiat,
    }
}

pub fn default_fee_priority(input_type: GemTransactionInputType) -> FeePriority {
    match input_type {
        GemTransactionInputType::Swap { from_asset, .. } if from_asset.chain() == Chain::Bitcoin => FeePriority::Fast,
        _ => FeePriority::Normal,
    }
}

pub fn is_insufficient_network_fee(fee_asset_id: AssetId, fee_available: &str) -> bool {
    if matches!(fee_asset_id.chain, Chain::HyperCore | Chain::Tron) || !fee_asset_id.is_native() {
        return false;
    }
    fee_available.trim().is_empty() || fee_available.trim().chars().all(|character| character == '0')
}

pub(super) fn pending_transactions(input: &GemSendInput, hashes: &[String], transactions: &[GemSignedTransaction]) -> Result<Vec<Transaction>, GemConfirmError> {
    let chain = input.confirm.input.transfer.input_type.asset().chain();
    let sender = input.wallet.account(chain).map(|account| account.address.clone()).ok_or_else(|| GemConfirmError::Record {
        msg: format!("wallet has no {chain} account"),
    })?;
    hashes
        .iter()
        .enumerate()
        .filter_map(|(index, hash)| {
            let transaction_type = transactions.get(index)?.transaction_type.clone();
            Some(
                transfer_rules::pending_transaction(GemPendingTransactionInput {
                    sender: sender.clone(),
                    transfer: input.confirm.input.transfer.clone(),
                    value: input.value.clone(),
                    transaction_type,
                    hash: hash.clone(),
                    fee: input.confirm.fee.clone(),
                    network_fee: input.network_fee.clone(),
                    metadata: input.confirm.metadata.clone(),
                    simulation: input.simulation.clone(),
                    transaction_index: index as u32,
                    transaction_count: transactions.len() as u32,
                })
                .map_err(|msg| GemConfirmError::Record { msg }),
            )
        })
        .map(|result| result.map(|transaction| transaction.into_iter()))
        .collect::<Result<Vec<_>, _>>()
        .map(|transactions| transactions.into_iter().flatten().collect())
}

pub(super) fn simulation_payload(input_type: &GemTransactionInputType) -> Option<String> {
    let GemTransactionInputType::Generic { metadata, extra, .. } = input_type else {
        return None;
    };
    match metadata.source {
        ApplicationMetadataSource::Payment => extra.data.as_ref().and_then(|data| String::from_utf8(data.clone()).ok()),
        ApplicationMetadataSource::WalletConnect => None,
    }
}

pub(super) fn broadcast_options(chain: Chain, input_type: &GemTransactionInputType) -> GemBroadcastOptions {
    match (chain, input_type) {
        (Chain::Solana, GemTransactionInputType::Swap { .. } | GemTransactionInputType::Generic { .. }) => GemBroadcastOptions { skip_preflight: true },
        _ => GemBroadcastOptions { skip_preflight: false },
    }
}

pub(super) fn broadcast_delay_milliseconds(chain: Chain) -> u64 {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => 0,
        ChainType::Solana
        | ChainType::Bitcoin
        | ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Xrp
        | ChainType::Polkadot
        | ChainType::Cardano => 500,
    }
}

pub(super) fn validate_scan(scan: Option<&ScanTransaction>, memo: Option<&str>, symbol: &str) -> Result<(), GemConfirmError> {
    let Some(scan) = scan else {
        return Ok(());
    };
    if scan.is_malicious == Some(true) {
        return Err(GemConfirmError::ScanMalicious);
    }
    if scan.is_memo_required == Some(true) && memo.unwrap_or_default().trim().is_empty() {
        return Err(GemConfirmError::ScanMemoRequired { symbol: symbol.to_string() });
    }
    Ok(())
}

pub(super) fn scan_payload(input: GemTransactionPreloadInput) -> ScanTransactionPayload {
    let input: TransactionPreloadInput = input.into();
    ScanTransactionPayload {
        origin: ScanAddressTarget {
            asset_id: input.input_type.get_asset().id.clone(),
            address: input.sender_address.clone(),
        },
        target: ScanAddressTarget {
            asset_id: input.input_type.get_recipient_asset().id.clone(),
            address: input.destination_address.clone(),
        },
        website: input.get_website(),
        transaction_type: input.input_type.transaction_type(),
    }
}

pub(super) fn select_fee_rate(rates: &[GemFeeRate], selection: &GemConfirmFeeSelection) -> Result<GemFeeRate, GemConfirmError> {
    match selection {
        GemConfirmFeeSelection::Priority { priority } => rates
            .iter()
            .find(|rate| &rate.priority == priority)
            .or_else(|| rates.first())
            .cloned()
            .ok_or(GemConfirmError::FeeRatesMissing),
        GemConfirmFeeSelection::Custom { gas_price } => {
            let base = rates
                .iter()
                .find(|rate| rate.priority == FeePriority::Normal)
                .or_else(|| rates.first())
                .ok_or(GemConfirmError::FeeRatesMissing)?;
            Ok(GemFeeRate {
                priority: base.priority,
                gas_price_type: custom_gas_price(base.gas_price_type.clone(), gas_price.clone()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::GemConfirmData;
    use super::*;
    use crate::models::custom_types::GemBigInt;
    use crate::models::custom_types::GemBigUint;
    use crate::models::gateway::GemGasPriceType;
    use crate::models::transaction::GemTransactionLoadMetadata;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use primitives::{
        Account, ApplicationMetadata, Asset, PerpetualConfirmData, PerpetualDirection, PerpetualType, StakeType, TransactionType, TransferDataExtra, Wallet, WalletId,
        swap::{ApprovalData, SwapData},
    };

    fn wallet(chain: Chain) -> Wallet {
        Wallet {
            id: WalletId::Multicoin("wallet".to_string()),
            ..Wallet::mock_with_accounts(vec![Account::mock(chain, "sender")])
        }
    }

    fn send_input(chain: Chain, input_type: GemTransactionInputType) -> GemSendInput {
        send_input_from(chain, input_type, "sender")
    }

    fn send_input_from(chain: Chain, input_type: GemTransactionInputType, from: &str) -> GemSendInput {
        GemSendInput {
            wallet: wallet(chain),
            confirm: GemConfirmData {
                input: GemConfirmInput {
                    from: Account::mock(chain, from),
                    transfer: GemTransferData {
                        input_type,
                        recipient: GemRecipient {
                            address: "recipient".to_string(),
                            name: None,
                            memo: Some("memo".to_string()),
                            references: vec![],
                        },
                        value: BigInt::from(10),
                        use_max_amount: true,
                        minimum_value: None,
                    },
                },
                fee: GemTransactionLoadFee {
                    fee: BigInt::ZERO,
                    gas_price_type: GemGasPriceType::Regular { gas_price: BigInt::from(5) },
                    gas_limit: BigInt::from(21_000),
                    options: Default::default(),
                    fee_asset: AssetId::from_chain(Chain::Solana),
                },
                selected_priority: FeePriority::Normal,
                fee_rates: vec![],
                metadata: GemTransactionLoadMetadata::None,
                simulation: None,
            },
            value: BigInt::from(9),
            network_fee: BigInt::from(1),
            simulation: None,
        }
    }

    #[test]
    fn test_signer_input_uses_wallet_account_and_network_fee() {
        let input = send_input(Chain::Solana, GemTransactionInputType::Transfer { asset: Asset::mock_sol() });

        let signer_input = signer_input(&input).unwrap();

        assert_eq!(signer_input.input.sender_address, "sender");
        assert_eq!(signer_input.input.destination_address, "recipient");
        assert_eq!(signer_input.input.value, "9");
        assert_eq!(signer_input.input.memo.as_deref(), Some("memo"));
        assert!(signer_input.input.is_max_value);
        assert_eq!(signer_input.fee.fee, BigInt::from(1));
        assert_eq!(signer_input.fee.gas_limit, BigInt::from(21_000));
    }

    #[test]
    fn test_signer_input_refuses_to_sign_for_an_address_the_transaction_was_not_priced_for() {
        let matching = send_input_from(Chain::Solana, GemTransactionInputType::Transfer { asset: Asset::mock_sol() }, "sender");
        assert!(signer_input(&matching).is_ok());

        let switched = send_input_from(Chain::Solana, GemTransactionInputType::Transfer { asset: Asset::mock_sol() }, "other");

        assert!(matches!(
            signer_input(&switched).unwrap_err(),
            GemConfirmError::SenderMismatch { from, signer } if from == "other" && signer == "sender"
        ));
    }

    #[test]
    fn test_signer_input_requires_account_for_chain() {
        let input = send_input(Chain::Ethereum, GemTransactionInputType::Transfer { asset: Asset::mock_sol() });

        match signer_input(&input) {
            Err(GemConfirmError::AccountMissing { chain: Chain::Solana }) => {}
            result => panic!("expected a missing account error, got {result:?}"),
        }
    }

    #[test]
    fn test_validate_approvals_rejects_non_integer_value() {
        let approval = |value: &str| GemTransactionInputType::TokenApprove {
            asset: Asset::mock_sol(),
            approval_data: ApprovalData {
                value: value.to_string(),
                ..ApprovalData::mock()
            },
        };
        let signed = |transaction_type: TransactionType| {
            vec![GemSignedTransaction {
                data: "signed".to_string(),
                transaction_type,
            }]
        };

        assert!(validate_approvals(&approval("1000"), &signed(TransactionType::TokenApproval)).is_ok());
        assert!(validate_approvals(&approval("abc"), &signed(TransactionType::Transfer)).is_ok());
        match validate_approvals(&approval("abc"), &signed(TransactionType::TokenApproval)) {
            Err(GemConfirmError::ApprovalInvalid { .. }) => {}
            result => panic!("expected an invalid approval error, got {result:?}"),
        }
        match validate_approvals(&GemTransactionInputType::Transfer { asset: Asset::mock_sol() }, &signed(TransactionType::TokenApproval)) {
            Err(GemConfirmError::ApprovalInvalid { .. }) => {}
            result => panic!("expected an invalid approval error, got {result:?}"),
        }
    }

    #[test]
    fn test_output_action_only_generic_transfers_can_sign() {
        let generic = GemTransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata: ApplicationMetadata::mock(),
            extra: TransferDataExtra {
                output_action: TransferDataOutputAction::Sign,
                ..TransferDataExtra::mock()
            }
            .into(),
        };

        assert_eq!(output_action(&generic), TransferDataOutputAction::Sign);
        assert_eq!(
            output_action(&GemTransactionInputType::Transfer { asset: Asset::mock_sol() }),
            TransferDataOutputAction::Send
        );
    }
    fn rate(priority: FeePriority, gas_price: &str) -> GemFeeRate {
        GemFeeRate {
            priority,
            gas_price_type: GemGasPriceType::Regular {
                gas_price: gas_price.parse().unwrap(),
            },
        }
    }

    #[test]
    fn test_select_fee_rate() {
        let rates = vec![rate(FeePriority::Normal, "10"), rate(FeePriority::Fast, "20")];

        let fast = select_fee_rate(&rates, &GemConfirmFeeSelection::Priority { priority: FeePriority::Fast }).unwrap();
        assert_eq!(fast.priority, FeePriority::Fast);

        let fallback = select_fee_rate(&[rate(FeePriority::Fast, "20")], &GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }).unwrap();
        assert_eq!(fallback.priority, FeePriority::Fast);

        let custom = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: BigInt::from(33) }).unwrap();
        assert_eq!(custom.priority, FeePriority::Normal);
        match custom.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, BigInt::from(33)),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&[], &GemConfirmFeeSelection::Priority { priority: FeePriority::Normal }) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_select_fee_rate_custom() {
        let eip1559 = GemFeeRate {
            priority: FeePriority::Normal,
            gas_price_type: GemGasPriceType::Eip1559 {
                gas_price: BigInt::from(20),
                priority_fee: BigInt::from(5),
            },
        };
        let rates = vec![rate(FeePriority::Fast, "1"), eip1559];

        let raised = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: BigInt::from(30) }).unwrap();
        assert_eq!(raised.priority, FeePriority::Normal);
        match raised.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price, priority_fee), (BigInt::from(25), BigInt::from(5))),
            gas_price_type => panic!("expected an eip1559 custom gas price, got {gas_price_type:?}"),
        }

        let capped = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: BigInt::from(3) }).unwrap();
        match capped.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price, priority_fee), (BigInt::from(0), BigInt::from(3))),
            gas_price_type => panic!("expected a capped eip1559 gas price, got {gas_price_type:?}"),
        }

        let without_normal = select_fee_rate(&[rate(FeePriority::Fast, "9")], &GemConfirmFeeSelection::Custom { gas_price: BigInt::from(4) }).unwrap();
        assert_eq!(without_normal.priority, FeePriority::Fast);
        match without_normal.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, BigInt::from(4)),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&[], &GemConfirmFeeSelection::Custom { gas_price: BigInt::from(1) }) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_a_custom_gas_price_carries_a_big_integer_so_a_malformed_one_cannot_read_as_zero() {
        let rates = vec![rate(FeePriority::Normal, "10")];
        let selection = GemConfirmFeeSelection::Custom { gas_price: GemBigInt::from(33) };
        match select_fee_rate(&rates, &selection).unwrap().gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, BigInt::from(33)),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }
    }

    #[test]
    fn test_broadcast_policy() {
        let transfer = GemTransactionInputType::Transfer { asset: Asset::mock_sol() };
        let swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        let payment = GemTransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata: ApplicationMetadata::mock(),
            extra: TransferDataExtra::mock().into(),
        };

        let approve = GemTransactionInputType::TokenApprove {
            asset: Asset::mock_sol(),
            approval_data: ApprovalData::mock(),
        };
        let stake = GemTransactionInputType::Stake {
            asset: Asset::mock_sol(),
            stake_type: StakeType::Rewards(vec![]),
        };
        let perpetual = GemTransactionInputType::Perpetual {
            asset: Asset::mock_sol(),
            perpetual_type: PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
        };
        let ethereum_swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock(),
            to_asset: Asset::mock_erc20(),
            swap_data: SwapData::mock(),
        };

        assert!(broadcast_options(Chain::Solana, &swap).skip_preflight);
        assert!(broadcast_options(Chain::Solana, &payment).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &transfer).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &approve).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &stake).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &perpetual).skip_preflight);
        assert!(!broadcast_options(Chain::Ethereum, &payment).skip_preflight);
        assert!(!broadcast_options(Chain::Ethereum, &ethereum_swap).skip_preflight);

        assert_eq!(broadcast_delay_milliseconds(Chain::Ethereum), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::HyperCore), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::Solana), 500);
        assert_eq!(broadcast_delay_milliseconds(Chain::Bitcoin), 500);
        assert_eq!(broadcast_delay_milliseconds(Chain::Polygon), 0);
    }

    #[test]
    fn test_scan_payload_covers_every_input_type() {
        let swap = GemTransactionPreloadInput {
            input_type: GemTransactionInputType::Swap {
                from_asset: Asset::mock_sol(),
                to_asset: Asset::mock_spl_token(),
                swap_data: SwapData::mock(),
            },
            sender_address: "sender".to_string(),
            destination_address: "router".to_string(),
            references: vec![],
        };
        let payload = scan_payload(swap);
        assert_eq!(payload.transaction_type, TransactionType::Swap);
        assert_eq!(payload.origin.asset_id, Asset::mock_sol().id);
        assert_eq!(payload.target.asset_id, Asset::mock_spl_token().id);
        assert_eq!(payload.target.address, "router");
        assert_eq!(payload.website, None);

        let generic = GemTransactionPreloadInput {
            input_type: GemTransactionInputType::Generic {
                asset: Asset::mock_sol(),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock().into(),
            },
            sender_address: "sender".to_string(),
            destination_address: "contract".to_string(),
            references: vec![],
        };
        let payload = scan_payload(generic);
        assert_eq!(payload.transaction_type, TransferDataExtra::mock().transaction_type);
        assert_eq!(payload.website, Some(ApplicationMetadata::mock().url));
    }

    #[test]
    fn test_simulation_payload_only_for_utf8_payment_calls() {
        let mut extra = TransferDataExtra::mock();
        extra.data = Some(b"0xdeadbeef".to_vec());
        let mut metadata = ApplicationMetadata::mock();
        metadata.source = ApplicationMetadataSource::Payment;
        let generic = |metadata: ApplicationMetadata, extra: TransferDataExtra| GemTransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata,
            extra: extra.into(),
        };

        assert_eq!(simulation_payload(&generic(metadata.clone(), extra.clone())), Some("0xdeadbeef".to_string()));

        let mut wallet_connect = metadata.clone();
        wallet_connect.source = ApplicationMetadataSource::WalletConnect;
        assert_eq!(simulation_payload(&generic(wallet_connect, extra.clone())), None);

        let mut binary = extra.clone();
        binary.data = Some(vec![0xff, 0xfe]);
        assert_eq!(simulation_payload(&generic(metadata.clone(), binary)), None);

        let mut empty = extra;
        empty.data = None;
        assert_eq!(simulation_payload(&generic(metadata, empty)), None);

        let swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(simulation_payload(&swap), None);
    }

    #[test]
    fn test_pending_transactions_follow_broadcast_hashes() {
        let wallet = primitives::Wallet {
            id: primitives::WalletId::Multicoin("wallet".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: primitives::WalletType::Multicoin,
            accounts: vec![Account {
                chain: Chain::Solana,
                address: "sender".to_string(),
                derivation_path: String::new(),
                extended_public_key: None,
            }],
            is_pinned: false,
            image_url: None,
            source: primitives::WalletSource::Import,
        };
        let input = GemSendInput {
            wallet: wallet.clone(),
            confirm: GemConfirmData {
                input: GemConfirmInput {
                    from: Account::mock(Chain::Solana, "sender"),
                    transfer: GemTransferData {
                        input_type: GemTransactionInputType::Transfer { asset: Asset::mock_sol() },
                        recipient: crate::services::transfer::GemRecipient {
                            address: "recipient".to_string(),
                            name: None,
                            memo: None,
                            references: vec![],
                        },
                        value: BigInt::from(10),
                        use_max_amount: false,
                        minimum_value: None,
                    },
                },
                fee: primitives::TransactionFee::new_from_fee(BigInt::from(1), AssetId::from_chain(Chain::Solana)).into(),
                selected_priority: FeePriority::Normal,
                fee_rates: vec![],
                metadata: GemTransactionLoadMetadata::None,
                simulation: None,
            },
            value: BigInt::from(10),
            network_fee: BigInt::from(1),
            simulation: None,
        };
        let signed = vec![GemSignedTransaction {
            data: "signed".to_string(),
            transaction_type: primitives::TransactionType::Transfer,
        }];
        let transactions = pending_transactions(&input, &["hash".to_string()], &signed).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id.hash, "hash");
        assert_eq!(transactions[0].from, "sender");

        let mut no_account = input.clone();
        no_account.wallet.accounts.clear();
        assert!(matches!(
            pending_transactions(&no_account, &["hash".to_string()], &signed),
            Err(GemConfirmError::Record { .. })
        ));
    }

    #[test]
    fn test_acquire_asset_flow_offers_options_only_on_tron() {
        assert_eq!(acquire_asset_flow(Chain::Tron), GemAcquireAssetFlow::Options);
        assert_eq!(acquire_asset_flow(Chain::Ethereum), GemAcquireAssetFlow::Fiat);
    }

    #[test]
    fn test_default_fee_priority_is_fast_only_for_bitcoin_swaps() {
        let bitcoin_swap = GemTransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Bitcoin),
            to_asset: Asset::mock_sol(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(default_fee_priority(bitcoin_swap), FeePriority::Fast);
        let solana_swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(default_fee_priority(solana_swap), FeePriority::Normal);
        assert_eq!(
            default_fee_priority(GemTransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Bitcoin)
            }),
            FeePriority::Normal
        );
    }

    #[test]
    fn test_insufficient_network_fee_only_for_empty_native_balances() {
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "0"));
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), ""));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "10"));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Tron), "0"));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::HyperCore), "0"));
        assert!(!is_insufficient_network_fee(
            AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".into())),
            "0"
        ));
    }

    #[test]
    fn test_validate_scan() {
        let safe = ScanTransaction {
            is_malicious: Some(false),
            is_memo_required: Some(false),
            is_scan_complete: false,
            malicious_addresses: None,
            malicious_assets: None,
            malicious_website: None,
        };
        let malicious = ScanTransaction {
            is_malicious: Some(true),
            is_memo_required: Some(false),
            is_scan_complete: false,
            malicious_addresses: None,
            malicious_assets: None,
            malicious_website: None,
        };
        let memo_required = ScanTransaction {
            is_malicious: Some(false),
            is_memo_required: Some(true),
            is_scan_complete: false,
            malicious_addresses: None,
            malicious_assets: None,
            malicious_website: None,
        };

        assert!(validate_scan(None, None, "USDT").is_ok());
        assert!(validate_scan(Some(&safe), None, "USDT").is_ok());
        assert!(validate_scan(Some(&memo_required), Some("deposit"), "USDT").is_ok());

        match validate_scan(Some(&malicious), Some("memo"), "USDT") {
            Err(GemConfirmError::ScanMalicious) => {}
            result => panic!("expected a malicious verdict, got {result:?}"),
        }
        match validate_scan(Some(&memo_required), Some("  "), "USDT") {
            Err(GemConfirmError::ScanMemoRequired { symbol }) => assert_eq!(symbol, "USDT"),
            result => panic!("expected a required memo, got {result:?}"),
        }
    }

    #[test]
    fn test_a_fee_asset_with_no_available_balance_is_not_selectable() {
        let funded = Asset::from_chain(Chain::Tempo);
        let empty = Asset::from_chain(Chain::Ethereum);
        let assets = vec![funded.clone(), empty.clone()];
        let balances = vec![balance(&funded.id, 1), balance(&empty.id, 0)];

        let selectable = selectable_fee_assets(assets, balances, vec![]);

        assert_eq!(selectable.iter().map(|fee| fee.asset.id.clone()).collect::<Vec<_>>(), vec![funded.id]);
    }

    #[test]
    fn test_preload_reports_the_amount_error_instead_of_failing_the_whole_preload() {
        let chain = Chain::Solana;
        let asset = Asset::from_chain(chain);
        let mut data = send_input_from(chain, GemTransactionInputType::Transfer { asset: asset.clone() }, "sender").confirm;
        data.input.transfer.use_max_amount = false;
        data.input.transfer.value = BigInt::from(1_000);
        data.fee.fee = BigInt::from(1);

        let short = GemConfirmMetadata {
            asset_balance: balance(&asset.id, 10),
            fee_asset_balance: balance(&asset.id, 10),
            prices: vec![],
        };
        let funded = GemConfirmMetadata {
            asset_balance: balance(&asset.id, 2_000_000),
            fee_asset_balance: balance(&asset.id, 2_000_000),
            prices: vec![],
        };

        assert!(matches!(preload_amount(&data, &short, &asset).unwrap(), GemTransferAmountResult::Error { .. }));
        assert!(matches!(preload_amount(&data, &funded, &asset).unwrap(), GemTransferAmountResult::Amount { .. }));
    }

    fn balance(asset_id: &AssetId, available: u32) -> GemAssetBalance {
        GemAssetBalance {
            asset_id: asset_id.clone(),
            available: GemBigUint::from(available),
            frozen: GemBigUint::ZERO,
            locked: GemBigUint::ZERO,
            staked: GemBigUint::ZERO,
            pending: GemBigUint::ZERO,
            pending_unconfirmed: GemBigUint::ZERO,
            rewards: GemBigUint::ZERO,
            reserved: GemBigUint::ZERO,
            withdrawable: GemBigUint::ZERO,
            earn: GemBigUint::ZERO,
            metadata: None,
        }
    }

    #[test]
    fn test_metadata_asset_ids_keeps_one_entry_per_asset() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);
        let extra = AssetId::from_chain(Chain::Bitcoin);

        let asset_ids = metadata_asset_ids(&asset_id, &fee_asset_id, vec![extra.clone(), extra.clone(), asset_id.clone()]);

        assert_eq!(asset_ids, vec![asset_id, extra]);
    }

    #[test]
    fn test_build_metadata_reads_each_balance_from_its_own_asset() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);
        let balances = vec![balance(&fee_asset_id, 7), balance(&asset_id, 3)];

        let metadata = build_metadata(asset_id.clone(), fee_asset_id.clone(), balances, vec![]).unwrap();

        assert_eq!(metadata.asset_balance.available, GemBigUint::from(3u32));
        assert_eq!(metadata.fee_asset_balance.available, GemBigUint::from(7u32));
    }

    #[test]
    fn test_build_metadata_rejects_a_missing_balance() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let fee_asset_id = AssetId::from_chain(Chain::Ethereum);

        match build_metadata(asset_id.clone(), fee_asset_id.clone(), vec![balance(&fee_asset_id, 7)], vec![]) {
            Err(GemConfirmError::BalanceMissing { asset_id: missing }) => assert_eq!(missing, asset_id),
            result => panic!("expected the asset balance to be required, got {result:?}"),
        }
        match build_metadata(asset_id.clone(), fee_asset_id.clone(), vec![balance(&asset_id, 3)], vec![]) {
            Err(GemConfirmError::BalanceMissing { asset_id: missing }) => assert_eq!(missing, fee_asset_id),
            result => panic!("expected the fee balance to be required, got {result:?}"),
        }
    }
}
