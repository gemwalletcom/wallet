use gem_evm::transaction::{decode_nft_collection_approval_operator, decode_token_transfer_recipient};
use gem_tron::decode_wallet_connect_token_recipient;
use primitives::{
    AssetId, Chain, ChainType, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, SimulationPayloadFieldKind, SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType, TransactionInputType,
    TransactionPreloadInput, TransactionType, TransferDataExtra,
};

fn requires_scan(input_type: &TransactionInputType) -> bool {
    match input_type {
        TransactionInputType::Transfer { .. } | TransactionInputType::Swap { .. } | TransactionInputType::TokenApprove { .. } | TransactionInputType::Generic { .. } | TransactionInputType::Payment { .. } => true,
        TransactionInputType::Deposit { .. }
        | TransactionInputType::Withdrawal { .. }
        | TransactionInputType::Stake { .. }
        | TransactionInputType::TransferNft { .. }
        | TransactionInputType::Account { .. }
        | TransactionInputType::Perpetual { .. }
        | TransactionInputType::Earn { .. } => false,
    }
}

pub fn transaction_payload(input: &TransactionPreloadInput) -> Option<ScanTransactionPayload> {
    if !requires_scan(&input.input_type) {
        return None;
    }
    Some(ScanTransactionPayload {
        origin: ScanAddressTarget {
            asset_id: input.input_type.get_asset().id.clone(),
            address: input.sender_address.clone(),
        },
        target: scan_target(input),
        website: input.get_website(),
        transaction_type: input.input_type.transaction_type(),
    })
}

fn scan_target(input: &TransactionPreloadInput) -> ScanAddressTarget {
    if let TransactionInputType::Swap { from_asset, swap_data, .. } = &input.input_type
        && !swap_data.data.to.is_empty()
    {
        return ScanAddressTarget {
            asset_id: from_asset.id.clone(),
            address: swap_data.data.to.clone(),
        };
    }
    if let TransactionInputType::Generic { asset, extra, .. } = &input.input_type
        && let Some(approval) = &extra.approval
    {
        return ScanAddressTarget {
            asset_id: AssetId::from_token(asset.chain(), &approval.token),
            address: approval.spender.clone(),
        };
    }
    if let TransactionInputType::Generic { asset, extra, .. } = &input.input_type
        && let Some(target) = contract_call_target(asset.chain(), extra)
    {
        return target;
    }
    ScanAddressTarget {
        asset_id: input.input_type.get_recipient_asset().id.clone(),
        address: input.destination_address.clone(),
    }
}

fn contract_call_target(chain: Chain, extra: &TransferDataExtra) -> Option<ScanAddressTarget> {
    let data = extra.data.as_deref()?;
    let address = match chain.chain_type() {
        ChainType::Ethereum => decode_nft_collection_approval_operator(data).or_else(|| decode_token_transfer_recipient(data))?.to_string(),
        ChainType::Tron => decode_wallet_connect_token_recipient(std::str::from_utf8(data).ok()?).ok().flatten()?,
        _ => return None,
    };
    Some(ScanAddressTarget {
        asset_id: AssetId::from_token(chain, &extra.to),
        address,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignMessageVerdict {
    Allowed,
    SuspiciousSpender,
    MaliciousWebsite,
}

pub fn sign_message_payload(chain: Chain, account_address: &str, simulation: &SimulationResult, domain: String) -> ScanTransactionPayload {
    let asset_id = simulation.header.as_ref().map(|header| header.asset_id.clone()).unwrap_or_else(|| chain.as_asset_id());
    let spender = simulation.payload.iter().find(|field| field.kind == SimulationPayloadFieldKind::Spender).map(|field| field.value.clone());
    let transaction_type = match spender {
        Some(_) => TransactionType::TokenApproval,
        None => TransactionType::SmartContractCall,
    };
    ScanTransactionPayload {
        origin: ScanAddressTarget {
            asset_id: asset_id.clone(),
            address: account_address.to_string(),
        },
        target: ScanAddressTarget {
            asset_id,
            address: spender.unwrap_or_default(),
        },
        website: Some(domain),
        transaction_type,
    }
}

pub fn sign_message_verdict(scan: Option<&ScanTransaction>, payload: &ScanTransactionPayload) -> SignMessageVerdict {
    let Some(scan) = scan else {
        return SignMessageVerdict::Allowed;
    };
    if scan.malicious_website.is_some() {
        return SignMessageVerdict::MaliciousWebsite;
    }
    let target = &payload.target;
    let is_spender_flagged = !target.address.is_empty() && scan.malicious_addresses.iter().flatten().any(|address| address.address.eq_ignore_ascii_case(&target.address));
    let is_token_flagged = scan.malicious_assets.iter().flatten().any(|asset_id| *asset_id == target.asset_id && asset_id.token_id.is_some());
    match is_spender_flagged || is_token_flagged {
        true => SignMessageVerdict::SuspiciousSpender,
        false => SignMessageVerdict::Allowed,
    }
}

pub fn with_suspicious_spender(mut simulation: SimulationResult) -> SimulationResult {
    simulation.warnings.insert(
        0,
        SimulationWarning {
            severity: SimulationSeverity::Critical,
            warning: SimulationWarningType::SuspiciousSpender,
            message: None,
        },
    );
    simulation
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::swap::ApprovalData;
    use primitives::{
        AccountDataType, ApplicationMetadata, Asset, ContractCallData, Delegation, DelegationValidator, EarnType, NFTAsset, PerpetualConfirmData, PerpetualDirection, PerpetualType, StakeType, SwapData, SwapQuoteData, TransactionType,
        TransferDataExtra,
    };
    use primitives::{AssetId, SimulationHeader, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldType};

    #[test]
    fn test_a_swap_scans_the_contract_that_receives_the_funds() {
        let swap = TransactionPreloadInput {
            input_type: TransactionInputType::Swap {
                from_asset: Asset::mock_eth(),
                to_asset: Asset::mock_spl_token(),
                swap_data: SwapData {
                    data: SwapQuoteData {
                        to: "0xrouter".to_string(),
                        ..SwapQuoteData::mock()
                    },
                    ..SwapData::mock()
                },
            },
            sender_address: "sender".to_string(),
            destination_address: "own-solana-address".to_string(),
            references: vec![],
        };

        let target = transaction_payload(&swap).unwrap().target;
        assert_eq!(target.address, "0xrouter", "the scan asks about the contract, not the address the user already owns");
        assert_eq!(target.asset_id, Asset::mock_eth().id, "the contract is on the chain the funds leave");

        let without_contract = transaction_payload(&TransactionPreloadInput {
            input_type: TransactionInputType::Swap {
                from_asset: Asset::mock_eth(),
                to_asset: Asset::mock_spl_token(),
                swap_data: SwapData {
                    data: SwapQuoteData { to: String::new(), ..SwapQuoteData::mock() },
                    ..SwapData::mock()
                },
            },
            destination_address: "recipient".to_string(),
            ..swap
        })
        .unwrap()
        .target;
        assert_eq!(without_contract.address, "recipient", "a swap with no contract falls back to the recipient");
    }

    #[test]
    fn test_a_dapp_approval_scans_the_spender_and_the_token() {
        let approval = TransactionPreloadInput {
            input_type: TransactionInputType::Generic {
                asset: Asset::mock_eth(),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra {
                    transaction_type: TransactionType::TokenApproval,
                    approval: Some(ApprovalData {
                        token: "0xtoken".to_string(),
                        spender: "0xspender".to_string(),
                        ..ApprovalData::mock()
                    }),
                    ..TransferDataExtra::mock()
                },
            },
            sender_address: "sender".to_string(),
            destination_address: "0xpermit2".to_string(),
            references: vec![],
        };

        let target = transaction_payload(&approval).unwrap().target;
        assert_eq!(target.address, "0xspender", "the scan asks about who can spend, not the contract called");
        assert_eq!(target.asset_id, AssetId::from_token(Chain::Ethereum, "0xtoken"));
    }

    #[test]
    fn test_contract_call_target() {
        let token = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        let recipient = "0x2222222222222222222222222222222222222222";
        let recipient_word = format!("{:0>64}", &recipient[2..]);
        let amount_word = format!("{:0>64}", "64");
        let recipient_target = ScanAddressTarget {
            asset_id: AssetId::from_token(Chain::Ethereum, token),
            address: recipient.to_string(),
        };
        for (calldata, expected) in [
            (format!("a9059cbb{recipient_word}{amount_word}"), Some(recipient_target.clone())),
            (format!("23b872dd{:0>64}{recipient_word}{amount_word}", "11"), Some(recipient_target.clone())),
            (format!("a22cb465{recipient_word}{:0>64}", "1"), Some(recipient_target)),
            (format!("a22cb465{recipient_word}{:0>64}", "0"), None),
            (format!("70a08231{recipient_word}"), None),
        ] {
            let extra = TransferDataExtra {
                to: token.to_string(),
                ..TransferDataExtra::mock_encoded_transaction(primitives::hex::decode(&calldata).unwrap())
            };
            assert_eq!(contract_call_target(Chain::Ethereum, &extra), expected, "{calldata}");
        }

        let tron_usdt = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
        let transfer = TransferDataExtra {
            to: tron_usdt.to_string(),
            ..TransferDataExtra::mock_encoded_transaction(include_bytes!("../../../../crates/gem_tron/testdata/wallet_connect_trc20_transfer.json").to_vec())
        };
        let expected = ScanAddressTarget {
            asset_id: AssetId::from_token(Chain::Tron, tron_usdt),
            address: "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7".to_string(),
        };
        assert_eq!(contract_call_target(Chain::Tron, &transfer), Some(expected));

        let approval = TransferDataExtra {
            to: tron_usdt.to_string(),
            ..TransferDataExtra::mock_encoded_transaction(include_bytes!("../../../../crates/gem_wallet_connect/testdata/tron_send_transaction.json").to_vec())
        };
        assert_eq!(contract_call_target(Chain::Tron, &approval), None);
    }

    #[test]
    fn test_scan_payload_covers_every_input_type() {
        let swap = TransactionPreloadInput {
            input_type: TransactionInputType::Swap {
                from_asset: Asset::mock_sol(),
                to_asset: Asset::mock_spl_token(),
                swap_data: SwapData::mock(),
            },
            sender_address: "sender".to_string(),
            destination_address: "router".to_string(),
            references: vec![],
        };
        let payload = transaction_payload(&swap).unwrap();
        assert_eq!(payload.transaction_type, TransactionType::Swap);
        assert_eq!(payload.origin.asset_id, Asset::mock_sol().id);
        assert_eq!(payload.website, None);

        let generic = TransactionPreloadInput {
            input_type: TransactionInputType::Generic {
                asset: Asset::mock_sol(),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock(),
            },
            sender_address: "sender".to_string(),
            destination_address: "contract".to_string(),
            references: vec![],
        };
        let payload = transaction_payload(&generic).unwrap();
        assert_eq!(payload.transaction_type, TransferDataExtra::mock().transaction_type);
        assert_eq!(payload.website, Some(ApplicationMetadata::mock().url));
    }

    #[test]
    fn test_requires_scan() {
        for (input_type, should_scan) in [
            (TransactionInputType::Deposit { asset: Asset::mock_erc20() }, false),
            (
                TransactionInputType::Earn {
                    asset: Asset::mock_erc20(),
                    earn_type: EarnType::Deposit(DelegationValidator::mock()),
                    data: ContractCallData::mock(),
                },
                false,
            ),
            (TransactionInputType::Transfer { asset: Asset::mock_erc20() }, true),
            (TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock()), true),
            (TransactionInputType::Withdrawal { asset: Asset::mock_erc20() }, false),
            (
                TransactionInputType::Earn {
                    asset: Asset::mock_erc20(),
                    earn_type: EarnType::Withdraw(Delegation::mock()),
                    data: ContractCallData::mock(),
                },
                false,
            ),
            (
                TransactionInputType::Account {
                    asset: Asset::mock_erc20(),
                    account_type: AccountDataType::Activate,
                },
                false,
            ),
            (
                TransactionInputType::Stake {
                    asset: Asset::mock_sol(),
                    stake_type: StakeType::Rewards(vec![]),
                },
                false,
            ),
            (
                TransactionInputType::TransferNft {
                    asset: Asset::mock_eth(),
                    nft_asset: NFTAsset::mock(),
                },
                false,
            ),
            (
                TransactionInputType::Perpetual {
                    asset: Asset::mock_erc20(),
                    perpetual_type: PerpetualType::Open {
                        data: PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None),
                    },
                },
                false,
            ),
            (
                TransactionInputType::Swap {
                    from_asset: Asset::mock_eth(),
                    to_asset: Asset::mock_erc20(),
                    swap_data: SwapData::mock(),
                },
                true,
            ),
            (
                TransactionInputType::TokenApprove {
                    asset: Asset::mock_erc20(),
                    approval_data: ApprovalData::mock(),
                },
                true,
            ),
            (
                TransactionInputType::Generic {
                    asset: Asset::mock_erc20(),
                    metadata: ApplicationMetadata::mock(),
                    extra: TransferDataExtra {
                        transaction_type: TransactionType::EarnDeposit,
                        ..TransferDataExtra::mock()
                    },
                },
                true,
            ),
        ] {
            assert_eq!(requires_scan(&input_type), should_scan);
        }
    }

    fn permit(spender: Option<&str>) -> SimulationResult {
        let usdt = AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7");
        SimulationResult {
            warnings: vec![],
            balance_changes: vec![],
            payload: spender
                .map(|spender| {
                    vec![SimulationPayloadField::standard(
                        SimulationPayloadFieldKind::Spender,
                        spender,
                        SimulationPayloadFieldType::Address,
                        SimulationPayloadFieldDisplay::Primary,
                    )]
                })
                .unwrap_or_default(),
            header: spender.map(|_| SimulationHeader {
                asset_id: usdt,
                value: None,
                is_unlimited: true,
            }),
        }
    }

    #[test]
    fn test_a_permit_scans_its_spender_and_token_and_a_plain_message_only_the_site() {
        let approval = sign_message_payload(Chain::Ethereum, "0xowner", &permit(Some("0xspender")), "https://dapp.example".to_string());
        assert_eq!(approval.target.address, "0xspender");
        assert_eq!(approval.target.asset_id.token_id.as_deref(), Some("0xdAC17F958D2ee523a2206206994597C13D831ec7"));
        assert_eq!(approval.origin.address, "0xowner");
        assert_eq!(approval.transaction_type, TransactionType::TokenApproval);
        assert_eq!(approval.website.as_deref(), Some("https://dapp.example"));

        let message = sign_message_payload(Chain::Ethereum, "0xowner", &permit(None), "https://dapp.example".to_string());
        assert_eq!(message.target.address, "");
        assert_eq!(message.target.asset_id, Chain::Ethereum.as_asset_id());
        assert_eq!(message.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(message.website.as_deref(), Some("https://dapp.example"));
    }

    #[test]
    fn test_the_verdict_rejects_a_site_and_warns_on_a_flagged_spender_or_token() {
        let payload = sign_message_payload(Chain::Ethereum, "0xowner", &permit(Some("0xSpender")), "https://dapp.example".to_string());
        let scan = |json: &str| serde_json::from_str::<ScanTransaction>(json).unwrap();

        assert_eq!(sign_message_verdict(None, &payload), SignMessageVerdict::Allowed);
        assert_eq!(sign_message_verdict(Some(&scan(r#"{"maliciousWebsite":"https://dapp.example"}"#)), &payload), SignMessageVerdict::MaliciousWebsite);
        assert_eq!(
            sign_message_verdict(Some(&scan(r#"{"maliciousAddresses":[{"chain":"ethereum","address":"0xspender"}]}"#)), &payload),
            SignMessageVerdict::SuspiciousSpender
        );
        assert_eq!(
            sign_message_verdict(Some(&scan(r#"{"maliciousAssets":["ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7"]}"#)), &payload),
            SignMessageVerdict::SuspiciousSpender
        );
        assert_eq!(sign_message_verdict(Some(&scan(r#"{"isScanComplete":true}"#)), &payload), SignMessageVerdict::Allowed);
        assert_eq!(with_suspicious_spender(permit(None)).warnings[0].severity, SimulationSeverity::Critical);
    }
}
