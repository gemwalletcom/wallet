use std::collections::HashMap;

use gem_encoding::decode_base64;

use super::{
    model::{TransactionArgument, TransactionCommand, TransactionInput, TransactionObject, WithdrawalReservation, WithdrawalTypeArg},
    replay::TransactionJsonReplay,
};
use crate::{SUI_COIN_TYPE, SuiError, full_coin_type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WalletObject {
    Coin(String),
    Other,
}

impl TransactionJsonReplay {
    pub fn wallet_spend(&self) -> Result<HashMap<String, u64>, SuiError> {
        let mut spend = HashMap::<String, u64>::new();
        let mut add = |coin_type: String, amount: u64| {
            let total = spend.entry(coin_type).or_default();
            *total = total.checked_add(amount).ok_or_else(|| SuiError::invalid_input("Sui transaction spends more than a coin can hold"))?;
            Ok::<(), SuiError>(())
        };
        for input in &self.transaction.inputs {
            if let TransactionInput::FundsWithdrawal { funds_withdrawal } = input {
                let WithdrawalReservation::MaxAmountU64 { max_amount } = funds_withdrawal.reservation;
                let WithdrawalTypeArg::Balance { balance } = &funds_withdrawal.type_arg;
                add(full_coin_type(balance), max_amount)?;
            }
        }
        for command in &self.transaction.commands {
            match command {
                TransactionCommand::SplitCoins { split_coins } => {
                    if let Some(coin_type) = self.wallet_coin(&split_coins.coin)? {
                        for amount in &split_coins.amounts {
                            add(coin_type.clone(), self.pure_u64(amount)?)?;
                        }
                    }
                }
                TransactionCommand::MergeCoins { merge_coins } => {
                    if self.wallet_coin(&merge_coins.destination)?.is_none() {
                        for source in &merge_coins.sources {
                            self.refuse_wallet_coin(source)?;
                        }
                    }
                }
                TransactionCommand::MoveCall { .. } | TransactionCommand::TransferObjects { .. } | TransactionCommand::MakeMoveVec { .. } | TransactionCommand::Publish { .. } | TransactionCommand::Upgrade { .. } => {
                    for argument in command.arguments() {
                        self.refuse_wallet_coin(argument)?;
                    }
                }
            }
        }
        Ok(spend)
    }

    pub fn unused_wallet_split(&self, argument: &TransactionArgument) -> Option<(String, u64)> {
        let TransactionArgument::NestedResult { nested_result: [command, index] } = *argument else {
            return None;
        };
        let TransactionCommand::SplitCoins { split_coins } = self.transaction.commands.get(command)? else {
            return None;
        };
        let coin_type = self.wallet_coin(&split_coins.coin).ok()??;
        let amount = self.pure_u64(split_coins.amounts.get(index)?).ok()?;
        let used = self.transaction.commands.iter().flat_map(TransactionCommand::arguments).any(|used| match *used {
            TransactionArgument::Result { result } => result == command,
            TransactionArgument::NestedResult { nested_result } => nested_result == [command, index],
            TransactionArgument::GasCoin { .. } | TransactionArgument::Input { .. } => false,
        });
        (!used).then_some((coin_type, amount))
    }

    fn wallet_coin(&self, argument: &TransactionArgument) -> Result<Option<String>, SuiError> {
        let index = match argument {
            TransactionArgument::GasCoin { .. } => return Ok(Some(full_coin_type(SUI_COIN_TYPE))),
            TransactionArgument::Input { input } => *input,
            TransactionArgument::Result { .. } | TransactionArgument::NestedResult { .. } => return Ok(None),
        };
        match self.transaction.inputs.get(index).ok_or_else(|| SuiError::invalid_input("Missing Sui input argument"))? {
            TransactionInput::UnresolvedObject { .. } => match self.wallet_objects.get(&index) {
                Some(WalletObject::Coin(coin_type)) => Ok(Some(coin_type.clone())),
                Some(WalletObject::Other) => Err(SuiError::invalid_input("Sui transaction uses a wallet object that is not a coin")),
                None => Ok(None),
            },
            TransactionInput::Object { object } => match object {
                TransactionObject::SharedObject { .. } => Ok(None),
                TransactionObject::ImmOrOwnedObject { .. } | TransactionObject::Receiving { .. } => Err(SuiError::invalid_input("Sui transaction names an object whose owner was not read")),
            },
            TransactionInput::Pure { .. } | TransactionInput::FundsWithdrawal { .. } | TransactionInput::UnresolvedPure { .. } => Ok(None),
        }
    }

    fn refuse_wallet_coin(&self, argument: &TransactionArgument) -> Result<(), SuiError> {
        match self.wallet_coin(argument)? {
            Some(_) => Err(SuiError::invalid_input("Sui transaction hands a wallet coin over whole")),
            None => Ok(()),
        }
    }

    fn pure_u64(&self, argument: &TransactionArgument) -> Result<u64, SuiError> {
        let unreadable = || SuiError::invalid_input("Sui transaction splits a wallet coin by an amount that cannot be read");
        let TransactionArgument::Input { input } = argument else {
            return Err(unreadable());
        };
        let Some(TransactionInput::Pure { pure }) = self.transaction.inputs.get(*input) else {
            return Err(unreadable());
        };
        decode_base64(&pure.bytes).ok().and_then(|bytes| bytes.try_into().ok()).map(u64::from_le_bytes).ok_or_else(unreadable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::encode_base64;
    use serde_json::{Value, json};

    const WAL: &str = "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL";

    fn spend(inputs: Value, commands: Value, wallet_coins: &[(usize, &str)]) -> Result<HashMap<String, u64>, SuiError> {
        let transaction = json!({ "version": 2, "inputs": inputs, "commands": commands });
        TransactionJsonReplay::mock_with_wallet_coins(&transaction.to_string(), wallet_coins).wallet_spend()
    }

    fn amount(value: u64) -> Value {
        json!({ "Pure": { "bytes": encode_base64(&value.to_le_bytes()) } })
    }

    fn object(id: u8) -> Value {
        json!({ "UnresolvedObject": { "objectId": format!("0x{id:064x}") } })
    }

    fn swap(coin: Value) -> Value {
        json!({ "MoveCall": { "package": "0x2", "module": "router", "function": "swap", "typeArguments": [], "arguments": [coin] } })
    }

    #[test]
    fn test_wallet_spend() {
        let sui = full_coin_type(SUI_COIN_TYPE);
        let wal = full_coin_type(WAL);

        assert_eq!(
            spend(
                json!([amount(0), amount(1_000_000_000), object(9)]),
                json!([{ "SplitCoins": { "coin": { "GasCoin": true }, "amounts": [{ "Input": 0 }, { "Input": 1 }] } }, swap(json!({ "NestedResult": [0, 1] })), swap(json!({ "Input": 2 }))]),
                &[]
            )
            .unwrap(),
            HashMap::from([(sui.clone(), 1_000_000_000)]),
            "SUI split from the gas coin, with a shared pool beside it"
        );
        assert_eq!(
            spend(
                json!([object(1), object(2), amount(5_000_000_000)]),
                json!([
                    { "MergeCoins": { "destination": { "Input": 0 }, "sources": [{ "Input": 1 }] } },
                    { "SplitCoins": { "coin": { "Input": 0 }, "amounts": [{ "Input": 2 }] } },
                    swap(json!({ "NestedResult": [1, 0] }))
                ]),
                &[(0, WAL), (1, WAL)]
            )
            .unwrap(),
            HashMap::from([(wal.clone(), 5_000_000_000)]),
            "a token split from the wallet's merged coins"
        );
        assert_eq!(
            spend(
                json!([{ "FundsWithdrawal": { "reservation": { "MaxAmountU64": "7" }, "typeArg": { "Balance": SUI_COIN_TYPE }, "withdrawFrom": { "Sender": true } } }]),
                json!([swap(json!({ "Input": 0 }))]),
                &[]
            )
            .unwrap(),
            HashMap::from([(sui, 7)]),
            "a withdrawal from the address balance counts in full"
        );

        for (inputs, commands, wallet_coins, reason) in [
            (json!([]), json!([swap(json!({ "GasCoin": true }))]), vec![], "the gas coin handed to a call whole"),
            (json!([object(1)]), json!([swap(json!({ "Input": 0 }))]), vec![(0, WAL)], "a wallet coin handed to a call whole"),
            (
                json!([object(1), amount(1)]),
                json!([{ "TransferObjects": { "objects": [{ "Input": 0 }], "address": { "Input": 1 } } }]),
                vec![(0, WAL)],
                "a wallet coin transferred away",
            ),
            (
                json!([object(1), amount(1)]),
                json!([
                    { "SplitCoins": { "coin": { "GasCoin": true }, "amounts": [{ "Input": 1 }] } },
                    { "MergeCoins": { "destination": { "NestedResult": [0, 0] }, "sources": [{ "Input": 0 }] } }
                ]),
                vec![(0, WAL)],
                "a wallet coin merged whole into a coin the calls may take",
            ),
            (
                json!([object(1)]),
                json!([swap(json!({ "Input": 0 })), { "SplitCoins": { "coin": { "GasCoin": true }, "amounts": [{ "Result": 0 }] } }]),
                vec![],
                "a split whose amount a call decides",
            ),
            (
                json!([{ "Object": { "ImmOrOwnedObject": { "objectId": format!("0x{:064x}", 1), "version": 1, "digest": "11111111111111111111111111111111" } } }]),
                json!([swap(json!({ "Input": 0 }))]),
                vec![],
                "an object whose owner the app did not read",
            ),
        ] {
            assert!(spend(inputs, commands, &wallet_coins).is_err(), "{reason}");
        }
    }

    #[test]
    fn test_unused_wallet_split() {
        let transaction = json!({
            "version": 2,
            "inputs": [amount(3), amount(5), object(1)],
            "commands": [
                { "SplitCoins": { "coin": { "GasCoin": true }, "amounts": [{ "Input": 0 }, { "Input": 1 }] } },
                swap(json!({ "NestedResult": [0, 0] })),
                { "SplitCoins": { "coin": { "NestedResult": [0, 0] }, "amounts": [{ "Input": 0 }] } },
                { "SplitCoins": { "coin": { "Input": 2 }, "amounts": [{ "Input": 1 }] } },
                { "MergeCoins": { "destination": { "NestedResult": [3, 0] }, "sources": [{ "NestedResult": [2, 0] }] } }
            ]
        });
        let replay = TransactionJsonReplay::mock_with_wallet_coins(&transaction.to_string(), &[(2, WAL)]);
        let unused = |command, index| replay.unused_wallet_split(&TransactionArgument::NestedResult { nested_result: [command, index] });

        assert_eq!(unused(0, 1), Some((full_coin_type(SUI_COIN_TYPE), 5)));
        assert_eq!(unused(0, 0), None, "a coin a call can split SUI off");
        assert_eq!(unused(2, 0), None, "a coin split from a coin the wallet does not hold");
        assert_eq!(unused(3, 0), None, "a coin something is merged into");
        assert_eq!(unused(0, 2), None);
        assert_eq!(replay.unused_wallet_split(&TransactionArgument::Result { result: 0 }), None);
        assert_eq!(replay.unused_wallet_split(&TransactionArgument::GasCoin { gas_coin: true }), None);
    }
}
