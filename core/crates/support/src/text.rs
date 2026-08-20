use primitives::{GEM_URL_SCHEME, HTTP_URL_SCHEME, HTTPS_URL_SCHEME, UrlAction};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportMessageDisplayContent {
    pub text: String,
    pub links: Vec<SupportMessageLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportMessageLink {
    pub title: String,
    pub url: String,
    pub subtitle: Option<String>,
}

pub fn markdown_plain_text(message: &str) -> String {
    collapse_whitespace(&message.replace("**", ""), " ")
}

pub fn parse_support_message_display_content(markdown: &str) -> SupportMessageDisplayContent {
    let markdown = markdown.trim();
    let mut text = String::new();
    let mut links = Vec::new();
    let mut current_index = 0;

    while let Some(title_start_offset) = markdown[current_index..].find('[') {
        let title_start = current_index + title_start_offset;
        let Some(title_end_offset) = markdown[title_start..].find(']') else {
            break;
        };
        let title_end = title_start + title_end_offset;
        let url_start_marker = title_end + 1;

        if !markdown[url_start_marker..].starts_with('(') {
            text.push_str(&markdown[current_index..=title_start]);
            current_index = title_start + 1;
            continue;
        }

        let url_start = url_start_marker + 1;
        let Some(url_end_offset) = markdown[url_start..].find(')') else {
            break;
        };
        let url_end = url_start + url_end_offset;
        let next_index = url_end + 1;
        let text_before_link = &markdown[current_index..title_start];
        let title = &markdown[title_start + 1..title_end];
        let url = &markdown[url_start..url_end];
        let label_prefix = link_label_prefix(text_before_link, title);
        let title = label_prefix.as_ref().map_or(title, |(_, label)| label);
        let text_before_link = label_prefix.as_ref().map_or(text_before_link, |(label_start, _)| &text_before_link[..*label_start]);

        match SupportMessageLink::new(Some(title), url) {
            Some(link) => {
                append_text_and_labeled_links(text_before_link, &mut text, &mut links);
                links.push(link);
            }
            None => text.push_str(&markdown[current_index..next_index]),
        }
        current_index = next_index;
    }

    append_text_and_labeled_links(&markdown[current_index..], &mut text, &mut links);

    SupportMessageDisplayContent {
        text: if links.is_empty() {
            markdown.to_string()
        } else {
            collapse_whitespace(&text, " ").trim().to_string()
        },
        links,
    }
}

impl SupportMessageLink {
    fn new(title: Option<&str>, url: &str) -> Option<Self> {
        let url = url.trim();
        let parsed_url = Url::parse(url).ok()?;
        let title = title.map(str::trim).filter(|title| !title.is_empty()).unwrap_or(url);

        if title.is_empty() || url.chars().any(char::is_whitespace) {
            return None;
        }

        let is_app_link = UrlAction::from_url(url).is_some();
        let subtitle = match parsed_url.scheme() {
            GEM_URL_SCHEME if is_app_link => None,
            GEM_URL_SCHEME => Some(url.to_string()),
            HTTP_URL_SCHEME | HTTPS_URL_SCHEME => {
                let host = parsed_url.host_str().filter(|host| !host.is_empty())?;
                if is_app_link {
                    None
                } else {
                    Some(host.strip_prefix("www.").unwrap_or(host).to_string())
                }
            }
            _ => return None,
        };

        Some(Self {
            title: title.to_string(),
            url: url.to_string(),
            subtitle,
        })
    }
}

fn append_text_and_labeled_links(markdown: &str, text: &mut String, links: &mut Vec<SupportMessageLink>) {
    let mut current_index = 0;

    while let Some(url_start) = next_bare_url_start(markdown, current_index) {
        let token_end = bare_url_token_end(&markdown[url_start..]) + url_start;
        let url_end = bare_url_end(&markdown[url_start..token_end]) + url_start;
        let text_before_url = &markdown[current_index..url_start];
        let url = &markdown[url_start..url_end];
        let Some((label_start, title)) = link_label_prefix(text_before_url, url) else {
            text.push_str(&markdown[current_index..token_end]);
            current_index = token_end;
            continue;
        };
        let text_before_url = &text_before_url[..label_start];

        match SupportMessageLink::new(Some(&title), url) {
            Some(link) => {
                text.push_str(text_before_url);
                links.push(link);
            }
            None => text.push_str(&markdown[current_index..token_end]),
        }
        current_index = token_end;
    }

    text.push_str(&markdown[current_index..]);
}

fn next_bare_url_start(text: &str, from: usize) -> Option<usize> {
    text[from..]
        .char_indices()
        .map(|(offset, _)| from + offset)
        .find(|&index| is_url_boundary(text, index) && starts_with_supported_url_scheme(&text[index..]))
}

fn starts_with_supported_url_scheme(text: &str) -> bool {
    [HTTP_URL_SCHEME, HTTPS_URL_SCHEME, GEM_URL_SCHEME]
        .into_iter()
        .any(|scheme| starts_with_url_scheme(text, scheme))
}

fn starts_with_url_scheme(text: &str, scheme: &str) -> bool {
    text.get(..scheme.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme)) && text[scheme.len()..].starts_with("://")
}

fn is_url_boundary(text: &str, index: usize) -> bool {
    index == 0
        || text[..index]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_whitespace() || matches!(character, '(' | '[' | '{' | '<' | '"' | '\'' | ':'))
}

fn bare_url_token_end(text: &str) -> usize {
    text.find(char::is_whitespace).unwrap_or(text.len())
}

fn bare_url_end(text: &str) -> usize {
    text.trim_end_matches(['.', ',', ';', '!', '?', ')', ']', '}', '"', '\'']).len()
}

fn collapse_whitespace(text: &str, separator: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
            (!line.is_empty()).then_some(line)
        })
        .collect::<Vec<_>>()
        .join(separator)
}

fn link_label_prefix(text: &str, link_title: &str) -> Option<(usize, String)> {
    let link_title = link_title.trim();
    let link_title_is_url = Url::parse(link_title).is_ok();
    let plain_link_title = (!link_title_is_url).then(|| markdown_plain_text(link_title));
    let mut end = text.len();

    while end > 0 {
        let line_start = text[..end].rfind('\n').map_or(0, |index| index + 1);
        let label = markdown_plain_text(&text[line_start..end]);
        let label = label.trim();

        if !label.is_empty() {
            let label = label.strip_suffix(':')?.trim();
            if !label.is_empty() && (link_title_is_url || plain_link_title.as_deref() == Some(label)) {
                return Some((line_start, label.to_string()));
            }
            return None;
        }
        if line_start == 0 {
            break;
        }
        end = line_start - 1;
    }

    None
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

    #[test]
    fn test_parse_support_message_display_content() {
        assert_eq!(
            parse_support_message_display_content(
                r#"
                The current price of Hyperliquid (HYPE) is $71.62.

                [Hyperliquid (HYPE)](https://hyperliquid.xyz)
                [Open market chart](https://www.coingecko.com/en/coins/hyperliquid)
                "#
            ),
            SupportMessageDisplayContent {
                text: "The current price of Hyperliquid (HYPE) is $71.62.".to_string(),
                links: vec![
                    link("Hyperliquid (HYPE)", "https://hyperliquid.xyz", Some("hyperliquid.xyz")),
                    link("Open market chart", "https://www.coingecko.com/en/coins/hyperliquid", Some("coingecko.com")),
                ],
            }
        );
        assert_eq!(
            parse_support_message_display_content("Here's the Bitcoin page: [Bitcoin](gem://tokens/bitcoin)"),
            SupportMessageDisplayContent {
                text: "Here's the Bitcoin page:".to_string(),
                links: vec![link("Bitcoin", "gem://tokens/bitcoin", None)],
            }
        );
        assert_eq!(
            parse_support_message_display_content("You can top up here: [Buy Bitcoin](gem://buy/bitcoin?amount=100)"),
            SupportMessageDisplayContent {
                text: "You can top up here:".to_string(),
                links: vec![link("Buy Bitcoin", "gem://buy/bitcoin?amount=100", None)],
            }
        );
        assert_eq!(
            parse_support_message_display_content("Here's the Bitcoin page: [Bitcoin](https://gemwallet.com/tokens/bitcoin)"),
            SupportMessageDisplayContent {
                text: "Here's the Bitcoin page:".to_string(),
                links: vec![link("Bitcoin", "https://gemwallet.com/tokens/bitcoin", None)],
            }
        );
        assert_eq!(
            parse_support_message_display_content(
                r#"
                Here's the Gem Wallet docs
                homepage:

                [Docs](https://docs.gemwallet.com)
                "#
            ),
            SupportMessageDisplayContent {
                text: "Here's the Gem Wallet docs homepage:".to_string(),
                links: vec![link("Docs", "https://docs.gemwallet.com", Some("docs.gemwallet.com"))],
            }
        );
        assert_eq!(
            parse_support_message_display_content(
                r#"
                Here are the links:

                **Docs:** https://docs.gemwallet.com

                **Robinhood announcement:**
                https://gemwallet.com/learn/gem-wallet-now-supports-robinhood-chain-store-and-swap-eth-and-tokenized-stocks/

                Transaction:
                https://etherscan.io/tx/0x123.

                Deep link: gem://tokens/bitcoin

                Which docs article were you looking for specifically?
                "#
            ),
            SupportMessageDisplayContent {
                text: "Here are the links: Which docs article were you looking for specifically?".to_string(),
                links: vec![
                    link("Docs", "https://docs.gemwallet.com", Some("docs.gemwallet.com")),
                    link(
                        "Robinhood announcement",
                        "https://gemwallet.com/learn/gem-wallet-now-supports-robinhood-chain-store-and-swap-eth-and-tokenized-stocks/",
                        Some("gemwallet.com"),
                    ),
                    link("Transaction", "https://etherscan.io/tx/0x123", Some("etherscan.io")),
                    link("Deep link", "gem://tokens/bitcoin", None),
                ],
            }
        );
        assert_eq!(
            parse_support_message_display_content("Open https://google.com for search."),
            SupportMessageDisplayContent {
                text: "Open https://google.com for search.".to_string(),
                links: vec![],
            }
        );
        assert_eq!(
            parse_support_message_display_content(
                r#"
                Here are the links:

                **Docs:** [https://docs.gemwallet.com](https://docs.gemwallet.com/)

                **Robinhood announcement:**
                [https://gemwallet.com/learn/gem-wallet-now-supports-robinhood-chain-store-and-swap-eth-and-tokenized-stocks/](https://gemwallet.com/learn/gem-wallet-now-supports-robinhood-chain-store-and-swap-eth-and-tokenized-stocks/)

                Which docs article were you looking for specifically?
                "#
            ),
            SupportMessageDisplayContent {
                text: "Here are the links: Which docs article were you looking for specifically?".to_string(),
                links: vec![
                    link("Docs", "https://docs.gemwallet.com/", Some("docs.gemwallet.com")),
                    link(
                        "Robinhood announcement",
                        "https://gemwallet.com/learn/gem-wallet-now-supports-robinhood-chain-store-and-swap-eth-and-tokenized-stocks/",
                        Some("gemwallet.com"),
                    ),
                ],
            }
        );
        let inline_text = "Read [the guide](javascript:alert(1)) before continuing.";
        assert_eq!(
            parse_support_message_display_content(inline_text),
            SupportMessageDisplayContent {
                text: inline_text.to_string(),
                links: vec![],
            }
        );
    }

    fn link(title: &str, url: &str, subtitle: Option<&str>) -> SupportMessageLink {
        SupportMessageLink {
            title: title.to_string(),
            url: url.to_string(),
            subtitle: subtitle.map(str::to_string),
        }
    }
}
