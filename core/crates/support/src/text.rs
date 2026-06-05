pub fn markdown_plain_text(message: &str) -> String {
    let message = message.replace("**", "");
    let mut result = String::with_capacity(message.len());

    for word in message.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_plain_text() {
        assert_eq!(
            markdown_plain_text("Gem Wallet charges a **0.5% fee** on all swaps.\nFAQ: https://docs.gemwallet.com/faq/swap-fees"),
            "Gem Wallet charges a 0.5% fee on all swaps. FAQ: https://docs.gemwallet.com/faq/swap-fees"
        );
    }
}
