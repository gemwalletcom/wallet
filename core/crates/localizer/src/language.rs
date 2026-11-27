use std::collections::HashMap;
use std::fmt::Display;
use std::sync::LazyLock;

use gem_fluent::Element;

include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

const FIRST_STRONG_ISOLATE: char = '\u{2068}';
const POP_DIRECTIONAL_ISOLATE: char = '\u{2069}';
const MAX_REFERENCE_DEPTH: usize = 4;

static LANGUAGES: LazyLock<HashMap<&str, Language>> = LazyLock::new(|| SOURCES.iter().map(|(name, source)| (*name, Language::new(source))).collect());

pub(crate) type Arguments<'a> = [(&'a str, &'a dyn Display)];

pub(crate) fn name(language: &str) -> &'static str {
    SOURCES.iter().map(|(name, _)| *name).find(|name| *name == language).unwrap_or(FALLBACK)
}

pub(crate) fn format(language: &str, id: &str, arguments: &Arguments) -> String {
    [language, FALLBACK]
        .iter()
        .filter_map(|name| LANGUAGES.get(name))
        .find_map(|language| language.messages.get(id).map(|pattern| language.resolve(pattern, arguments)))
        .unwrap_or_else(|| id.to_string())
}

struct Language {
    messages: HashMap<&'static str, Vec<Element<'static>>>,
}

impl Language {
    fn new(source: &'static str) -> Self {
        let mut messages = HashMap::new();
        for message in gem_fluent::parse(source).unwrap_or_default() {
            messages.entry(message.id).or_insert(message.pattern);
        }
        Self { messages }
    }

    fn resolve(&self, pattern: &[Element], arguments: &Arguments) -> String {
        let mut output = String::new();
        self.write(pattern, arguments, 0, &mut output);
        output
    }

    fn write(&self, pattern: &[Element], arguments: &Arguments, depth: usize, output: &mut String) {
        for element in pattern {
            match element {
                Element::Text(text) => output.push_str(text),
                Element::Message(id) => match self.messages.get(id).filter(|_| depth < MAX_REFERENCE_DEPTH) {
                    Some(pattern) => self.write(pattern, arguments, depth + 1, output),
                    None => output.push_str(&format!("{{{id}}}")),
                },
                Element::Variable(name) => {
                    let value = arguments.iter().find(|(argument, _)| argument == name).map_or_else(|| format!("{{${name}}}"), |(_, value)| value.to_string());
                    match pattern.len() {
                        1 => output.push_str(&value),
                        _ => {
                            output.push(FIRST_STRONG_ISOLATE);
                            output.push_str(&value);
                            output.push(POP_DIRECTIONAL_ISOLATE);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(name("pt-BR"), "pt-BR");
        assert_eq!(name("pt"), FALLBACK);
        assert_eq!(name(""), FALLBACK);
    }

    #[test]
    fn test_format() {
        assert_eq!(format("en", id::notification_sent_title, &[("value", &"1 BTC")]), "💸 Sent: \u{2068}1 BTC\u{2069}");
        assert_eq!(format("en", id::notification_sent_title, &[]), "💸 Sent: \u{2068}{$value}\u{2069}");
        assert_eq!(format("en", id::notification_reward_title, &[("value", &1)]), "Points earned: \u{2068}1\u{2069}");
        assert_eq!(format("en", "missing_message", &[]), "missing_message");
        assert_eq!(format("pt-BR", id::notification_test, &[]), "Teste");
        assert_eq!(format("pt", id::notification_test, &[]), "Test");
    }

    #[test]
    fn test_new() {
        assert_eq!(Language::new("test = Test\ntest = Again").messages["test"], vec![Element::Text("Test")]);
        assert_eq!(Language::new("test = Test\ninvalid").messages.len(), 0);
    }

    #[test]
    fn test_resolve() {
        let cyclic = Language::new("first = { second }\nsecond = { first }");

        assert_eq!(cyclic.resolve(&cyclic.messages["first"], &[]), "{second}");
        assert_eq!(cyclic.resolve(&[Element::Variable("value")], &[("value", &"1 BTC")]), "1 BTC");
    }
}
