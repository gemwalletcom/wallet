mod swift;

use serde_json::{Map, Value, json};

use crate::localization::{ANDROID_ONLY_KEYS, DEFAULT_LANGUAGE};

use std::{collections::BTreeMap, error::Error, fs, path::Path};

const CATALOG_FILE_NAME: &str = "Localizable.xcstrings";
const INFO_PLIST_CATALOG_FILE_NAME: &str = "InfoPlist.xcstrings";
const RESOURCES_DIRECTORY: &str = "Resources";
const ESCAPED_NEWLINE: &str = "\\n";

pub fn write_app(localizations: &BTreeMap<String, Vec<(String, String)>>, package_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    write_package(localizations, package_path, "Localized", "Localized.swift", ANDROID_ONLY_KEYS)
}

pub fn write_widget(localizations: &BTreeMap<String, Vec<(String, String)>>, package_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    write_package(localizations, package_path, "WidgetLocalized", "WidgetLocalized.swift", &[])
}

pub fn write_info_plist(localizations: &BTreeMap<String, Vec<(String, String)>>, output_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    let entries = catalog_entries(localizations, &[], &BTreeMap::new());
    fs::create_dir_all(output_path)?;
    fs::write(output_path.join(INFO_PLIST_CATALOG_FILE_NAME), catalog_json(&entries)?)?;
    Ok(())
}

fn write_package(
    localizations: &BTreeMap<String, Vec<(String, String)>>,
    package_path: &Path,
    enum_name: &str,
    swift_file_name: &str,
    excluded_keys: &[&str],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let catalog_path = package_path.join(RESOURCES_DIRECTORY).join(CATALOG_FILE_NAME);
    let key_map = read_key_map(&catalog_path)?;
    let entries = catalog_entries(localizations, excluded_keys, &key_map);
    let default_entries = entries.get(DEFAULT_LANGUAGE).ok_or("default localization was checked")?;
    let swift_source = swift::source(enum_name, default_entries)?;
    fs::create_dir_all(catalog_path.parent().ok_or("invalid catalog path")?)?;
    fs::write(&catalog_path, catalog_json(&entries)?)?;
    fs::write(package_path.join(swift_file_name), swift_source)?;
    Ok(())
}

fn catalog_entries(
    localizations: &BTreeMap<String, Vec<(String, String)>>,
    excluded_keys: &[&str],
    key_map: &BTreeMap<String, String>,
) -> BTreeMap<String, BTreeMap<String, String>> {
    localizations
        .iter()
        .map(|(language, entries)| {
            let entries = entries
                .iter()
                .filter(|(key, _)| !excluded_keys.contains(&key.as_str()))
                .map(|(key, value)| (ios_key(key, key_map), value.clone()))
                .collect();
            (language.clone(), entries)
        })
        .collect()
}

fn ios_key(key: &str, key_map: &BTreeMap<String, String>) -> String {
    let mapped_key = key_map.get(key);
    if mapped_key.is_some_and(|key| key.contains('.')) {
        return mapped_key.expect("mapped key was checked").to_string();
    }
    let Some(prefix) = key_map
        .values()
        .flat_map(|key| group_prefixes(key))
        .filter(|prefix| key.starts_with(&format!("{}_", prefix.replace('.', "_"))))
        .max_by_key(|prefix| prefix.len())
    else {
        return mapped_key.cloned().unwrap_or_else(|| key.to_string());
    };
    format!("{prefix}.{}", &key[prefix.len() + 1..])
}

fn group_prefixes(key: &str) -> Vec<&str> {
    key.match_indices('.').map(|(index, _)| &key[..index]).collect()
}

fn read_key_map(catalog_path: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error + Send + Sync>> {
    if !catalog_path.exists() {
        return Ok(BTreeMap::new());
    }
    let catalog: Value = serde_json::from_str(&fs::read_to_string(catalog_path)?)?;
    let strings = catalog
        .get("strings")
        .and_then(|value| value.as_object())
        .ok_or_else(|| format!("{} contains no strings object", catalog_path.display()))?;
    Ok(strings.keys().map(|key| (key.replace('.', "_"), key.clone())).collect())
}

fn catalog_json(entries: &BTreeMap<String, BTreeMap<String, String>>) -> Result<String, Box<dyn Error + Send + Sync>> {
    let default_entries = entries.get(DEFAULT_LANGUAGE).ok_or("default localization was checked")?;
    let mut strings = Map::new();
    for key in default_entries.keys() {
        let mut localizations = Map::new();
        for (language, language_entries) in entries {
            let value = language_entries.get(key).ok_or_else(|| format!("{language} is missing key {key}"))?;
            let value = value.replace(ESCAPED_NEWLINE, "\n");
            localizations.insert(language.clone(), json!({"stringUnit": {"state": "translated", "value": value}}));
        }
        strings.insert(key.clone(), json!({"extractionState": "manual", "localizations": localizations}));
    }
    let catalog = json!({"sourceLanguage": DEFAULT_LANGUAGE, "strings": strings, "version": "1.0"});
    Ok(serde_json::to_string_pretty(&catalog)? + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_map() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("common_cancel".to_string(), "common.cancel".to_string()),
            ("secret_phrase_title".to_string(), "secret_phrase.title".to_string()),
            ("errors_import_invalid_secret_phrase".to_string(), "errors.import.invalid_secret_phrase".to_string()),
        ])
    }

    #[test]
    fn test_ios_key() {
        assert_eq!(ios_key("common_cancel", &key_map()), "common.cancel");
        assert_eq!(ios_key("common_new_key", &key_map()), "common.new_key");
        assert_eq!(ios_key("secret_phrase_new_key", &key_map()), "secret_phrase.new_key");
        assert_eq!(ios_key("errors_import_invalid_private_key", &key_map()), "errors.import.invalid_private_key");
        assert_eq!(ios_key("errors_important_note", &key_map()), "errors.important_note");
        assert_eq!(ios_key("errors_new_key", &key_map()), "errors.new_key");
        assert_eq!(ios_key("unknown_key", &key_map()), "unknown_key");
        assert_eq!(ios_key("unknown_key", &BTreeMap::new()), "unknown_key");
    }

    #[test]
    fn test_catalog_json() {
        let entries = BTreeMap::from([
            ("en".to_string(), BTreeMap::from([("common.cancel".to_string(), "Cancel\\nNow".to_string())])),
            ("ru".to_string(), BTreeMap::from([("common.cancel".to_string(), "Отмена".to_string())])),
        ]);
        let catalog = catalog_json(&entries).unwrap();
        let expected = "{\n  \"sourceLanguage\": \"en\",\n  \"strings\": {\n    \"common.cancel\": {\n      \"extractionState\": \"manual\",\n      \"localizations\": {\n        \"en\": {\n          \"stringUnit\": {\n            \"state\": \"translated\",\n            \"value\": \"Cancel\\nNow\"\n          }\n        },\n        \"ru\": {\n          \"stringUnit\": {\n            \"state\": \"translated\",\n            \"value\": \"Отмена\"\n          }\n        }\n      }\n    }\n  },\n  \"version\": \"1.0\"\n}\n";
        assert_eq!(catalog, expected);
        assert!(catalog_json(&BTreeMap::new()).is_err());
    }
}
