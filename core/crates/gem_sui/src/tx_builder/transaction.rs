use super::{Argument, TransactionBuilder, TransactionBuilderInput};
use crate::{
    SUI_COIN_TYPE, SuiError,
    address::SuiAddress,
    is_sui_coin,
    models::{Coin, OwnedCoins, TxOutput},
    sui_framework_package_address,
    types::{Address, GasPayment, Identifier, MoveCall, ObjectInput, ObjectReference, Transaction, TransactionExpiration, TransactionKind},
};
use gem_encoding::decode_base64;
use serde::de::DeserializeOwned;
use std::error::Error;

const MODULE_COIN: &str = "coin";
const FUNCTION_ZERO: &str = "zero";
const FUNCTION_REDEEM_FUNDS: &str = "redeem_funds";

fn build_withdrawn_coin(txb: &mut TransactionBuilder, coin_type: &str, amount: u64) -> Result<Argument, SuiError> {
    let withdrawal = txb.funds_withdrawal(coin_type.parse()?, amount);
    move_call(txb, sui_framework_package_address(), MODULE_COIN, FUNCTION_REDEEM_FUNDS, &[coin_type], vec![withdrawal])
}

/// Build a `Coin<T>` of exactly `amount`: pure withdrawal if Address Balance covers it, else coin objects topped up by the shortfall.
pub(crate) fn build_amount_coin(txb: &mut TransactionBuilder, coin_type: &str, amount: u64, address_balance: u64, coins: &[Coin]) -> Result<Argument, SuiError> {
    if address_balance >= amount {
        return build_withdrawn_coin(txb, coin_type, amount);
    }

    if coins.is_empty() {
        return Err(SuiError::invalid_input("no coin sources for Sui amount"));
    }

    let coin_total: u64 = coins.iter().map(|c| c.balance).fold(0, u64::saturating_add);
    let mut coin_args: Vec<Argument> = coins.iter().map(|c| txb.object(c.to_input())).collect();
    let primary = coin_args.remove(0);
    if !coin_args.is_empty() {
        txb.merge_coins(primary, coin_args);
    }
    if let Some(shortfall) = amount.checked_sub(coin_total).filter(|s| *s > 0) {
        let withdrawn = build_withdrawn_coin(txb, coin_type, shortfall)?;
        txb.merge_coins(primary, vec![withdrawn]);
    }

    let amount_arg = txb.pure(&amount);
    txb.split_coins(primary, vec![amount_arg]).pop().ok_or_else(|| SuiError::invalid_input("Sui split coin failed"))
}

pub(crate) fn build_sui_gas_coin(txb: &mut TransactionBuilder, address_balance: u64) -> Result<Argument, SuiError> {
    let gas = txb.gas();
    if address_balance > 0 {
        let withdrawn = build_withdrawn_coin(txb, SUI_COIN_TYPE, address_balance)?;
        txb.merge_coins(gas, vec![withdrawn]);
    }
    Ok(gas)
}

pub(crate) fn build_sui_amount_coin(txb: &mut TransactionBuilder, amount: u64, address_balance: u64) -> Result<Argument, SuiError> {
    if address_balance >= amount {
        return build_withdrawn_coin(txb, SUI_COIN_TYPE, amount);
    }

    let gas = build_sui_gas_coin(txb, address_balance)?;
    let amount_arg = txb.pure(&amount);
    txb.split_coins(gas, vec![amount_arg]).pop().ok_or_else(|| SuiError::invalid_input("Sui split coin failed"))
}

pub fn move_call(txb: &mut TransactionBuilder, package: Address, module: &str, function: &str, type_args: &[&str], arguments: Vec<Argument>) -> Result<Argument, SuiError> {
    Ok(txb.move_call(MoveCall {
        package,
        module: Identifier::new(module)?,
        function: Identifier::new(function)?,
        type_arguments: type_args.iter().map(|value| value.parse()).collect::<Result<_, _>>()?,
        arguments,
    }))
}

pub fn zero_coin(txb: &mut TransactionBuilder, coin_type: &str) -> Result<Argument, SuiError> {
    move_call(txb, sui_framework_package_address(), MODULE_COIN, FUNCTION_ZERO, &[coin_type], vec![])
}

pub fn build_input_coin(txb: &mut TransactionBuilder, coin_type: &str, amount: u64, source: &OwnedCoins<Coin>) -> Result<Argument, SuiError> {
    if amount == 0 {
        return zero_coin(txb, coin_type);
    }

    if is_sui_coin(coin_type) {
        return build_sui_amount_coin(txb, amount, source.address_balance);
    }

    if source.total() < amount {
        return Err(SuiError::InsufficientBalance { coin_type: coin_type.to_string() });
    }

    build_amount_coin(txb, coin_type, amount, source.address_balance, &source.coins)
}

pub fn finish_transaction(txb: TransactionBuilder, input: TransactionBuilderInput) -> Result<TxOutput, SuiError> {
    if input.gas_objects.is_empty() {
        return Err(SuiError::NoGasCoins);
    }
    let sender = SuiAddress::parse(&input.sender)?.into();
    let transaction = Transaction {
        kind: TransactionKind::ProgrammableTransaction(txb.build()?),
        sender,
        gas_payment: GasPayment {
            objects: input.gas_objects.into_iter().map(gas_payment_object).collect::<Result<_, _>>()?,
            owner: sender,
            price: input.gas_price,
            budget: input.gas_budget,
        },
        expiration: TransactionExpiration::None,
    };
    TxOutput::from_tx(&transaction).map_err(SuiError::from_display)
}

fn gas_payment_object(object: ObjectInput) -> Result<ObjectReference, SuiError> {
    match object {
        ObjectInput::ImmutableOrOwned(reference) => Ok(reference),
        ObjectInput::Shared(_) | ObjectInput::Receiving(_) => Err(SuiError::invalid_input("Sui gas payment must be an owned object")),
    }
}

pub fn decode_transaction<T: DeserializeOwned>(encoded: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
    let bytes = decode_base64(encoded)?;
    let transaction = bcs::from_bytes::<T>(&bytes)?;
    Ok(transaction)
}

pub fn validate_and_hash(encoded: &str) -> Result<TxOutput, Box<dyn Error + Send + Sync>> {
    if encoded.trim().is_empty() {
        return Err("Missing Sui transaction data".into());
    }

    let transaction = decode_transaction(encoded).map_err(|err| format!("Invalid Sui transaction payload: {err}"))?;
    TxOutput::from_tx(&transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Coin;
    use primitives::hex;
    use std::str::FromStr;

    #[test]
    fn test_decode_transaction() {
        let encoded = "AAAPAAhkx5NBAAAAAAAIKUO8sgMAAAAAAQAAAQAAAQAACGTHk0EAAAAAAQFexM/GvrUlJRacMqd+FsKIt7/Lm4mCielL8xCFcLPvpBbjZwAAAAAAAQEB2qRikmMsPE2PMfI+oPmzaij/NnfpaEmA5EOEA6Z6PY8uBRgAAAAAAAABAYBJ0AkRYmmsBO4UIGt6/YtktYASefhUAe5LOXefgJE0zicvAAAAAAABAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAEB8ZTZsbytly5Fp91n3Umz7h4zV6AKUIUMUs1Ru0UOE7QXwmUAAAAAAAABASjkmd/16GSi6v5HYmmk9QNfHBbzONp74YsQNJmr8nHO7fIyAAAAAAABAQHwxA1nsHgADhgDIzTDMlxHueyfPZrkEovoINVGY9FOO+/yMgAAAAAAAQEBNdNbDlsXdZPYw6gBRiSFVy/DCGHmzpalWvbcRzBwknju8jIAAAAAAAAAIJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2BgIAAQEAAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFvYnRhaW5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAUCAAABAQABAgABAwABBAAA3BVyG6qCumSCLVhac0mhUI922UroDombBuSDacJXdQ4Ic3dhcF9jYXANaW5pdGlhdGVfcGF0aAEHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQACAgEAAQUAAB7GqMWsC4uXwofNNLn8apS1OgfJMKhQWVJnncjUs3gKBnJvdXRlchBzd2FwX2JfdG9fYV9ieV9iAwcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgNzdWkDU1VJAAfkI5zZUfbFPZxB4lJw2A0x+SWtFlXlultUOEPUpml17gRTVUlQBFNVSVAABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkABgEGAAIBAAEHAAEIAAICAAEJAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFyZXR1cm5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAYCAQACAwABCgABCwABDAABDQABAQIDAAEOAJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2AQAX1Cs2B1S8591qpdZjDUOB/CBDy2V8/6tqhBbwbdyxj734BAAAAAAg6yrtiW5R0TC68GDMmZye6U+KDjfZlq21n3bztRGzXjuT9luMFsJjNDu/Zs+fju9pyx28ktE/DDMbDcrrdrSqtu4CAAAAAAAA3P9fAAAAAAAA";
        let transaction: Transaction = decode_transaction(encoded).unwrap();

        assert_eq!(transaction.sender.to_string(), "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        let TransactionKind::ProgrammableTransaction(programmable) = transaction.kind;
        assert_eq!(programmable.commands.len(), 6);

        let output = validate_and_hash(encoded).unwrap();
        assert_eq!(hex::encode(output.hash), "883f6f54145fdaf357e3d404a8353b1f6eda265bc2b28ec8178631e092c24e3b");

        for vector in include_str!("../../testdata/dapp_transactions.txt").lines() {
            let (encoded, hash) = vector.split_once(' ').unwrap();
            let output = validate_and_hash(encoded).unwrap();

            assert_eq!(output.base64_encoded(), encoded);
            assert_eq!(hex::encode(output.hash), hash);
        }

        assert!(validate_and_hash("").is_err());
        assert!(validate_and_hash("AAAA").is_err());
        assert!(validate_and_hash(&format!("{encoded}AA")).is_err());
    }

    #[test]
    fn test_move_call() {
        let sender = "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1";
        let type_arguments = [
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "u256",
            "bool",
            "address",
            "signer",
            "vector<u8>",
            "vector<vector<0x2::sui::SUI>>",
            "0x2::coin::Coin<0x2::sui::SUI>",
            "0x00000000000000000000000000000000000000000000000000000000000000a::_m::_T1",
            "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
            "0x1::pair::Pair<0x2::sui::SUI,   0xabc::token::TOKEN<u64 , vector<bool>>>",
        ];
        let mut txb = TransactionBuilder::new();
        let coin = zero_coin(&mut txb, "0x2::sui::SUI").unwrap();
        let result = move_call(&mut txb, sui_framework_package_address(), "module_name", "function_name", &type_arguments, vec![coin]).unwrap();
        let nested = result.to_nested(2);
        let recipient = txb.pure(&Address::from_str(sender).unwrap());
        txb.transfer_objects(nested, recipient);
        let output = finish_transaction(txb, TransactionBuilderInput::new(sender, 750, 25_000_000, vec![Coin::mock_sui().to_input()])).unwrap();

        assert_eq!(hex::encode(output.hash), "cc55fe4db287ec0d264317603eb9d0bd459482bc4567d8cd21983ebf1ef0279f");

        for invalid in [
            "",
            "u8 ",
            " u8",
            "u7",
            "U64",
            "vector<u8",
            "vector<>",
            "0x2::sui",
            "2::sui::SUI",
            "0x2::sui::SUI<>",
            "0x2::1sui::SUI",
            "0x2::sui::SUI<u8,>",
            "0xg::sui::SUI",
            "0x2:sui::SUI",
        ] {
            let mut txb = TransactionBuilder::new();
            assert!(move_call(&mut txb, sui_framework_package_address(), "module_name", "function_name", &[invalid], vec![]).is_err(), "{invalid}");
        }
        for (module, function) in [("", "function_name"), ("module_name", ""), ("1module", "function_name"), ("module_name", "function-name"), ("_", "function_name")] {
            let mut txb = TransactionBuilder::new();
            assert!(move_call(&mut txb, sui_framework_package_address(), module, function, &[], vec![]).is_err(), "{module}::{function}");
        }
    }
}
