pub(super) fn error_origin(logs: &[String], code: u32) -> Option<&str> {
    let (last, preceding) = logs.split_last()?;
    let mut program = parse_failure(last, code)?;

    for log in preceding.iter().rev() {
        let Some((logged_program, event)) = log.strip_prefix("Program ").and_then(|line| line.split_once(' ')) else {
            break;
        };
        if logged_program == program && event.starts_with("consumed ") {
            continue;
        }
        let Some(failed) = parse_failure(log, code) else {
            break;
        };
        program = failed;
    }

    Some(program)
}

fn parse_failure(log: &str, code: u32) -> Option<&str> {
    let (program, value) = log.strip_prefix("Program ")?.split_once(" failed: custom program error: 0x")?;
    (u32::from_str_radix(value, 16).ok()? == code).then_some(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TOKEN_PROGRAM, TOKEN_PROGRAM_2022};

    #[test]
    fn test_error_origin() {
        let token_failure = format!("Program {TOKEN_PROGRAM} failed: custom program error: 0x1");
        let parent_failure = "Program parent failed: custom program error: 0x1".to_string();
        let consumed = "Program parent consumed 5000 of 200000 compute units".to_string();
        let cases = [
            (vec![token_failure.clone(), consumed.clone(), parent_failure.clone()], Some(TOKEN_PROGRAM)),
            (vec![token_failure.clone(), "Program parent success".to_string(), parent_failure.clone()], Some("parent")),
            (
                vec![token_failure.clone(), "Program other invoke [2]".to_string(), consumed.clone(), parent_failure.clone()],
                Some("parent"),
            ),
            (
                vec![token_failure.clone(), "Program log: handled error".to_string(), consumed.clone(), parent_failure.clone()],
                Some("parent"),
            ),
            (vec![token_failure, consumed, "Program parent failed: custom program error: 0x2".to_string()], None),
            (
                vec![
                    "Program hook failed: custom program error: 0x1".to_string(),
                    format!("Program {TOKEN_PROGRAM_2022} consumed 5000 of 200000 compute units"),
                    format!("Program {TOKEN_PROGRAM_2022} failed: custom program error: 0x1"),
                ],
                Some("hook"),
            ),
        ];
        for (logs, expected) in cases {
            assert_eq!(error_origin(&logs, 1), expected, "{logs:?}");
        }
    }
}
