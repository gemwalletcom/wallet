use num_bigint::BigInt;
use primitives::TransferDataOutputAction;

use super::{GemConfirmError, GemSendInput};
use crate::models::transaction::{GemSignedTransaction, GemSignerInput, GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadInput};
use crate::services::transfer::rules as transfer_rules;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::gateway::GemGasPriceType;
    use crate::models::transaction::GemTransactionLoadMetadata;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use primitives::{Account, ApplicationMetadata, Asset, AssetId, Chain, TransactionType, TransferDataExtra, Wallet, WalletId, WalletSource, WalletType, swap::ApprovalData};

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
}
