pub fn to_slack_mrkdwn(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_code = false;
    for line in s.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            out.push_str(line);
            continue;
        }
        if in_code {
            out.push_str(line);
            continue;
        }
        let with_links = convert_markdown_links(line);
        let mut chars = with_links.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '*' && chars.peek() == Some(&'*') {
                chars.next();
                out.push('*');
            } else {
                out.push(c);
            }
        }
    }
    out
}

fn convert_markdown_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some((before, after_open)) = rest.split_once('[') {
        out.push_str(before);
        if let Some((label, url, tail)) = parse_link(after_open) {
            out.push('<');
            out.push_str(url);
            out.push('|');
            out.push_str(label);
            out.push('>');
            rest = tail;
        } else {
            out.push('[');
            rest = after_open;
        }
    }
    out.push_str(rest);
    out
}

fn parse_link(s: &str) -> Option<(&str, &str, &str)> {
    let (label, after) = s.split_once("](")?;
    let (url, tail) = after.split_once(')')?;
    Some((label, url, tail))
}

#[cfg(test)]
mod tests {
    use super::to_slack_mrkdwn;

    #[test]
    fn bold() {
        assert_eq!(to_slack_mrkdwn("**bold**"), "*bold*");
        assert_eq!(to_slack_mrkdwn("a **b c** d"), "a *b c* d");
    }

    #[test]
    fn leaves_correct_mrkdwn_alone() {
        assert_eq!(to_slack_mrkdwn("*already*"), "*already*");
        assert_eq!(to_slack_mrkdwn("plain text"), "plain text");
    }

    #[test]
    fn preserves_code_blocks() {
        assert_eq!(to_slack_mrkdwn("```\n**kept inside code**\n```"), "```\n**kept inside code**\n```");
    }

    #[test]
    fn converts_markdown_links() {
        assert_eq!(
            to_slack_mrkdwn("see [#4670](https://example.com/4670) for context"),
            "see <https://example.com/4670|#4670> for context"
        );
        assert_eq!(to_slack_mrkdwn("[a](u1) and [b](u2)"), "<u1|a> and <u2|b>");
        assert_eq!(to_slack_mrkdwn("no link here"), "no link here");
        assert_eq!(to_slack_mrkdwn("[bracket only]"), "[bracket only]");
        assert_eq!(to_slack_mrkdwn("**bold** and [link](url)"), "*bold* and <url|link>");
        assert_eq!(to_slack_mrkdwn("[foo](bar"), "[foo](bar");
    }
}
