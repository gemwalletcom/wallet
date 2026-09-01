use crate::localization::DEFAULT_LANGUAGE;

use std::{collections::BTreeMap, error::Error, fs, path::Path};

pub fn write(localizations: &BTreeMap<String, Vec<(String, String)>>, output_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    for (language, entries) in localizations {
        let directory = output_path.join(values_directory(language));
        fs::create_dir_all(&directory)?;
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<resources>\n");
        for (key, value) in entries {
            let android_value = android_value(value);
            output.push_str("  <string name=\"");
            output.push_str(key);
            output.push_str("\">");
            output.push_str(&android_value);
            output.push_str("</string>\n");
        }
        output.push_str("</resources>\n");
        fs::write(directory.join("strings.xml"), output)?;
    }
    Ok(())
}

fn values_directory(language: &str) -> String {
    match language_qualifier(language) {
        None => "values".to_string(),
        Some(qualifier) => format!("values-{qualifier}"),
    }
}

fn language_qualifier(language: &str) -> Option<&str> {
    match language {
        DEFAULT_LANGUAGE => None,
        "he" => Some("iw"),
        "pt-BR" => Some("pt-rBR"),
        "zh-Hans" => Some("zh-rCN"),
        "zh-Hant" => Some("zh-rTW"),
        other => Some(other),
    }
}

fn android_value(value: &str) -> String {
    let mut output = String::new();
    let mut index = 1;
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '%' {
            match characters.peek() {
                Some('@') => {
                    characters.next();
                    output.push_str(&format!("%{index}$s"));
                    index += 1;
                }
                Some('d') => {
                    characters.next();
                    output.push_str(&format!("%{index}$d"));
                    index += 1;
                }
                _ => output.push(character),
            }
        } else if character == '\'' {
            output.push_str("\\'");
        } else {
            output.push(character);
        }
    }
    output
}
