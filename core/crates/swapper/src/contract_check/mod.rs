mod solana;

use crate::{Binding, Contracts, Deployment, Quote, SwapperError, SwapperQuoteData};
use alloy_sol_types::SolCall;
use gem_evm::contracts::IERC20;
use gem_ton::models::TonConnectMessage;
use gem_tron::address::TronAddress;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, ChainType, decode_hex, swap::SwapQuoteDataType};

pub(crate) use solana::{SolanaStep, verify_solana, verify_steps};

enum QuoteData {
    EvmCall,
    SolanaTransaction,
    TonMessages,
    Unread,
}

impl QuoteData {
    fn of(chain_type: ChainType) -> Self {
        match chain_type {
            ChainType::Ethereum | ChainType::Tron => Self::EvmCall,
            ChainType::Solana => Self::SolanaTransaction,
            ChainType::Ton => Self::TonMessages,
            ChainType::Bitcoin | ChainType::Cosmos | ChainType::Aptos | ChainType::Sui | ChainType::Xrp | ChainType::Near | ChainType::Stellar | ChainType::Algorand | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => {
                Self::Unread
            }
        }
    }
}

pub(crate) fn verify(quote: &Quote, binding: &Binding, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let from_asset = quote.request.from_asset.asset_id();
    verify_amounts(&from_asset, &quote.from_value, data)?;
    match binding {
        Binding::Contracts(contracts) => verify_contracts(quote, &from_asset, contracts, data),
        Binding::DepositAddress | Binding::BuiltOnDevice => Ok(()),
    }
}

fn verify_amounts(from_asset: &AssetId, from_value: &BigUint, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let is_swapped_token = |token: &str| from_asset.token_id.as_deref().is_some_and(|token_id| Deployment::contains(from_asset.chain, &[token_id], token));
    let approves_swapped_token = data
        .approval
        .iter()
        .map(|approval| approval.token.as_str())
        .chain(data.permit2.iter().map(|permit2| permit2.token.as_str()))
        .all(is_swapped_token);
    let sends_quoted_value = match (&data.data_type, QuoteData::of(from_asset.chain.chain_type())) {
        (SwapQuoteDataType::Transfer, _) => data.value <= *from_value,
        (SwapQuoteDataType::Contract, QuoteData::EvmCall) => match from_asset.is_native() {
            true => data.value <= *from_value,
            false => data.value == BigUint::ZERO,
        },
        (SwapQuoteDataType::Contract, QuoteData::SolanaTransaction | QuoteData::TonMessages | QuoteData::Unread) => true,
    };
    if !approves_swapped_token || !sends_quoted_value {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

fn verify_contracts(quote: &Quote, from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    if contracts.targets.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    match (QuoteData::of(from_asset.chain.chain_type()), &data.data_type) {
        (QuoteData::SolanaTransaction, _) => verify_solana(&data.data, &quote.request.wallet_address, &contracts.targets),
        (QuoteData::TonMessages, _) => verify_ton_messages(&data.data, &contracts.targets),
        (QuoteData::EvmCall | QuoteData::Unread, SwapQuoteDataType::Transfer) => match Deployment::contains(from_asset.chain, &contracts.targets, &data.to) {
            true => Ok(()),
            false => Err(SwapperError::InvalidRoute),
        },
        (QuoteData::EvmCall, SwapQuoteDataType::Contract) => verify_targets(from_asset, contracts, data),
        (QuoteData::Unread, SwapQuoteDataType::Contract) => Err(SwapperError::InvalidRoute),
    }
}

fn verify_targets(from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let chain = from_asset.chain;
    let is_contract = |address: &str| Deployment::contains(chain, &contracts.targets, address);
    let is_spender = |address: &str| Deployment::contains(chain, &contracts.spenders, address);
    if !is_contract(&data.to) && !token_transfer_recipient(from_asset, data).is_some_and(|recipient| is_spender(&recipient)) {
        return Err(SwapperError::InvalidRoute);
    }
    if let Some(approval) = &data.approval
        && !is_spender(&approval.spender)
    {
        return Err(SwapperError::InvalidRoute);
    }
    if let Some(permit2) = &data.permit2
        && !is_spender(&permit2.spender)
    {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

fn token_transfer_recipient(from_asset: &AssetId, data: &SwapperQuoteData) -> Option<String> {
    from_asset.token_id.as_deref().filter(|token| Deployment::contains(from_asset.chain, &[token], &data.to))?;
    let transfer = IERC20::transferCall::abi_decode(&decode_hex(&data.data).ok()?).ok()?;
    Some(match from_asset.chain.chain_type() {
        ChainType::Tron => TronAddress::from(transfer.to).to_string(),
        _ => format!("{:#x}", transfer.to),
    })
}

fn verify_ton_messages(messages: &str, contracts: &[&str]) -> Result<(), SwapperError> {
    let messages: Vec<TonConnectMessage> = serde_json::from_str(messages).map_err(SwapperError::transaction_error)?;
    if !messages.is_empty() && messages.iter().all(|message| Deployment::contains(Chain::Ton, contracts, message.address())) {
        Ok(())
    } else {
        Err(SwapperError::InvalidRoute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QuoteRequest, solana_dex::SolanaTransaction};
    use alloy_primitives::{Address, U256};
    use gem_solana::{JUPITER_PROGRAM_ID, compute_budget::set_compute_unit_limit};
    use primitives::{
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID, TRON_USDT_TOKEN_ID},
        hex,
        swap::{ApprovalData, Permit2ApprovalData},
        testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_RECIPIENT, TEST_SOLANA_SENDER},
    };

    #[test]
    fn test_verify() {
        let quote = Quote::mock(Chain::Ethereum, None);
        let data = SwapperQuoteData::mock();

        assert_eq!(verify(&quote, &Binding::BuiltOnDevice, &data), Ok(()));
        assert_eq!(verify(&quote, &Binding::DepositAddress, &data), Ok(()));
        assert_eq!(verify(&quote, &Binding::Contracts(Contracts::new(vec![TEST_EVM_RECIPIENT])), &data), Ok(()));
        assert_eq!(
            verify(&quote, &Binding::Contracts(Contracts::default()), &data),
            Err(SwapperError::InvalidRoute),
            "a provider that ships no contracts binds nothing"
        );
        let solana = Quote {
            request: QuoteRequest {
                wallet_address: TEST_SOLANA_SENDER.to_string(),
                ..QuoteRequest::mock(Chain::Solana, None)
            },
            ..Quote::mock(Chain::Solana, None)
        };
        let standard_steps = SolanaTransaction::new(vec![set_compute_unit_limit(200_000)], vec![]).encode(TEST_SOLANA_SENDER, &[]).unwrap();
        assert_eq!(verify(&solana, &Binding::Contracts(Contracts::new(vec![JUPITER_PROGRAM_ID])), &standard_steps), Ok(()));
        assert_eq!(
            verify(&solana, &Binding::Contracts(Contracts::default()), &standard_steps),
            Err(SwapperError::InvalidRoute),
            "a provider that ships no programs binds nothing, even on standard steps"
        );
        assert_eq!(
            verify(&quote, &Binding::BuiltOnDevice, &SwapperQuoteData { value: BigUint::from(1_000_001u64), ..data }),
            Err(SwapperError::InvalidRoute),
            "the amounts bind every provider, bound or not"
        );

        let xrp = Quote::mock(Chain::Xrp, None);
        let deposit = SwapperQuoteData::new_transfer("rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn".to_string(), xrp.from_value.clone(), None);
        assert_eq!(verify(&xrp, &Binding::Contracts(Contracts::new(vec!["rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn"])), &deposit), Ok(()));
        assert_eq!(
            verify(&xrp, &Binding::Contracts(Contracts::new(vec!["rDsbeomae4FXwgQTJp9Rs64Qg9vDiTCdBv"])), &deposit),
            Err(SwapperError::InvalidRoute),
            "a transfer pays only a shipped address"
        );
        assert_eq!(
            verify(&Quote::mock(Chain::Sui, None), &Binding::Contracts(Contracts::new(vec![TEST_EVM_RECIPIENT])), &SwapperQuoteData::mock()),
            Err(SwapperError::InvalidRoute),
            "a contract call on a network the check cannot read is refused"
        );
    }

    #[test]
    fn test_verify_amounts() {
        let coin = AssetId::from_chain(Chain::Ethereum);
        let usdc = AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID);
        let amount = BigUint::from(1_000u64);
        let approval = ApprovalData::make(ETHEREUM_USDC_TOKEN_ID, TEST_EVM_RECIPIENT, amount.clone(), false);
        let permit2 = Permit2ApprovalData {
            token: ETHEREUM_USDC_TOKEN_ID.to_lowercase(),
            spender: TEST_EVM_RECIPIENT.to_string(),
            value: amount.clone(),
            permit2_contract: TEST_EVM_RECIPIENT.to_string(),
            permit2_nonce: 0,
        };
        let token_call = SwapperQuoteData {
            approval: Some(approval.clone()),
            permit2: Some(permit2.clone()),
            ..SwapperQuoteData::mock()
        };

        assert_eq!(
            verify_amounts(
                &coin,
                &amount,
                &SwapperQuoteData {
                    value: amount.clone(),
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(
            verify_amounts(
                &coin,
                &amount,
                &SwapperQuoteData {
                    value: BigUint::from(999u64),
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(verify_amounts(&usdc, &amount, &token_call), Ok(()));
        assert_eq!(
            verify_amounts(
                &AssetId::from_chain(Chain::Solana),
                &amount,
                &SwapperQuoteData {
                    value: amount.clone() + 1u32,
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(
            verify_amounts(&AssetId::from_chain(Chain::Bitcoin), &amount, &SwapperQuoteData::new_transfer(TEST_BITCOIN_SENDER.to_string(), amount.clone(), None)),
            Ok(())
        );

        for (asset, data, reason) in [
            (
                &coin,
                SwapperQuoteData {
                    value: amount.clone() + 1u32,
                    ..SwapperQuoteData::mock()
                },
                "a coin swap that sends more than quoted",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    value: BigUint::from(1u32),
                    ..token_call.clone()
                },
                "a token swap that also sends coins",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    approval: Some(ApprovalData::make(ETHEREUM_USDT_TOKEN_ID, TEST_EVM_RECIPIENT, amount.clone(), false)),
                    ..token_call.clone()
                },
                "an approval for another token",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    permit2: Some(Permit2ApprovalData {
                        token: ETHEREUM_USDT_TOKEN_ID.to_string(),
                        ..permit2
                    }),
                    ..token_call
                },
                "a Permit2 signature for another token",
            ),
            (
                &coin,
                SwapperQuoteData {
                    approval: Some(approval),
                    ..SwapperQuoteData::mock()
                },
                "an approval on a coin swap",
            ),
            (
                &AssetId::from_chain(Chain::Bitcoin),
                SwapperQuoteData::new_transfer(TEST_BITCOIN_SENDER.to_string(), amount.clone() + 1u32, None),
                "a deposit that sends more than quoted",
            ),
        ] {
            assert_eq!(verify_amounts(asset, &amount, &data), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_verify_token_transfer_recipient() {
        let vault = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC";
        let usdt = Quote::mock(Chain::Tron, Some(TRON_USDT_TOKEN_ID));
        let transfer = |recipient: &str| {
            let to = TronAddress::from_hex_or_base58(recipient).unwrap();
            let call = IERC20::transferCall {
                to: Address::from(to),
                value: U256::from(1_000_000u64),
            };
            SwapperQuoteData::new_contract(TRON_USDT_TOKEN_ID.to_string(), BigUint::ZERO, hex::encode(call.abi_encode()), None, None)
        };

        assert_eq!(
            verify(&usdt, &Binding::Contracts(Contracts::new(vec![vault])), &transfer(vault)),
            Ok(()),
            "a token transfer into the shipped vault is the deposit"
        );
        let router = "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN";
        let routed = Contracts {
            targets: vec![vault, router],
            spenders: vec![vault],
        };
        assert_eq!(
            verify(&usdt, &Binding::Contracts(routed), &transfer(router)),
            Err(SwapperError::InvalidRoute),
            "a token sent to a contract that only runs calls is not a deposit"
        );
        assert_eq!(
            verify(&usdt, &Binding::Contracts(Contracts::new(vec![vault])), &transfer("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi")),
            Err(SwapperError::InvalidRoute)
        );
        assert_eq!(
            verify(&Quote::mock(Chain::Tron, None), &Binding::Contracts(Contracts::new(vec![vault])), &transfer(vault)),
            Err(SwapperError::InvalidRoute),
            "a coin swap may not call a token contract"
        );
    }

    #[test]
    fn test_verify_ton_messages() {
        let depository = "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs";
        let messages = |addresses: &[&str]| serde_json::to_string(&addresses.iter().map(|address| TonConnectMessage::new(address.to_string(), &BigUint::from(1u64), None)).collect::<Vec<_>>()).unwrap();

        assert!(verify_ton_messages(&messages(&[depository]), &[depository]).is_ok());
        assert!(verify_ton_messages(&messages(&[depository, depository]), &[depository]).is_ok());
        assert_eq!(
            verify_ton_messages(&messages(&[depository, "EQBvW8Z5huBkMJYdnfAEM5JqTNkuWX3diqYENkWsIL0XggGG"]), &[depository]),
            Err(SwapperError::InvalidRoute),
            "every message must go to a shipped contract"
        );
        assert_eq!(verify_ton_messages("[]", &[depository]), Err(SwapperError::InvalidRoute));
    }
}
