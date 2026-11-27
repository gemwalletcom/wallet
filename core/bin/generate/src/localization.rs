mod android;
mod ios;

use gem_fluent::Element;

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_LANGUAGE: &str = "en";
pub const ANDROID_ONLY_KEYS: &[&str] = &[
    "application_name",
    "common_add",
    "common_back",
    "common_close",
    "common_grant_permission",
    "rootcheck_body",
    "rootcheck_exit",
    "rootcheck_ignore",
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
        let mut entries = Vec::new();
        let mut keys = BTreeSet::new();
        for message in gem_fluent::parse(&source).map_err(|error| format!("failed to parse {}: {error}", path.display()))? {
            if !keys.insert(message.id.to_string()) {
                return Err(format!("{} contains duplicate key {}", path.display(), message.id).into());
            }
            let value = match message.pattern.as_slice() {
                [Element::Text(value)] => *value,
                [] => return Err(format!("{} contains empty value for {}", path.display(), message.id).into()),
                _ => return Err(format!("{} contains unsupported placeable in {}", path.display(), message.id).into()),
            };
            validate_value(&path, message.id, value)?;
            entries.push((message.id.to_string(), value.trim().to_string()));
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

fn validate_value(path: &Path, id: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    if value.contains('"') || value.contains('&') || value.contains('<') || value.contains('>') {
        return Err(format!("{} contains unsupported native string syntax in {}", path.display(), id).into());
    }
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '\\' && characters.next() != Some('n') {
            return Err(format!("{} contains unsupported escape in {}", path.display(), id).into());
        }
    }
    Ok(())
}
