use std::str::FromStr;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use borsh::BorshDeserialize;
use gem_evm::{contracts::IERC20, u256::u256_to_biguint};
use gem_solana::{Instruction, Pubkey, anchor::global_discriminator, associated_token::is_associated_token_address};
use gem_tron::address::TronAddress;
use num_bigint::BigUint;
use primitives::hex::decode_hex;

use super::{
    broker::{EvmVaultSwapResponse, TronVaultSwapResponse},
    contracts::IVault,
    destination::{NO_CCM, VaultSwapDestination},
    refund::VaultSwapRefund,
};
use crate::SwapperError;

const SOLANA_NATIVE_SWAP: &str = "x_swap_native";
const SOLANA_TOKEN_SWAP: &str = "x_swap_token";
const SOLANA_DISCRIMINATOR_LENGTH: usize = 8;
const SOLANA_SOURCE_TOKEN_ACCOUNT: usize = 3;

#[derive(BorshDeserialize)]
struct SolanaSwapParameters {
    amount: u64,
    destination_chain: u32,
    destination_address: Vec<u8>,
    destination_asset: u32,
    ccm: u8,
    cf_parameters: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct VaultSwap {
    pub(super) vault: &'static str,
    pub(super) wallet: String,
    pub(super) token_id: Option<String>,
    pub(super) amount: BigUint,
    pub(super) destination: VaultSwapDestination,
    pub(super) refund: VaultSwapRefund,
}

impl VaultSwap {
    pub(super) fn validate_evm(&self, response: &EvmVaultSwapResponse) -> Result<BigUint, SwapperError> {
        let vault = Address::from_str(self.vault).map_err(|_| SwapperError::InvalidRoute)?;
        if Address::from_str(&response.to).ok() != Some(vault) {
            return Err(SwapperError::InvalidRoute);
        }
        let value = self.value(&response.value)?;
        let calldata = decode_hex(&response.calldata).map_err(|_| SwapperError::InvalidRoute)?;
        match self.token_id.as_deref() {
            None => {
                let call = IVault::xSwapNativeCall::abi_decode(&calldata).map_err(|_| SwapperError::InvalidRoute)?;
                self.destination.validate_call(call.dstChain, &call.dstAddress, call.dstToken)?;
                self.refund.validate(&call.cfParameters)?;
            }
            Some(token_id) => {
                let call = IVault::xSwapTokenCall::abi_decode(&calldata).map_err(|_| SwapperError::InvalidRoute)?;
                if Address::from_str(token_id).ok() != Some(call.srcToken) {
                    return Err(SwapperError::InvalidRoute);
                }
                self.validate_amount(&u256_to_biguint(&call.amount))?;
                self.destination.validate_call(call.dstChain, &call.dstAddress, call.dstToken)?;
                self.refund.validate(&call.cfParameters)?;
            }
        }
        Ok(value)
    }

    pub(super) fn validate_tron(&self, response: &TronVaultSwapResponse) -> Result<(TronAddress, BigUint), SwapperError> {
        let is_vault = |address: &TronAddress| address.to_string() == self.vault;
        let value = self.value(&response.value)?;
        let to = match self.token_id.as_deref() {
            None => {
                let vault = TronAddress::from_hex_or_base58(&response.to).filter(is_vault).ok_or(SwapperError::InvalidRoute)?;
                if !decode_hex(&response.calldata).map_err(|_| SwapperError::InvalidRoute)?.is_empty() {
                    return Err(SwapperError::InvalidRoute);
                }
                vault
            }
            Some(token_id) => {
                let source_token = response
                    .source_token_address
                    .as_deref()
                    .and_then(TronAddress::from_hex_or_base58)
                    .filter(|source_token| TronAddress::from_hex_or_base58(token_id) == Some(*source_token))
                    .ok_or(SwapperError::InvalidRoute)?;
                let transfer = decode_hex(&response.calldata).ok().and_then(|calldata| IERC20::transferCall::abi_decode(&calldata).ok()).ok_or(SwapperError::InvalidRoute)?;
                if !is_vault(&TronAddress::from(transfer.to)) {
                    return Err(SwapperError::InvalidRoute);
                }
                self.validate_amount(&u256_to_biguint(&transfer.value))?;
                source_token
            }
        };
        let note = decode_hex(&response.note).map_err(|_| SwapperError::InvalidRoute)?;
        self.refund.validate(self.destination.validate_tron(&note)?)?;
        Ok((to, value))
    }

    pub(super) fn validate_solana(&self, instruction: &Instruction) -> Result<(), SwapperError> {
        if instruction.program_id.to_base58() != self.vault {
            return Err(SwapperError::InvalidRoute);
        }
        let name = match self.token_id {
            None => SOLANA_NATIVE_SWAP,
            Some(_) => SOLANA_TOKEN_SWAP,
        };
        let (discriminator, mut arguments) = instruction.data.split_at_checked(SOLANA_DISCRIMINATOR_LENGTH).ok_or(SwapperError::InvalidRoute)?;
        if discriminator != global_discriminator(name) {
            return Err(SwapperError::InvalidRoute);
        }
        if let Some(token_id) = &self.token_id {
            let wallet = Pubkey::from_base58(&self.wallet).map_err(|_| SwapperError::InvalidRoute)?;
            let mint = Pubkey::from_base58(token_id).map_err(|_| SwapperError::InvalidRoute)?;
            if !instruction.accounts.get(SOLANA_SOURCE_TOKEN_ACCOUNT).is_some_and(|account| is_associated_token_address(&account.pubkey, &wallet, &mint)) {
                return Err(SwapperError::InvalidRoute);
            }
        }
        let parameters = SolanaSwapParameters::deserialize(&mut arguments).map_err(|_| SwapperError::InvalidRoute)?;
        if parameters.ccm != NO_CCM {
            return Err(SwapperError::InvalidRoute);
        }
        self.validate_amount(&BigUint::from(parameters.amount))?;
        self.destination.validate_call(parameters.destination_chain, &parameters.destination_address, parameters.destination_asset)?;
        self.refund.validate(&parameters.cf_parameters)
    }

    fn value(&self, broker_value: &BigUint) -> Result<BigUint, SwapperError> {
        match self.token_id {
            None => self.validate_amount(broker_value).map(|()| self.amount.clone()),
            Some(_) if *broker_value == BigUint::ZERO => Ok(BigUint::ZERO),
            Some(_) => Err(SwapperError::InvalidRoute),
        }
    }

    fn validate_amount(&self, amount: &BigUint) -> Result<(), SwapperError> {
        if *amount != self.amount {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainflip::{
        broker::SolanaVaultSwapResponse,
        provider::TRON_VAULT,
        testkit::{ETHEREUM_NATIVE_MIN_PRICE, SOLANA_MIN_PRICE, arbitrum_usdc_destination, ethereum_usdc_refund, ethereum_vault_swap, mock_refund, solana_vault_swap, tron_vault_swap},
        tx_builder::solana_instruction,
    };
    use primitives::{
        AssetId, Chain,
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID, SOLANA_USDC_TOKEN_ID, SOLANA_USDT_TOKEN_ID, TRON_USDT_TOKEN_ID},
        hex::encode_with_0x,
        testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_RECIPIENT, TEST_EVM_SENDER, TEST_PRIVATE_KEY_SOLANA_ADDRESS},
    };

    #[test]
    fn test_validate_evm() {
        let native: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_native.json")).unwrap();
        let usdc: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_usdc.json")).unwrap();
        let swap = ethereum_vault_swap();
        let token_swap = VaultSwap {
            token_id: Some(ETHEREUM_USDC_TOKEN_ID.to_string()),
            amount: BigUint::from(500_000_000u64),
            refund: ethereum_usdc_refund(),
            ..ethereum_vault_swap()
        };

        assert_eq!(swap.validate_evm(&native), Ok(swap.amount.clone()));
        assert_eq!(token_swap.validate_evm(&usdc), Ok(BigUint::ZERO));

        for (refused, response, reason) in [
            (
                swap.clone(),
                EvmVaultSwapResponse {
                    to: TEST_EVM_RECIPIENT.to_string(),
                    ..native.clone()
                },
                "a call to another contract",
            ),
            (VaultSwap { amount: BigUint::from(1u32), ..swap.clone() }, native.clone(), "a native amount other than quoted"),
            (
                VaultSwap {
                    amount: BigUint::from(1u32),
                    ..token_swap.clone()
                },
                usdc.clone(),
                "a token amount other than quoted",
            ),
            (
                VaultSwap {
                    token_id: Some(ETHEREUM_USDT_TOKEN_ID.to_string()),
                    ..token_swap.clone()
                },
                usdc.clone(),
                "a token other than quoted",
            ),
            (token_swap.clone(), EvmVaultSwapResponse { value: BigUint::ZERO, ..native.clone() }, "a native call for a token swap"),
            (token_swap, EvmVaultSwapResponse { value: BigUint::from(1u32), ..usdc }, "a token swap that also sends coins"),
            (
                VaultSwap {
                    destination: VaultSwapDestination::new(&AssetId::from_chain(Chain::Bitcoin), "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh").unwrap(),
                    ..swap.clone()
                },
                native.clone(),
                "a payout to another address",
            ),
            (
                VaultSwap {
                    destination: arbitrum_usdc_destination(),
                    ..swap.clone()
                },
                native.clone(),
                "a payout of another asset",
            ),
            (
                VaultSwap {
                    refund: mock_refund(Chain::Ethereum, "0x1111111111111111111111111111111111111111", ETHEREUM_NATIVE_MIN_PRICE),
                    ..swap.clone()
                },
                native.clone(),
                "a refund to another address",
            ),
            (
                VaultSwap {
                    refund: mock_refund(Chain::Ethereum, TEST_EVM_SENDER, "0x3292bb84399604d6d7d2345"),
                    ..swap
                },
                native,
                "another minimum price",
            ),
        ] {
            assert_eq!(refused.validate_evm(&response), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_validate_tron() {
        let native = TronVaultSwapResponse::mock();
        let token = TronVaultSwapResponse {
            calldata: "0xa9059cbb00000000000000000000000032e07e5dfcb75c2977adddc593eb8d6b6e71f96e0000000000000000000000000000000000000000000000000000000000989680".to_string(),
            value: BigUint::ZERO,
            source_token_address: Some("0xa614f803b6fd780986a42c78ec9c7f77e6ded13c".to_string()),
            ..TronVaultSwapResponse::mock()
        };
        let swap = tron_vault_swap();
        let token_swap = VaultSwap {
            token_id: Some(TRON_USDT_TOKEN_ID.to_string()),
            amount: BigUint::from(10_000_000u32),
            ..tron_vault_swap()
        };
        let native_call = IVault::xSwapNativeCall {
            dstChain: 4,
            dstAddress: vec![0x66; 20].into(),
            dstToken: 7,
            cfParameters: vec![].into(),
        };

        assert_eq!(swap.validate_tron(&native), Ok((TronAddress::parse(TRON_VAULT).unwrap(), swap.amount.clone())));
        assert_eq!(
            swap.validate_tron(&TronVaultSwapResponse { calldata: String::new(), ..native.clone() }),
            Ok((TronAddress::parse(TRON_VAULT).unwrap(), swap.amount.clone()))
        );
        assert_eq!(token_swap.validate_tron(&token), Ok((TronAddress::parse(TRON_USDT_TOKEN_ID).unwrap(), BigUint::ZERO)));

        for (refused, response, reason) in [
            (
                VaultSwap {
                    amount: BigUint::from(40_000_000u32),
                    ..swap.clone()
                },
                native.clone(),
                "a native amount other than quoted",
            ),
            (
                VaultSwap {
                    amount: BigUint::from(9_999_999u32),
                    ..token_swap.clone()
                },
                token.clone(),
                "a token amount other than quoted",
            ),
            (
                swap.clone(),
                TronVaultSwapResponse {
                    calldata: encode_with_0x(&native_call.abi_encode()),
                    ..native.clone()
                },
                "a native deposit that also runs a call",
            ),
            (token_swap.clone(), TronVaultSwapResponse { value: BigUint::from(1u32), ..token.clone() }, "a token swap that also sends coins"),
            (
                swap.clone(),
                TronVaultSwapResponse {
                    to: "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi".to_string(),
                    ..native.clone()
                },
                "a coin sent elsewhere than the vault",
            ),
            (
                token_swap.clone(),
                TronVaultSwapResponse {
                    calldata: "0xa9059cbb0000000000000000000000002523ae929fecd9d665f472f59b99a8ce6b1795100000000000000000000000000000000000000000000000000000000000989680".to_string(),
                    ..token.clone()
                },
                "a token sent elsewhere than the vault",
            ),
            (
                token_swap,
                TronVaultSwapResponse {
                    source_token_address: Some("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf".to_string()),
                    ..token
                },
                "a token other than quoted",
            ),
            (
                VaultSwap {
                    destination: ethereum_vault_swap().destination,
                    refund: solana_vault_swap().refund,
                    ..swap.clone()
                },
                native.clone(),
                "a payout other than quoted",
            ),
            (
                VaultSwap {
                    refund: mock_refund(Chain::Tron, TRON_VAULT, "0x0"),
                    ..swap
                },
                native,
                "a refund to another address",
            ),
        ] {
            assert_eq!(refused.validate_tron(&response), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_validate_solana() {
        let native = solana_instruction(&serde_json::from_str::<SolanaVaultSwapResponse>(include_str!("broker/test/vault_swap_sol_native.json")).unwrap()).unwrap();
        let usdc = solana_instruction(&serde_json::from_str::<SolanaVaultSwapResponse>(include_str!("broker/test/vault_swap_sol_usdc.json")).unwrap()).unwrap();
        let swap = solana_vault_swap();
        let token_swap = VaultSwap {
            token_id: Some(SOLANA_USDC_TOKEN_ID.to_string()),
            amount: BigUint::from(100_000_000u64),
            ..solana_vault_swap()
        };

        assert_eq!(swap.validate_solana(&native), Ok(()));
        assert_eq!(token_swap.validate_solana(&usdc), Ok(()));

        let mut with_message = native.clone();
        with_message.data[SOLANA_DISCRIMINATOR_LENGTH + size_of::<u64>() + 3 * size_of::<u32>() + TEST_BITCOIN_SENDER.len()] = 1;
        for (refused, instruction, reason) in [
            (swap.clone(), with_message, "a payout that also runs a contract message"),
            (VaultSwap { amount: BigUint::from(1u32), ..swap.clone() }, native.clone(), "an amount other than quoted"),
            (
                swap.clone(),
                Instruction {
                    program_id: Pubkey::new([0; 32]),
                    ..native.clone()
                },
                "another program",
            ),
            (
                VaultSwap {
                    destination: arbitrum_usdc_destination(),
                    ..swap.clone()
                },
                native.clone(),
                "a payout other than quoted",
            ),
            (
                VaultSwap {
                    refund: mock_refund(Chain::Solana, "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy", SOLANA_MIN_PRICE),
                    ..swap.clone()
                },
                native.clone(),
                "a refund to another address",
            ),
            (
                VaultSwap {
                    amount: token_swap.amount.clone(),
                    ..swap.clone()
                },
                usdc.clone(),
                "a token swap for a coin quote",
            ),
            (VaultSwap { amount: swap.amount, ..token_swap.clone() }, native, "a coin swap for a token quote"),
            (
                VaultSwap {
                    token_id: Some(SOLANA_USDT_TOKEN_ID.to_string()),
                    ..token_swap.clone()
                },
                usdc.clone(),
                "a token other than quoted",
            ),
            (
                VaultSwap {
                    wallet: TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string(),
                    ..token_swap
                },
                usdc,
                "a token account that is not the wallet's",
            ),
        ] {
            assert_eq!(refused.validate_solana(&instruction), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }
}
