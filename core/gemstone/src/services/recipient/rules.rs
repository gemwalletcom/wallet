use primitives::name::NameRecord;
use primitives::{Asset, Chain, ChainAsset, Wallet, WalletType};

use super::model::{GemRecipientError, GemRecipientErrorDisplay, GemRecipientNext, GemRecipientScan, GemRecipientSection, GemRecipientType, GemRecipientValidation};
use crate::address::{checksum_address, validate_address};
use crate::models::custom_types::GemBigInt;
use crate::payment::{GemPaymentConfirmTransfer, GemPaymentDestination, GemPaymentRecipient};
use crate::services::name::GemNameRecordState;
use crate::services::name::rules::is_name_supported;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::TransactionInputType;

pub fn validation(chain: Chain, input: &str, state: &GemNameRecordState) -> GemRecipientValidation {
    let name_record = state.record_ref();
    let is_valid = state.can_validate_recipient() && is_valid(chain, input, name_record);
    GemRecipientValidation {
        is_valid,
        address: address(chain, input, name_record),
        error: error_display(chain, input, is_valid),
    }
}

pub fn recipient(chain: Chain, input: &str, state: &GemNameRecordState, memo: Option<String>, references: Vec<String>) -> Result<GemRecipient, GemRecipientError> {
    let name_record = state.record_ref();
    if name_record.is_some_and(|record| !matches_input(record, chain, input)) {
        return Err(GemRecipientError::NameRecordMismatch);
    }
    if !state.can_validate_recipient() || !is_valid(chain, input, name_record) {
        return Err(GemRecipientError::InvalidAddress);
    }
    Ok(GemRecipient {
        address: address(chain, input, name_record),
        name: name_record.map(|record| record.name.clone()),
        memo,
        references,
    })
}

fn is_valid(chain: Chain, input: &str, name_record: Option<&NameRecord>) -> bool {
    match name_record {
        Some(record) => matches_input(record, chain, input) && validate_address(&record.address, chain),
        None => !input.trim().is_empty() && validate_address(&checksum_address(input, chain), chain),
    }
}

fn matches_input(record: &NameRecord, chain: Chain, input: &str) -> bool {
    record.name == input && record.chain == chain
}

fn address(chain: Chain, input: &str, name_record: Option<&NameRecord>) -> String {
    let address = name_record.map(|record| record.address.as_str()).filter(|address| !address.is_empty()).unwrap_or(input);
    checksum_address(address, chain)
}

fn error_display(chain: Chain, input: &str, is_valid: bool) -> Option<GemRecipientErrorDisplay> {
    let shows_error = !input.trim().is_empty() && !is_name_supported(input) && !is_valid;
    shows_error.then(|| GemRecipientErrorDisplay::InvalidAddress {
        network: ChainAsset::from_chain(chain).network_name,
    })
}

pub fn scan_route(
    destination: GemPaymentDestination,
    recipient_type: &GemRecipientType,
    transfer_data: impl FnOnce(GemPaymentConfirmTransfer) -> GemTransferData,
) -> Result<GemRecipientScan, GemRecipientError> {
    match destination {
        GemPaymentDestination::Confirm { transfer } => {
            let transfer = transfer_data(transfer);
            Ok(match recipient_type {
                GemRecipientType::Asset { .. } => GemRecipientScan::Confirm { transfer },
                GemRecipientType::Nft { .. } => GemRecipientScan::Recipient {
                    payment: GemPaymentRecipient {
                        recipient: transfer.recipient,
                        amount: None,
                    },
                },
            })
        }
        GemPaymentDestination::Recipient { payment, .. } => Ok(GemRecipientScan::Recipient { payment }),
        GemPaymentDestination::SelectAsset { .. } | GemPaymentDestination::Unsupported => Err(GemRecipientError::InvalidAddress),
    }
}

pub fn next_step(recipient_type: GemRecipientType, payment: GemPaymentRecipient) -> GemRecipientNext {
    match recipient_type {
        GemRecipientType::Asset { .. } => GemRecipientNext::Amount { payment },
        GemRecipientType::Nft { nft_asset } => GemRecipientNext::Confirm {
            transfer: GemTransferData {
                input_type: TransactionInputType::TransferNft {
                    asset: Asset::from_chain(nft_asset.chain),
                    nft_asset,
                },
                recipient: payment.recipient,
                value: GemBigInt::from(0),
                use_max_amount: false,
            },
        },
    }
}

pub fn recipient_sections(wallets: Vec<Wallet>, chain: Chain, has_contacts: bool) -> Vec<GemRecipientSection> {
    let on_chain: Vec<Wallet> = wallets.into_iter().filter(|wallet| wallet.account(chain).is_some()).collect();
    let of = |pinned: bool, view: bool| -> Vec<Wallet> {
        on_chain
            .iter()
            .filter(|wallet| wallet.is_pinned == pinned && (wallet.wallet_type == WalletType::View) == view)
            .cloned()
            .collect()
    };
    let pinned: Vec<Wallet> = on_chain.iter().filter(|wallet| wallet.is_pinned).cloned().collect();

    [
        (!pinned.is_empty()).then_some(GemRecipientSection::Pinned { wallets: pinned }),
        has_contacts.then_some(GemRecipientSection::Contacts),
        Some(of(false, false))
            .filter(|wallets| !wallets.is_empty())
            .map(|wallets| GemRecipientSection::Wallets { wallets }),
        Some(of(false, true))
            .filter(|wallets| !wallets.is_empty())
            .map(|wallets| GemRecipientSection::ViewWallets { wallets }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payment::GemPaymentService;
    use crate::testkit::TestAlienProvider;
    use std::sync::Arc;

    const ADDRESS: &str = "0x1f9090aae28b8a3dceadf281b0f12828e676c326";
    const CHECKSUMMED: &str = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326";

    #[test]
    fn test_a_pending_or_failed_name_lookup_is_never_a_valid_recipient() {
        let loading = GemNameRecordState::Loading { name: "h3rman.near".into() };
        let pending = validation(Chain::Near, "h3rman.near", &loading);
        assert!(!pending.is_valid);
        assert!(pending.error.is_none());
        assert_eq!(pending.address, "h3rman.near");
        assert!(!validation(Chain::Near, "h3rman.near", &GemNameRecordState::Error).is_valid);
        assert!(validation(Chain::Near, "h3rman.near", &GemNameRecordState::None).is_valid);
        assert_eq!(recipient(Chain::Near, "h3rman.near", &loading, None, vec![]), Err(GemRecipientError::InvalidAddress));
    }

    #[test]
    fn test_address_input_is_checksummed_and_validated() {
        let valid = validation(Chain::Ethereum, ADDRESS, &GemNameRecordState::None);
        assert!(valid.is_valid);
        assert_eq!(valid.address, CHECKSUMMED);
        assert!(valid.error.is_none());

        let invalid = validation(Chain::Ethereum, "0xinvalid", &GemNameRecordState::None);
        assert!(!invalid.is_valid);
        assert_eq!(
            invalid.error,
            Some(GemRecipientErrorDisplay::InvalidAddress {
                network: ChainAsset::from_chain(Chain::Ethereum).network_name
            })
        );
        assert!(validation(Chain::Ethereum, "", &GemNameRecordState::None).error.is_none());
        assert!(validation(Chain::Ethereum, "vitalik.eth", &GemNameRecordState::None).error.is_none());

        let arbitrum = validation(Chain::Arbitrum, "0xinvalid", &GemNameRecordState::None);
        assert_eq!(
            arbitrum.error,
            Some(GemRecipientErrorDisplay::InvalidAddress {
                network: "Arbitrum".to_string()
            }),
            "the error names the network, not the chain's asset name"
        );
    }

    #[test]
    fn test_name_record_must_match_input_and_chain() {
        let ens = NameRecord::mock("vitalik.eth", ADDRESS);
        let valid = validation(Chain::Ethereum, "vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() });
        assert!(valid.is_valid);
        assert_eq!(valid.address, CHECKSUMMED);

        assert!(!validation(Chain::Ethereum, "other.eth", &GemNameRecordState::Complete { record: ens.clone() }).is_valid);
        assert!(!validation(Chain::Polygon, "vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() }).is_valid);
        assert!(
            !validation(
                Chain::Ethereum,
                "vitalik.eth",
                &GemNameRecordState::Complete {
                    record: NameRecord::mock("vitalik.eth", "0xbad")
                }
            )
            .is_valid
        );
    }

    #[test]
    fn test_recipient_builds_from_record_or_address() {
        let ens = NameRecord::mock("vitalik.eth", ADDRESS);
        let named = recipient(
            Chain::Ethereum,
            "vitalik.eth",
            &GemNameRecordState::Complete { record: ens.clone() },
            Some("memo".into()),
            vec!["ref".into()],
        )
        .unwrap();
        assert_eq!(named.address, CHECKSUMMED);
        assert_eq!(named.name.as_deref(), Some("vitalik.eth"));
        assert_eq!(named.memo.as_deref(), Some("memo"));
        assert_eq!(named.references, vec!["ref".to_string()]);

        let plain = recipient(Chain::Ethereum, ADDRESS, &GemNameRecordState::None, None, vec![]).unwrap();
        assert_eq!(plain.address, CHECKSUMMED);
        assert_eq!(plain.name, None);
        assert_eq!(
            recipient(Chain::Ethereum, "other.eth", &GemNameRecordState::Complete { record: ens.clone() }, None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(
            recipient(Chain::Ethereum, "0xinvalid", &GemNameRecordState::None, None, vec![]),
            Err(GemRecipientError::InvalidAddress)
        );
    }

    #[test]
    fn test_input_is_trimmed_and_prefix_is_case_sensitive() {
        let padded = format!("  {ADDRESS} \n");
        assert!(validation(Chain::Ethereum, &padded, &GemNameRecordState::None).is_valid);
        assert_eq!(recipient(Chain::Ethereum, &padded, &GemNameRecordState::None, None, vec![]).unwrap().address, CHECKSUMMED);

        let upper_prefix = ADDRESS.replacen("0x", "0X", 1);
        assert!(!validation(Chain::Ethereum, &upper_prefix, &GemNameRecordState::None).is_valid);
        assert_eq!(
            recipient(Chain::Ethereum, &upper_prefix, &GemNameRecordState::None, None, vec![]),
            Err(GemRecipientError::InvalidAddress)
        );
    }

    #[test]
    fn test_name_record_matching_is_exact() {
        let ens = NameRecord::mock("vitalik.eth", ADDRESS);
        assert!(!validation(Chain::Ethereum, "Vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() }).is_valid);
        assert_eq!(
            recipient(Chain::Ethereum, "Vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() }, None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(
            recipient(Chain::Ethereum, " vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() }, None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );
        assert_eq!(
            recipient(Chain::Polygon, "vitalik.eth", &GemNameRecordState::Complete { record: ens.clone() }, None, vec![]),
            Err(GemRecipientError::NameRecordMismatch)
        );

        let empty = NameRecord::mock("vitalik.eth", "");
        assert_eq!(
            recipient(Chain::Ethereum, "vitalik.eth", &GemNameRecordState::Complete { record: empty.clone() }, None, vec![]),
            Err(GemRecipientError::InvalidAddress)
        );
        let fallback = validation(Chain::Ethereum, "vitalik.eth", &GemNameRecordState::Complete { record: empty.clone() });
        assert!(!fallback.is_valid);
        assert_eq!(fallback.address, "vitalik.eth");
    }

    #[test]
    fn test_non_evm_addresses_keep_their_case() {
        let near = recipient(Chain::Near, "h3rman.near", &GemNameRecordState::None, None, vec![]).unwrap();
        assert_eq!(near.address, "h3rman.near");
        assert_eq!(near.name, None);

        let solana = "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2";
        assert_eq!(validation(Chain::Solana, solana, &GemNameRecordState::None).address, solana);
        assert!(validation(Chain::Solana, solana, &GemNameRecordState::None).is_valid);

        let tron = "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC";
        let tron_recipient = recipient(Chain::Tron, tron, &GemNameRecordState::None, Some("  memo ".into()), vec!["a".into(), "b".into()]).unwrap();
        assert_eq!(tron_recipient.address, tron);
        assert_eq!(tron_recipient.memo.as_deref(), Some("  memo "));
        assert_eq!(tron_recipient.references, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            recipient(Chain::Tron, &tron.to_lowercase(), &GemNameRecordState::None, None, vec![]),
            Err(GemRecipientError::InvalidAddress)
        );
    }

    #[test]
    fn test_scan_confirms_an_asset_payment_and_only_fills_an_nft_recipient() {
        let payments = GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200)));
        let destination = GemPaymentDestination::Confirm {
            transfer: GemPaymentConfirmTransfer {
                asset_id: primitives::AssetId::from_chain(Chain::Ethereum),
                address: ADDRESS.to_string(),
                value: 5u32.into(),
                memo: None,
                references: vec![],
            },
        };
        let asset = GemRecipientType::Asset {
            asset: Asset::from_chain(Chain::Ethereum),
        };
        let nft = GemRecipientType::Nft {
            nft_asset: primitives::NFTAsset::mock(),
        };

        assert!(
            matches!(scan_route(destination.clone(), &asset, |transfer| payments.transfer_data(transfer, Asset::from_chain(Chain::Ethereum))), Ok(GemRecipientScan::Confirm { transfer }) if transfer.recipient.address == ADDRESS)
        );
        assert!(
            matches!(scan_route(destination, &nft, |transfer| payments.transfer_data(transfer, Asset::from_chain(Chain::Ethereum))), Ok(GemRecipientScan::Recipient { payment }) if payment.recipient.address == ADDRESS && payment.amount.is_none())
        );
    }

    #[test]
    fn test_scan_fills_a_recipient_and_rejects_the_rest() {
        let payments = GemPaymentService::new(Arc::new(TestAlienProvider::with_status(200)));
        let asset = GemRecipientType::Asset {
            asset: Asset::from_chain(Chain::Ethereum),
        };
        let payment = GemPaymentRecipient {
            recipient: GemRecipient::address(ADDRESS.to_string()),
            amount: Some("1.5".to_string()),
        };
        let recipient = GemPaymentDestination::Recipient {
            asset_id: primitives::AssetId::from_chain(Chain::Ethereum),
            payment: payment.clone(),
        };

        assert!(
            matches!(scan_route(recipient, &asset, |transfer| payments.transfer_data(transfer, Asset::from_chain(Chain::Ethereum))), Ok(GemRecipientScan::Recipient { payment: found }) if found == payment)
        );
        assert!(matches!(
            scan_route(GemPaymentDestination::Unsupported, &asset, |transfer| payments
                .transfer_data(transfer, Asset::from_chain(Chain::Ethereum))),
            Err(GemRecipientError::InvalidAddress)
        ));
        assert!(matches!(
            scan_route(GemPaymentDestination::SelectAsset { payment, chains: vec![] }, &asset, |transfer| payments
                .transfer_data(transfer, Asset::from_chain(Chain::Ethereum))),
            Err(GemRecipientError::InvalidAddress)
        ));
    }

    #[test]
    fn test_next_goes_to_the_amount_for_an_asset_and_to_confirm_for_an_nft() {
        let payment = GemPaymentRecipient {
            recipient: GemRecipient::address(ADDRESS.to_string()),
            amount: Some("2".to_string()),
        };
        let nft_asset = primitives::NFTAsset::mock();

        assert!(matches!(
            next_step(GemRecipientType::Asset { asset: Asset::from_chain(Chain::Ethereum) }, payment.clone()),
            GemRecipientNext::Amount { payment: found } if found == payment
        ));
        match next_step(GemRecipientType::Nft { nft_asset: nft_asset.clone() }, payment) {
            GemRecipientNext::Confirm { transfer } => {
                assert!(
                    matches!(transfer.input_type, TransactionInputType::TransferNft { asset, nft_asset: found } if asset == Asset::from_chain(nft_asset.chain) && found.id == nft_asset.id)
                );
                assert_eq!(transfer.recipient.address, ADDRESS);
                assert_eq!(transfer.value, GemBigInt::from(0));
                assert!(!transfer.use_max_amount);
            }
            GemRecipientNext::Amount { .. } => panic!("an nft recipient goes straight to confirm"),
        }
    }

    fn names(section: &GemRecipientSection) -> Vec<String> {
        match section {
            GemRecipientSection::Pinned { wallets } | GemRecipientSection::Wallets { wallets } | GemRecipientSection::ViewWallets { wallets } => {
                wallets.iter().map(|wallet| wallet.name.clone()).collect()
            }
            GemRecipientSection::Contacts => vec![],
        }
    }

    #[test]
    fn test_a_private_key_wallet_is_offered_beside_the_other_wallets() {
        let wallets = vec![
            Wallet {
                name: "pinned".to_string(),
                is_pinned: true,
                ..Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Ethereum])
            },
            Wallet {
                name: "multicoin".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Ethereum])
            },
            Wallet {
                name: "private key".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::PrivateKey, &[Chain::Ethereum])
            },
            Wallet {
                name: "single".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::Single, &[Chain::Ethereum])
            },
            Wallet {
                name: "watching".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::View, &[Chain::Ethereum])
            },
            Wallet {
                name: "other chain".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Bitcoin])
            },
        ];

        let sections = recipient_sections(wallets, Chain::Ethereum, true);

        assert_eq!(names(&sections[0]), vec!["pinned"]);
        assert!(matches!(sections[1], GemRecipientSection::Contacts));
        assert_eq!(names(&sections[2]), vec!["multicoin", "private key", "single"]);
        assert_eq!(names(&sections[3]), vec!["watching"]);
    }

    #[test]
    fn test_an_empty_section_is_left_out_and_contacts_only_appear_when_there_are_some() {
        let sections = recipient_sections(
            vec![Wallet {
                name: "watching".to_string(),
                is_pinned: false,
                ..Wallet::mock_with_type(WalletType::View, &[Chain::Ethereum])
            }],
            Chain::Ethereum,
            false,
        );

        assert_eq!(sections.len(), 1);
        assert!(matches!(sections[0], GemRecipientSection::ViewWallets { .. }));
    }

    #[test]
    fn test_a_pinned_view_wallet_stays_in_the_pinned_section() {
        let sections = recipient_sections(
            vec![Wallet {
                name: "watching".to_string(),
                is_pinned: true,
                ..Wallet::mock_with_type(WalletType::View, &[Chain::Ethereum])
            }],
            Chain::Ethereum,
            false,
        );

        assert_eq!(names(&sections[0]), vec!["watching"]);
        assert_eq!(sections.len(), 1);
    }
}
