const SILENCE_PHRASES: &[&str] = &[
    "no output",
    "no response",
    "no reply",
    "nothing to add",
    "nothing useful to add",
    "nothing actionable to add",
    "no actionable input",
    "no actionable reply",
    "no actionable response",
    "no useful answer",
    "no useful reply",
    "not enough signal to respond",
    "staying silent",
    "stay silent",
    "ambient mode",
    "not actionable",
    "non-actionable",
    "no confident reply",
    "no confident answer",
];

enum BlockSearch<'a> {
    Found { before: &'a str, body: &'a str, after: &'a str },
    Unclosed { before: &'a str },
    None,
}

fn find_block<'a>(s: &'a str, open: &str, close: &str) -> BlockSearch<'a> {
    let Some(start) = s.find(open) else {
        return BlockSearch::None;
    };
    let before = &s[..start];
    let after_open = &s[start + open.len()..];
    let Some(end) = after_open.find(close) else {
        return BlockSearch::Unclosed { before };
    };
    BlockSearch::Found {
        before,
        body: &after_open[..end],
        after: &after_open[end + close.len()..],
    }
}

fn strip_thinking(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        match find_block(rest, "<thinking>", "</thinking>") {
            BlockSearch::Found { before, after, .. } => {
                out.push_str(before);
                rest = after;
            }
            BlockSearch::Unclosed { before } => {
                out.push_str(before);
                break;
            }
            BlockSearch::None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out.trim().to_string()
}

fn extract_replies(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let BlockSearch::Found { body, after, .. } = find_block(rest, "<reply>", "</reply>") {
        let trimmed = body.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
        rest = after;
    }
    out
}

pub enum ReplyOutcome {
    Tagged(Vec<String>),
    Untagged(String),
    Silent,
}

pub fn classify_reply(raw: &str) -> ReplyOutcome {
    let stripped = strip_thinking(raw);
    let tags = extract_replies(&stripped);

    if !tags.is_empty() {
        let kept: Vec<String> = tags.into_iter().filter(|c| !is_silence(c)).collect();
        return if kept.is_empty() { ReplyOutcome::Silent } else { ReplyOutcome::Tagged(kept) };
    }

    let trimmed = stripped.trim();
    if trimmed.is_empty() || is_silence(trimmed) {
        ReplyOutcome::Silent
    } else {
        ReplyOutcome::Untagged(trimmed.to_string())
    }
}

fn is_silence(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    let lower = t.to_lowercase();
    if !t.contains('\n') && t.chars().count() < 240 && SILENCE_PHRASES.iter().any(|p| lower.contains(p)) {
        return true;
    }
    let last_line = lower.lines().rev().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    SILENCE_PHRASES.iter().any(|p| last_line.contains(p))
}

#[cfg(test)]
mod tests {
    use super::is_silence;

    #[test]
    fn treats_empty_and_meta_replies_as_silence() {
        assert!(is_silence(""));
        assert!(is_silence("(no response)"));
        assert!(is_silence("(no output)"));
        assert!(is_silence("(no output - nothing to add)"));
        assert!(is_silence("(no useful answer)"));
        assert!(is_silence("(staying silent)"));
        assert!(is_silence("I have nothing useful to add."));
        assert!(is_silence("No actionable response."));
        assert!(is_silence("non-actionable"));
        assert!(is_silence("(no response — the reasoning is sound)"));
        assert!(is_silence("(no confident reply)"));
        assert!(is_silence("(no confident answer to give)"));
    }

    #[test]
    fn treats_reasoning_with_trailing_silence_as_silence() {
        let leak = "This is a single-word \"Test\" message with no actual question or issue — falls into the spam/gibberish/single-word category. I'll stay silent and let support sweep later if needed.\n\n(no confident reply)";
        assert!(is_silence(leak));
    }

    #[test]
    fn strips_thinking_blocks() {
        use super::strip_thinking;
        assert_eq!(strip_thinking("hello"), "hello");
        assert_eq!(strip_thinking("<thinking>let me think</thinking>actual reply"), "actual reply");
        assert_eq!(strip_thinking("before <thinking>middle\nlines</thinking> after"), "before  after");
        assert_eq!(strip_thinking("<thinking>one</thinking><thinking>two</thinking>visible"), "visible");
        assert_eq!(strip_thinking("<thinking>unclosed runaway"), "");
    }

    #[test]
    fn keeps_substantive_replies() {
        assert!(!is_silence("Can you share the app version and transaction hash?"));
        assert!(!is_silence(
            "I checked `gemwalletcom/core`; the likely owner is <@U123> because the staking client lives there."
        ));
    }

    #[test]
    fn extracts_single_reply_tag() {
        use super::extract_replies;
        let raw = "Reasoning paragraph that should never reach the customer.\n\n<reply>Привет! Чем могу помочь?</reply>";
        assert_eq!(extract_replies(raw), vec!["Привет! Чем могу помочь?".to_string()]);
    }

    #[test]
    fn extracts_multiple_reply_tags() {
        use super::extract_replies;
        let raw = "<reply>First message.</reply> some thinking <reply>Second message.</reply>";
        assert_eq!(extract_replies(raw), vec!["First message.".to_string(), "Second message.".to_string()]);
    }

    #[test]
    fn returns_empty_when_no_reply_tags() {
        use super::extract_replies;
        let raw = "Free-form reasoning with no reply tag.";
        assert!(extract_replies(raw).is_empty());
    }

    #[test]
    fn ignores_unclosed_reply_tag() {
        use super::extract_replies;
        let raw = "<reply>unclosed";
        assert!(extract_replies(raw).is_empty());
    }
}
