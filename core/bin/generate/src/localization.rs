mod android;
mod ios;

use fluent_syntax::{
    ast::{Entry, Pattern, PatternElement},
    parser,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_LANGUAGE: &str = "en";
pub const ANDROID_ONLY_KEYS: &[&str] = &[
    "application_name",
    "camera_permission_request_camera",
    "common_grant_permission",
    "common_no_thanks",
    "common_required_field",
    "confirm_fee_error",
    "errors_token_unable_fetch_token_information",
    "errors_unknown_try_again",
    "notifications_permission_request_notification",
    "rootcheck_body",
    "rootcheck_exit",
    "rootcheck_ignore",
    "rootcheck_security_alert",
    "transfer_amount",
    "transfer_amount_title",
    "update_app_downloading",
    "update_app_permission_description",
    "update_app_permission_open_settings",
    "update_app_permission_title",
];

pub fn generate(args: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
    let target = args.first().ok_or("no localization target specified")?;
    let source_path = args.get(1).ok_or("no localization source path specified")?;
    let output_path = args.get(2).ok_or("no localization output path specified")?;
    let localizations = read_localizations(Path::new(source_path))?;

    match target.as_str() {
        "ios" => ios::write_app(&localizations, Path::new(output_path))?,
        "ios-widget" => ios::write_widget(&localizations, Path::new(output_path))?,
        "ios-plist" => ios::write_info_plist(&localizations, Path::new(output_path))?,
        "android" => android::write(&localizations, Path::new(output_path))?,
        other => return Err(format!("unsupported localization target: {other}").into()),
    }

    Ok(())
}

fn read_localizations(source_path: &Path) -> Result<BTreeMap<String, Vec<(String, String)>>, Box<dyn Error + Send + Sync>> {
    let mut localizations = BTreeMap::new();
    for path in localization_paths(source_path)? {
        let language = path.file_stem().and_then(|value| value.to_str()).ok_or("invalid localization file name")?.to_string();
        let source = fs::read_to_string(&path)?;
        let resource = match parser::parse::<&str>(&source) {
            Ok(resource) => resource,
            Err((_, errors)) => return Err(format!("failed to parse {}: {:?}", path.display(), errors).into()),
        };
        let mut entries = Vec::new();
        let mut keys = BTreeSet::new();
        for entry in resource.body {
            match entry {
                Entry::Message(message) => {
                    if !keys.insert(message.id.name.to_string()) {
                        return Err(format!("{} contains duplicate key {}", path.display(), message.id.name).into());
                    }
                    if !message.attributes.is_empty() {
                        return Err(format!("{} contains unsupported attributes on {}", path.display(), message.id.name).into());
                    }
                    let value = message.value.ok_or_else(|| format!("{} contains empty value for {}", path.display(), message.id.name))?;
                    let value = pattern_text(&path, message.id.name, value)?;
                    validate_value(&path, message.id.name, &value)?;
                    entries.push((message.id.name.to_string(), value.trim().to_string()));
                }
                Entry::Comment(_) | Entry::GroupComment(_) | Entry::ResourceComment(_) => {}
                Entry::Term(term) => return Err(format!("{} contains unsupported term {}", path.display(), term.id.name).into()),
                Entry::Junk { content } => return Err(format!("{} contains invalid Fluent content: {}", path.display(), content).into()),
            }
        }
        localizations.insert(language, entries);
    }

    if !localizations.contains_key(DEFAULT_LANGUAGE) {
        return Err(format!("{} must contain {DEFAULT_LANGUAGE}.ftl", source_path.display()).into());
    }

    validate_key_sets(source_path, &localizations)?;

    Ok(localizations)
}

fn validate_key_sets(source_path: &Path, localizations: &BTreeMap<String, Vec<(String, String)>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let default_keys = key_set(localizations.get(DEFAULT_LANGUAGE).expect("default localization was checked"));
    for (language, entries) in localizations {
        let keys = key_set(entries);
        let missing = default_keys.difference(&keys).cloned().collect::<Vec<_>>();
        let extra = keys.difference(&default_keys).cloned().collect::<Vec<_>>();
        if !missing.is_empty() || !extra.is_empty() {
            return Err(format!(
                "{} {language}.ftl must match {DEFAULT_LANGUAGE}.ftl keys: missing [{}], extra [{}]",
                source_path.display(),
                missing.join(", "),
                extra.join(", ")
            )
            .into());
        }
    }
    Ok(())
}

fn key_set(entries: &[(String, String)]) -> BTreeSet<String> {
    entries.iter().map(|(key, _)| key.clone()).collect()
}

fn localization_paths(source_path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error + Send + Sync>> {
    let mut paths = fs::read_dir(source_path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("ftl"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn pattern_text(path: &Path, id: &str, pattern: Pattern<&str>) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut value = String::new();
    for element in pattern.elements {
        match element {
            PatternElement::TextElement { value: text } => value.push_str(text),
            PatternElement::Placeable { .. } => return Err(format!("{} contains unsupported placeable in {}", path.display(), id).into()),
        }
    }
    Ok(value)
}

fn validate_value(path: &Path, id: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    if value.contains('"') || value.contains('&') || value.contains('<') || value.contains('>') {
        return Err(format!("{} contains unsupported native string syntax in {}", path.display(), id).into());
    }
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\\' && characters.next() != Some('n') {
            return Err(format!("{} contains unsupported escape in {}", path.display(), id).into());
        }
    }
    Ok(())
}
