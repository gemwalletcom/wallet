mod error;

pub use error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'a> {
    pub id: &'a str,
    pub pattern: Vec<Element<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element<'a> {
    Text(&'a str),
    Variable(&'a str),
    Message(&'a str),
}

pub fn parse(source: &str) -> Result<Vec<Message<'_>>, ParseError> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|(index, line)| {
            let (id, value) = line.split_once('=').ok_or(ParseError { line: index + 1 })?;
            match (identifier(id.trim_end()), parse_pattern(value.trim())) {
                (Some(id), Some(pattern)) => Ok(Message { id, pattern }),
                _ => Err(ParseError { line: index + 1 }),
            }
        })
        .collect()
}

fn identifier(text: &str) -> Option<&str> {
    let valid = text.starts_with(|character: char| character.is_ascii_alphabetic()) && text.chars().all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-');
    valid.then_some(text)
}

fn parse_pattern(mut text: &str) -> Option<Vec<Element<'_>>> {
    let mut elements = Vec::new();
    while let Some((before, placeable)) = text.split_once('{') {
        if !before.is_empty() {
            elements.push(Element::Text(before));
        }
        let (reference, after) = placeable.split_once('}')?;
        let reference = reference.trim();
        elements.push(match reference.strip_prefix('$') {
            Some(variable) => Element::Variable(identifier(variable)?),
            None => Element::Message(identifier(reference)?),
        });
        text = after;
    }
    if text.contains('}') {
        return None;
    }
    if !text.is_empty() {
        elements.push(Element::Text(text));
    }
    Some(elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let source = "# Comment\n\nplain = Plain text \nargument = Sent: {$value}\nreference = { plain } {$value}\n";

        assert_eq!(
            parse(source),
            Ok(vec![
                Message {
                    id: "plain",
                    pattern: vec![Element::Text("Plain text")],
                },
                Message {
                    id: "argument",
                    pattern: vec![Element::Text("Sent: "), Element::Variable("value")],
                },
                Message {
                    id: "reference",
                    pattern: vec![Element::Message("plain"), Element::Text(" "), Element::Variable("value")],
                },
            ])
        );
        assert_eq!(parse("empty ="), Ok(vec![Message { id: "empty", pattern: vec![] }]));
        assert_eq!(parse("first = One\nwithout a value"), Err(ParseError { line: 2 }));
        assert_eq!(parse("-term = Term"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = Text\n    .attribute = Value"), Err(ParseError { line: 2 }));
        assert_eq!(parse("message = Stray } brace"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = { -term }"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = { NUMBER($value) }"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = { \"literal\" }"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = { other.attribute }"), Err(ParseError { line: 1 }));
        assert_eq!(parse("message = { $value ->\n    [one] One\n   *[other] Other\n    }"), Err(ParseError { line: 1 }));
    }
}
