use num_bigint::BigInt;
use primitives::{
    ApplicationMetadataSource, AssetId, Chain, ChainType, FeePriority, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, Transaction, TransactionPreloadInput,
    TransferDataOutputAction,
};

use super::error::GemConfirmError;
use super::model::{GemAcquireAssetFlow, GemConfirmFeeSelection, GemSendInput};
use crate::fee::custom_gas_price;
use crate::models::gateway::{GemBroadcastOptions, GemFeeRate, GemTransactionPreloadInput};
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadInput};
use crate::services::transfer::{GemPendingTransactionInput, rules as transfer_rules};

pub fn signer_input(input: &GemSendInput) -> Result<GemSignerInput, GemConfirmError> {
    let chain = input.transfer.input_type.asset().chain();
    let sender = input.wallet.account(chain).ok_or(GemConfirmError::AccountMissing { chain })?;
    Ok(GemSignerInput {
        input: GemTransactionLoadInput {
            input_type: input.transfer.input_type.clone(),
            sender_address: sender.address.clone(),
            destination_address: input.transfer.recipient.address.clone(),
            value: input.value.clone(),
            gas_price: input.fee.gas_price_type.clone(),
            memo: input.transfer.recipient.memo.clone(),
            is_max_value: input.transfer.use_max_amount,
            metadata: input.metadata.clone(),
        },
        fee: GemTransactionLoadFee {
            fee: input.network_fee.clone(),
            ..input.fee.clone()
        },
    })
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

#[uniffi::export]
pub fn acquire_asset_flow(chain: Chain) -> GemAcquireAssetFlow {
    match chain {
        Chain::Tron => GemAcquireAssetFlow::Options,
        _ => GemAcquireAssetFlow::Fiat,
    }
}

#[uniffi::export]
pub fn default_fee_priority(input_type: GemTransactionInputType) -> String {
    let priority = match input_type {
        GemTransactionInputType::Swap { from_asset, .. } if from_asset.chain() == Chain::Bitcoin => FeePriority::Fast,
        _ => FeePriority::Normal,
    };
    priority.as_ref().to_string()
}

#[uniffi::export]
pub fn is_insufficient_network_fee(fee_asset_id: AssetId, fee_available: String) -> bool {
    if matches!(fee_asset_id.chain, Chain::HyperCore | Chain::Tron) || !fee_asset_id.is_native() {
        return false;
    }
    fee_available.trim().is_empty() || fee_available.trim().chars().all(|character| character == '0')
}

pub(super) fn pending_transactions(input: &GemSendInput, hashes: &[String], transactions: &[GemSignedTransaction]) -> Result<Vec<Transaction>, GemConfirmError> {
    let chain = input.transfer.input_type.asset().chain();
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
                    transfer: input.transfer.clone(),
                    value: input.value.clone(),
                    transaction_type,
                    hash: hash.clone(),
                    fee: input.fee.clone(),
                    network_fee: input.network_fee.clone(),
                    metadata: input.metadata.clone(),
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
    if scan.is_malicious {
        return Err(GemConfirmError::ScanMalicious);
    }
    if scan.is_memo_required && memo.unwrap_or_default().trim().is_empty() {
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
                .find(|rate| rate.priority == FeePriority::Normal.as_ref())
                .or_else(|| rates.first())
                .ok_or(GemConfirmError::FeeRatesMissing)?;
            let gas_price = gas_price.parse::<BigInt>().map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
            Ok(GemFeeRate {
                priority: base.priority.clone(),
                gas_price_type: custom_gas_price(base.gas_price_type.clone(), gas_price),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::gateway::GemGasPriceType;
    use crate::models::transaction::GemTransactionLoadMetadata;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use primitives::{
        Account, ApplicationMetadata, Asset, PerpetualConfirmData, PerpetualDirection, PerpetualType, StakeType, TransactionType, TransferDataExtra, Wallet, WalletId,
        WalletSource, WalletType,
        swap::{ApprovalData, SwapData},
    };

    fn wallet(chain: Chain) -> Wallet {
        Wallet {
            id: WalletId::Multicoin("wallet".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts: vec![Account {
                chain,
                address: "sender".to_string(),
                derivation_path: String::new(),
                extended_public_key: None,
            }],
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        }
    }

    fn send_input(chain: Chain, input_type: GemTransactionInputType) -> GemSendInput {
        GemSendInput {
            wallet: wallet(chain),
            transfer: GemTransferData {
                input_type,
                recipient: GemRecipient {
                    address: "recipient".to_string(),
                    name: None,
                    memo: Some("memo".to_string()),
                    references: vec![],
                },
                value: "10".to_string(),
                use_max_amount: true,
                minimum_value: None,
            },
            value: "9".to_string(),
            fee: GemTransactionLoadFee {
                fee: "0".to_string(),
                gas_price_type: GemGasPriceType::Regular { gas_price: "5".to_string() },
                gas_limit: "21000".to_string(),
                options: Default::default(),
                fee_asset: AssetId::from_chain(Chain::Solana),
            },
            network_fee: "1".to_string(),
            metadata: GemTransactionLoadMetadata::None,
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
        assert_eq!(signer_input.fee.fee, "1");
        assert_eq!(signer_input.fee.gas_limit, "21000");
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
    fn rate(priority: &str, gas_price: &str) -> GemFeeRate {
        GemFeeRate {
            priority: priority.to_string(),
            gas_price_type: GemGasPriceType::Regular { gas_price: gas_price.to_string() },
        }
    }

    #[test]
    fn test_select_fee_rate() {
        let rates = vec![rate("normal", "10"), rate("fast", "20")];

        let fast = select_fee_rate(&rates, &GemConfirmFeeSelection::Priority { priority: "fast".to_string() }).unwrap();
        assert_eq!(fast.priority, "fast");

        let fallback = select_fee_rate(&rates, &GemConfirmFeeSelection::Priority { priority: "slow".to_string() }).unwrap();
        assert_eq!(fallback.priority, "normal");

        let custom = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "33".to_string() }).unwrap();
        assert_eq!(custom.priority, "normal");
        match custom.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, "33"),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&[], &GemConfirmFeeSelection::Priority { priority: "normal".to_string() }) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_select_fee_rate_custom() {
        let eip1559 = GemFeeRate {
            priority: "normal".to_string(),
            gas_price_type: GemGasPriceType::Eip1559 {
                gas_price: "20".to_string(),
                priority_fee: "5".to_string(),
            },
        };
        let rates = vec![rate("slow", "1"), eip1559];

        let raised = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "30".to_string() }).unwrap();
        assert_eq!(raised.priority, "normal");
        match raised.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price.as_str(), priority_fee.as_str()), ("25", "5")),
            gas_price_type => panic!("expected an eip1559 custom gas price, got {gas_price_type:?}"),
        }

        let capped = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "3".to_string() }).unwrap();
        match capped.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price.as_str(), priority_fee.as_str()), ("0", "3")),
            gas_price_type => panic!("expected a capped eip1559 gas price, got {gas_price_type:?}"),
        }

        let without_normal = select_fee_rate(&[rate("slow", "1"), rate("fast", "9")], &GemConfirmFeeSelection::Custom { gas_price: "4".to_string() }).unwrap();
        assert_eq!(without_normal.priority, "slow");
        match without_normal.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, "4"),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "abc".to_string() }) {
            Err(GemConfirmError::Load { .. }) => {}
            result => panic!("expected a load error for a malformed gas price, got {result:?}"),
        }
        match select_fee_rate(&[], &GemConfirmFeeSelection::Custom { gas_price: "1".to_string() }) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
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
            transfer: GemTransferData {
                input_type: GemTransactionInputType::Transfer { asset: Asset::mock_sol() },
                recipient: crate::services::transfer::GemRecipient {
                    address: "recipient".to_string(),
                    name: None,
                    memo: None,
                    references: vec![],
                },
                value: "10".to_string(),
                use_max_amount: false,
                minimum_value: None,
            },
            value: "10".to_string(),
            fee: primitives::TransactionFee::new_from_fee(BigInt::from(1), AssetId::from_chain(Chain::Solana)).into(),
            network_fee: "1".to_string(),
            metadata: GemTransactionLoadMetadata::None,
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
        assert_eq!(default_fee_priority(bitcoin_swap), FeePriority::Fast.as_ref());
        let solana_swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(default_fee_priority(solana_swap), FeePriority::Normal.as_ref());
        assert_eq!(
            default_fee_priority(GemTransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Bitcoin)
            }),
            FeePriority::Normal.as_ref()
        );
    }

    #[test]
    fn test_insufficient_network_fee_only_for_empty_native_balances() {
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "0".into()));
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "10".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Tron), "0".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::HyperCore), "0".into()));
        assert!(!is_insufficient_network_fee(
            AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".into())),
            "0".into()
        ));
    }

    #[test]
    fn test_validate_scan() {
        let safe = ScanTransaction {
            is_malicious: false,
            is_memo_required: false,
        };
        let malicious = ScanTransaction {
            is_malicious: true,
            is_memo_required: false,
        };
        let memo_required = ScanTransaction {
            is_malicious: false,
            is_memo_required: true,
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
}
