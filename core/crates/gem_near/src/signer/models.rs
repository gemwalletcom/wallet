use primitives::{FeeOption, SignerError, SignerInput};
use serde::Serialize;

use crate::constants::{FUNGIBLE_TOKEN_FUNCTION_CALL_GAS, FUNGIBLE_TOKEN_TRANSFER_DEPOSIT};

#[derive(Serialize)]
struct FungibleTokenTransferArgs<'a> {
    receiver_id: &'a str,
    amount: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    memo: Option<&'a str>,
}

#[derive(Serialize)]
struct StorageDepositArgs<'a> {
    account_id: &'a str,
    registration_only: bool,
}

pub(super) struct NearTransaction {
    pub(super) signer_id: String,
    pub(super) receiver_id: String,
    pub(super) nonce: u64,
    pub(super) block_hash: [u8; 32],
    pub(super) actions: Vec<NearAction>,
}

pub(super) enum NearAction {
    Transfer {
        deposit: u128,
    },
    FunctionCall {
        method_name: &'static str,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    },
}

impl NearTransaction {
    pub(super) fn from_transfer_input(input: &SignerInput) -> Result<Self, SignerError> {
        let deposit = input.value.parse::<u128>().map_err(|_| SignerError::invalid_input("invalid NEAR amount"))?;
        Self::new(input, input.destination_address.clone(), vec![NearAction::Transfer { deposit }])
    }

    pub(super) fn from_token_transfer_input(input: &SignerInput) -> Result<Self, SignerError> {
        let token_id = input.input_type.get_asset().id.get_token_id()?.clone();

        let mut actions = Vec::new();
        if let Some(deposit) = input.fee.options.get(&FeeOption::TokenAccountCreation) {
            let deposit: u128 = deposit.try_into().map_err(|_| SignerError::invalid_input("invalid NEAR token account creation deposit"))?;
            let args = serde_json::to_vec(&StorageDepositArgs {
                account_id: &input.destination_address,
                registration_only: true,
            })
            .map_err(SignerError::from_display)?;
            actions.push(NearAction::FunctionCall {
                method_name: "storage_deposit",
                args,
                gas: FUNGIBLE_TOKEN_FUNCTION_CALL_GAS,
                deposit,
            });
        }

        let args = serde_json::to_vec(&FungibleTokenTransferArgs {
            receiver_id: &input.destination_address,
            amount: &input.value,
            memo: input.get_memo(),
        })
        .map_err(SignerError::from_display)?;
        actions.push(NearAction::FunctionCall {
            method_name: "ft_transfer",
            args,
            gas: FUNGIBLE_TOKEN_FUNCTION_CALL_GAS,
            deposit: FUNGIBLE_TOKEN_TRANSFER_DEPOSIT,
        });

        Self::new(input, token_id, actions)
    }

    fn new(input: &SignerInput, receiver_id: String, actions: Vec<NearAction>) -> Result<Self, SignerError> {
        let block_hash = bs58::decode(input.metadata.get_block_hash()?)
            .into_vec()
            .map_err(|error| SignerError::invalid_input(format!("invalid NEAR block hash: {error}")))?
            .try_into()
            .map_err(|_| SignerError::invalid_input("NEAR block hash must be 32 bytes"))?;

        Ok(Self {
            signer_id: input.sender_address.clone(),
            receiver_id,
            nonce: input.metadata.get_sequence()?,
            block_hash,
            actions,
        })
    }
}
