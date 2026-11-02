use primitives::single_line;

const MAX_NAME_CHARS: usize = 64;
pub(crate) const MAX_MESSAGE_CHARS: usize = 280;

pub(crate) fn merchant_name(name: &str) -> String {
    capped(name, MAX_NAME_CHARS)
}

pub(crate) fn merchant_message(message: &str) -> String {
    capped(message, MAX_MESSAGE_CHARS)
}

/// Merchant text shows above the warnings on Confirm, so it stays short.
fn capped(text: &str, max_chars: usize) -> String {
    let line = single_line(text);
    match line.chars().nth(max_chars) {
        Some(_) => format!("{}…", line.chars().take(max_chars).collect::<String>()),
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_name_drops_direction_characters() {
        assert_eq!(merchant_name("Constant\u{202e} K"), "Constant K");
        assert_eq!(merchant_name(&"k".repeat(MAX_NAME_CHARS + 1)), format!("{}…", "k".repeat(MAX_NAME_CHARS)));
    }
}
