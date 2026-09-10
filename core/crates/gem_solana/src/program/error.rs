use primitives::ValueAccess;
use serde_json::Value;

use crate::{ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM, TOKEN_PROGRAM_2022};

use super::log::error_origin;

pub fn error_message(error: &Value, logs: &[String]) -> Option<&'static str> {
    let code = error.get_value("InstructionError").ok()?.at(1).ok()?.get_i64("Custom").ok()?;
    let code = u32::try_from(code).ok()?;
    known_program_error(error_origin(logs, code)?, code)
}

fn known_program_error(program: &str, code: u32) -> Option<&'static str> {
    Some(match program {
        TOKEN_PROGRAM | TOKEN_PROGRAM_2022 => match code {
            0 => "Lamport balance below rent-exempt threshold",
            1 => "Insufficient funds",
            2 => "Invalid mint",
            3 => "Account not associated with this mint",
            4 => "Owner does not match",
            5 => "Fixed supply",
            6 => "Already in use",
            7 => "Invalid number of provided signers",
            8 => "Invalid number of required signers",
            9 => "State is uninitialized",
            10 => "Instruction does not support native tokens",
            11 => "Non-native account can only be closed if its balance is zero",
            12 => "Invalid instruction",
            13 => "State is invalid for requested operation",
            14 => "Operation overflowed",
            15 => "Account does not support specified authority type",
            16 => "This token mint cannot freeze accounts",
            17 => "Account is frozen",
            18 => "Mint decimals mismatch",
            19 => "Instruction does not support non-native tokens",
            _ => return None,
        },
        SYSTEM_PROGRAM_ID => match code {
            0 => "An account with the same address already exists",
            1 => "Account does not have enough SOL to perform the operation",
            2 => "Cannot assign account to this program id",
            3 => "Cannot allocate account data of this length",
            4 => "Length of requested seed is too long",
            5 => "Provided address does not match address derived from seed",
            6 => "Advancing stored nonce requires a populated RecentBlockhashes sysvar",
            7 => "Stored nonce is still in recent blockhashes",
            8 => "Specified nonce does not match stored nonce",
            _ => return None,
        },
        ASSOCIATED_TOKEN_ACCOUNT_PROGRAM if code == 0 => "Associated token account owner does not match address derivation",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message() {
        let error: Value = serde_json::from_str(r#"{"InstructionError":[1,{"Custom":1}]}"#).unwrap();
        let cases = [
            (vec![format!("Program {TOKEN_PROGRAM} failed: custom program error: 0x1")], Some("Insufficient funds")),
            (vec![format!("Program {TOKEN_PROGRAM_2022} failed: custom program error: 0x1")], Some("Insufficient funds")),
            (
                vec![format!("Program {SYSTEM_PROGRAM_ID} failed: custom program error: 0x1")],
                Some("Account does not have enough SOL to perform the operation"),
            ),
            (vec![format!("Program {ASSOCIATED_TOKEN_ACCOUNT_PROGRAM} failed: custom program error: 0x1")], None),
            (vec!["Program unknown failed: custom program error: 0x1".to_string()], None),
            (vec![format!("Program {TOKEN_PROGRAM} failed: custom program error: 0x2")], None),
            (vec![format!("Program log: Program {TOKEN_PROGRAM} failed: custom program error: 0x1")], None),
            (
                vec![format!("Program {TOKEN_PROGRAM} failed: custom program error: 0x1"), "Log truncated".to_string()],
                None,
            ),
            (vec![], None),
        ];
        for (logs, expected) in cases {
            assert_eq!(error_message(&error, &logs), expected, "{logs:?}");
        }
    }

    #[test]
    fn test_error_message_invalid_error() {
        let logs = vec![format!("Program {TOKEN_PROGRAM} failed: custom program error: 0x1")];
        for raw in [
            "null",
            r#""BlockhashNotFound""#,
            r#"{"InstructionError":[]}"#,
            r#"{"InstructionError":[0,"InvalidArgument"]}"#,
            r#"{"InstructionError":[0,{"Custom":-1}]}"#,
            r#"{"InstructionError":[0,{"Custom":4294967296}]}"#,
            r#"{"InstructionError":[0,{"Custom":"1"}]}"#,
        ] {
            let error = serde_json::from_str(raw).unwrap();
            assert_eq!(error_message(&error, &logs), None, "{raw}");
        }
    }
}
