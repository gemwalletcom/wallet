use crate::{Contracts, Deployment, SwapperError, SwapperQuoteData};
use alloy_sol_types::SolCall;
use gem_evm::contracts::IERC20;
use gem_solana::{Pubkey, decode_transaction, instructions};
use gem_ton::models::TonConnectMessage;
use gem_tron::address::TronAddress;
use primitives::{
    AssetId, Chain, ChainType,
    contract_constants::{SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SOLANA_COMPUTE_BUDGET_PROGRAM_ID, SOLANA_MEMO_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID, SOLANA_TOKEN_2022_PROGRAM_ID, SOLANA_TOKEN_PROGRAM_ID},
    decode_hex,
    swap::SwapQuoteDataType,
};

const SOLANA_PROGRAMS: &[&str] = &[
    SOLANA_SYSTEM_PROGRAM_ID,
    SOLANA_TOKEN_PROGRAM_ID,
    SOLANA_TOKEN_2022_PROGRAM_ID,
    SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    SOLANA_COMPUTE_BUDGET_PROGRAM_ID,
    SOLANA_MEMO_PROGRAM_ID,
];

pub(crate) fn verify(from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    if contracts.targets.is_empty() {
        return Ok(());
    }
    match from_asset.chain.chain_type() {
        ChainType::Solana => verify_solana_programs(&data.data, &contracts.targets),
        ChainType::Ton => verify_ton_messages(&data.data, &contracts.targets),
        _ => verify_targets(from_asset, contracts, data),
    }
}

fn verify_targets(from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let chain = from_asset.chain;
    let is_contract = |address: &str| Deployment::contains(chain, &contracts.targets, address);
    let is_spender = |address: &str| Deployment::contains(chain, &contracts.spenders, address);
    if data.data_type == SwapQuoteDataType::Contract && !is_contract(&data.to) && !token_transfer_recipient(from_asset, data).is_some_and(|recipient| is_spender(&recipient)) {
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

pub(crate) fn verify_ton_messages(messages: &str, contracts: &[&str]) -> Result<(), SwapperError> {
    let messages: Vec<TonConnectMessage> = serde_json::from_str(messages).map_err(SwapperError::transaction_error)?;
    if !messages.is_empty() && messages.iter().all(|message| Deployment::contains(Chain::Ton, contracts, message.address())) {
        Ok(())
    } else {
        Err(SwapperError::InvalidRoute)
    }
}

pub(crate) fn verify_solana_programs(transaction: &str, contracts: &[&str]) -> Result<(), SwapperError> {
    let transaction = decode_transaction(transaction).map_err(SwapperError::transaction_error)?;
    let keys = transaction.account_keys();
    for instruction in transaction.instructions() {
        let program = keys.get(instruction.program_id_index as usize).map(Pubkey::to_base58).ok_or(SwapperError::InvalidRoute)?;
        let is_standard = SOLANA_PROGRAMS.contains(&program.as_str());
        if (!is_standard && !Deployment::contains(Chain::Solana, contracts, &program)) || (is_standard && instructions::grants_authority(&program, &instruction.data)) {
            return Err(SwapperError::InvalidRoute);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::SolanaTransaction;
    use gem_solana::{
        AccountMeta, Instruction, SolanaAddress,
        instructions::{program_ids::token_program, token},
    };
    use num_bigint::BigUint;
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;

    #[test]
    fn test_a_provider_that_ships_no_contracts_is_not_bound() {
        assert!(verify(&Chain::Osmosis.as_asset_id(), &Contracts::default(), &SwapperQuoteData::mock()).is_ok());
    }

    #[test]
    fn test_verify_token_transfer_recipient() {
        let vault = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC";
        let usdt = AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID);
        let transfer = |recipient: &str| {
            let to = TronAddress::from_hex_or_base58(recipient).unwrap();
            let call = IERC20::transferCall {
                to: alloy_primitives::Address::from(to),
                value: alloy_primitives::U256::from(1_000_000u64),
            };
            SwapperQuoteData::new_contract(TRON_USDT_TOKEN_ID.to_string(), BigUint::ZERO, hex::encode(call.abi_encode()), None, None)
        };

        assert!(verify(&usdt, &Contracts::new(vec![vault]), &transfer(vault)).is_ok(), "a token transfer into the shipped vault is the deposit");
        let router = "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN";
        let routed = Contracts {
            targets: vec![vault, router],
            spenders: vec![vault],
        };
        assert_eq!(verify(&usdt, &routed, &transfer(router)), Err(SwapperError::InvalidRoute), "a token sent to a contract that only runs calls is not a deposit");
        assert_eq!(verify(&usdt, &Contracts::new(vec![vault]), &transfer("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi")), Err(SwapperError::InvalidRoute));
        assert_eq!(
            verify(&Chain::Tron.as_asset_id(), &Contracts::new(vec![vault]), &transfer(vault)),
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

    #[test]
    fn test_standard_programs_cannot_hand_over_an_account() {
        let user = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let owner: Pubkey = SolanaAddress::parse(user).unwrap().into();
        let (token_account, mint, destination) = (Pubkey::new([2; 32]), Pubkey::new([3; 32]), Pubkey::new([4; 32]));
        let transaction = |instruction: Instruction| SolanaTransaction::new(vec![instruction], vec![]).encode(user, &[]).unwrap().data;
        let set_authority = Instruction {
            program_id: token_program(),
            accounts: vec![AccountMeta::new(token_account, false, true), AccountMeta::new(owner, true, false)],
            data: [vec![token::SET_AUTHORITY_DISCRIMINANT, 2, 1], Pubkey::new([9; 32]).as_bytes().to_vec()].concat(),
        };

        assert_eq!(
            verify_solana_programs(&transaction(set_authority), &[]),
            Err(SwapperError::InvalidRoute),
            "handing the user's token account to another key is refused"
        );
        let transfer_checked = token::transfer_checked_with_program_id(&token_account, &mint, &destination, &owner, 1_000, 6, &token_program());
        assert!(verify_solana_programs(&transaction(transfer_checked), &[]).is_ok(), "moving tokens is a swap");
    }

    #[test]
    fn test_verify_solana_programs() {
        let okx: serde_json::Value = serde_json::from_str(include_str!("okx/testdata/swap_sol_to_usdc.json")).unwrap();
        let transaction = gem_encoding::encode_base64(&bs58::decode(okx["data"][0]["tx"]["data"].as_str().unwrap()).into_vec().unwrap());

        assert!(verify_solana_programs(&transaction, &["proVF4pMXVaYqmy4NjniPh4pqKNfMmsihgd4wdkCX3u"]).is_ok(), "OKX's program plus the standard programs");
        assert_eq!(
            verify_solana_programs(&transaction, &["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"]),
            Err(SwapperError::InvalidRoute),
            "a transaction calling another program is refused"
        );
    }
}
