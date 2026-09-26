use config::{File, FileFormat};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{self, AppField, AppRecord, TypeMappings};
use crate::remote_mappers::HEADER;
use primitives::Platform;

const SWIFT_SOURCES: &str = "Store/Sources";
const SWIFT_PATH: &str = "Store/Sources/Generated/RecordMappers.swift";
/// The store module's sources, from the Android models directory the generator is given.
const KOTLIN_SOURCES: &str = "../../../../../../../data/services/store/src/main/kotlin";
const KOTLIN_PATH: &str = "com/gemwallet/android/data/services/store/generated/EntityMappers.kt";
const KOTLIN_PACKAGE: &str = "com.gemwallet.android.data.services.store.database.entities";

/// The `records:` section of `remote_types.yml`.
#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    records: BTreeMap<String, Spec>,
    #[serde(default)]
    record_aliases: BTreeMap<String, BTreeMap<String, String>>,
}

/// A `records:` entry, keyed by the model: the record each app stores it in, and the record
/// fields the caller passes because the model does not carry them (a parent key).
#[derive(Deserialize)]
struct Spec {
    #[serde(default)]
    keys: Vec<String>,
    ios: AppSpec,
    android: AppSpec,
}

/// One app's record for a model: its name, or its name with the fields that do not copy by name.
#[derive(Deserialize)]
#[serde(untagged)]
enum AppSpec {
    Named(String),
    Detailed {
        record: String,
        /// Record field → the model path it stores, for a renamed or flattened field.
        #[serde(default)]
        fields: BTreeMap<String, String>,
        /// Model path → the value the model takes where the record stores none, or stores it optional.
        #[serde(default)]
        defaults: BTreeMap<String, String>,
    },
}

impl AppSpec {
    fn record(&self) -> &str {
        match self {
            Self::Named(record) | Self::Detailed { record, .. } => record,
        }
    }

    fn field(&self, name: &str) -> Option<&str> {
        match self {
            Self::Named(_) => None,
            Self::Detailed { fields, .. } => fields.get(name).map(String::as_str),
        }
    }

    fn default(&self, path: &str) -> Option<&str> {
        match self {
            Self::Named(_) => None,
            Self::Detailed { defaults, .. } => defaults.get(path).map(String::as_str),
        }
    }
}

impl Config {
    fn from_yaml(yaml: &str) -> Self {
        config::Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .and_then(|config| config.try_deserialize())
            .expect("the records section of remote_types.yml is malformed")
    }
}

/// A stored field of a record declaration, in declaration order.
struct Declared {
    name: String,
    type_name: String,
    has_default: bool,
}

#[derive(Clone, Copy)]
enum Language {
    Swift,
    Kotlin,
}

impl Language {
    fn key(self) -> &'static str {
        match self {
            Self::Swift => "swift",
            Self::Kotlin => "kotlin",
        }
    }

    fn app_type(self, field: &AppField) -> &str {
        match self {
            Self::Swift => &field.swift,
            Self::Kotlin => &field.kotlin,
        }
    }

    fn model_name(self, name: &str, model: &AppRecord) -> String {
        match self {
            Self::Swift => format!("Primitives.{name}"),
            Self::Kotlin => format!("{}.{name}", model.kotlin_package),
        }
    }

    fn reference(self, name: &str) -> String {
        match self {
            Self::Swift => models::swift_identifier(name),
            Self::Kotlin => name.to_string(),
        }
    }

    fn argument(self, label: &str, value: &str) -> String {
        match self {
            Self::Swift => format!("{label}: {value}"),
            Self::Kotlin => format!("{label} = {value}"),
        }
    }

    fn fallback(self, value: &str, default: &str) -> String {
        match self {
            Self::Swift => format!("{value} ?? {default}"),
            Self::Kotlin => format!("{value} ?: {default}"),
        }
    }

    fn empty_list(self) -> &'static str {
        match self {
            Self::Swift => "[]",
            Self::Kotlin => "emptyList()",
        }
    }

    fn is_list(self, type_name: &str) -> bool {
        match self {
            Self::Swift => type_name.starts_with('['),
            Self::Kotlin => type_name.starts_with("List<"),
        }
    }

    fn is_integer(self, type_name: &str) -> bool {
        let integers: &[&str] = match self {
            Self::Swift => &["Int", "Int8", "Int16", "Int32", "Int64", "UInt", "UInt8", "UInt16", "UInt32", "UInt64"],
            Self::Kotlin => &["Int", "Long", "Short", "Byte", "UInt", "ULong", "UShort", "UByte"],
        };
        integers.contains(&type_name)
    }

    fn cast(self, value: &str, to: &str, optional: bool) -> String {
        match (self, optional) {
            (Self::Swift, false) => format!("{to}({value})"),
            (Self::Swift, true) => format!("{value}.map {{ {to}($0) }}"),
            (Self::Kotlin, false) => format!("{value}.to{to}()"),
            (Self::Kotlin, true) => format!("{value}?.to{to}()"),
        }
    }
}

/// The record mappers of one app: the models `records:` names, the record declarations the app
/// writes by hand, and the model declarations they copy.
pub struct Generator {
    config: Config,
    models: BTreeMap<String, AppRecord>,
    language: Language,
    declarations: BTreeMap<String, Vec<Declared>>,
}

impl Generator {
    pub fn load(root: &Path, platform: Platform, platform_directory: &Path) -> Self {
        let (language, sources) = match platform {
            Platform::IOS => (Language::Swift, platform_directory.join(SWIFT_SOURCES)),
            Platform::Android => (Language::Kotlin, platform_directory.join(KOTLIN_SOURCES)),
        };
        Self::parse(crate::remote_mappers::CONFIG, &root.join(crate::remote_mappers::PRIMITIVES_SOURCE), language, &sources)
    }

    fn parse(yaml: &str, primitives: &Path, language: Language, sources: &Path) -> Self {
        Self {
            config: Config::from_yaml(yaml),
            models: models::app_records(primitives, &TypeMappings::from_yaml(yaml)),
            language,
            declarations: match language {
                Language::Swift => swift_records(sources),
                Language::Kotlin => kotlin_records(sources),
            },
        }
    }

    pub fn path(&self, platform_directory: &Path) -> PathBuf {
        match self.language {
            Language::Swift => platform_directory.join(SWIFT_PATH),
            Language::Kotlin => platform_directory.join(KOTLIN_SOURCES).join(KOTLIN_PATH),
        }
    }

    pub fn render(&self) -> String {
        let body = self
            .config
            .records
            .iter()
            .map(|(model, spec)| {
                let app = match self.language {
                    Language::Swift => &spec.ios,
                    Language::Kotlin => &spec.android,
                };
                let record = app.record();
                let declared = self.declarations.get(record).unwrap_or_else(|| {
                    panic!(
                        "{model} is stored in {record}, which the {} store does not declare as a struct or data class with a memberwise constructor",
                        self.language.key()
                    )
                });
                Mapping {
                    language: self.language,
                    model,
                    record,
                    spec,
                    app,
                    declared,
                    generator: self,
                }
                .render()
            })
            .collect::<String>();
        match self.language {
            Language::Swift => format!("{HEADER}\nimport Foundation\nimport Primitives\n{body}"),
            Language::Kotlin => format!("{HEADER}\npackage {KOTLIN_PACKAGE}\n{body}"),
        }
    }

    fn model(&self, name: &str) -> &AppRecord {
        self.models.get(name).unwrap_or_else(|| panic!("{name} is not a struct app model"))
    }
}

/// One model ↔ record pair in one app language.
struct Mapping<'a> {
    language: Language,
    model: &'a str,
    record: &'a str,
    spec: &'a Spec,
    app: &'a AppSpec,
    declared: &'a [Declared],
    generator: &'a Generator,
}

impl Mapping<'_> {
    fn render(&self) -> String {
        let model = self.generator.model(self.model);
        let model_name = self.language.model_name(self.model, model);
        let keys = self
            .spec
            .keys
            .iter()
            .map(|key| {
                let field = self.declared.iter().find(|field| &field.name == key).unwrap_or_else(|| panic!("{}: key {key} is not a field of {}", self.model, self.record));
                format!("{key}: {}", field.type_name)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let to_record = self.record_arguments();
        let to_model = self.construct(self.model, "", false);
        match self.language {
            Language::Swift => format!(
                "\nextension {model_name} {{\n    func toRecord({keys}) -> {record} {{\n        {record}(\n{to_record}        )\n    }}\n}}\n\nextension {record} {{\n    func to{name}() -> {model_name} {{\n        {to_model}\n    }}\n}}\n",
                record = self.record,
                name = self.model,
                to_record = to_record.iter().map(|argument| format!("            {argument},\n")).collect::<String>(),
            ),
            Language::Kotlin => format!(
                "\nfun {model_name}.toRecord({keys}): {record} = {record}(\n{to_record})\n\nfun {record}.to{name}(): {model_name} = {to_model}\n",
                record = self.record,
                name = self.model,
                to_record = to_record.iter().map(|argument| format!("    {argument},\n")).collect::<String>(),
            ),
        }
    }

    /// The model path a record field stores: a renamed or flattened one from the spec, else the
    /// model field of the same name.
    fn bound_path(&self, field: &Declared) -> Option<String> {
        if self.spec.keys.contains(&field.name) {
            return None;
        }
        match self.app.field(&field.name) {
            Some(path) => Some(path.to_string()),
            None => self.generator.model(self.model).fields.iter().any(|model_field| model_field.name == field.name).then(|| field.name.clone()),
        }
    }

    fn record_arguments(&self) -> Vec<String> {
        self.declared
            .iter()
            .filter_map(|field| {
                if self.spec.keys.contains(&field.name) {
                    return Some(self.language.argument(&field.name, &self.language.reference(&field.name)));
                }
                let Some(path) = self.bound_path(field) else {
                    assert!(
                        field.has_default,
                        "{}: {}.{} is not a field of the model, a key or a field with a default; name it under keys or fields",
                        self.model, self.record, field.name
                    );
                    return None;
                };
                let (value, model_type) = self.model_value(&path);
                Some(self.language.argument(&field.name, &self.stored(&value, &model_type, field, &path)))
            })
            .collect()
    }

    /// The expression that reads a model path, and its type, optional when a step on the way is.
    fn model_value(&self, path: &str) -> (String, String) {
        let mut holder = self.model.to_string();
        let mut value = String::new();
        let mut chained = false;
        let mut leaf = None;
        for (index, segment) in path.split('.').enumerate() {
            assert!(leaf.is_none(), "{}: {path} goes through a field that holds no model", self.model);
            let model = self.generator.model(&holder);
            let field = model
                .fields
                .iter()
                .find(|field| field.name == segment)
                .unwrap_or_else(|| panic!("{}: {holder} has no field {segment} on the path {path}", self.model));
            let separator = match (index, chained) {
                (0, _) => "",
                (_, true) => "?.",
                (_, false) => ".",
            };
            value = format!("{value}{separator}{}", self.language.reference(segment));
            let app_type = self.language.app_type(field).to_string();
            match &field.holds {
                Some(held) if self.generator.models.contains_key(held) && path.split('.').count() > index + 1 => {
                    chained |= app_type.ends_with('?');
                    holder = held.clone();
                }
                _ => leaf = Some(app_type),
            }
        }
        let leaf = leaf.unwrap_or_else(|| panic!("{}: {path} ends on a model, not a field", self.model));
        match chained && !leaf.ends_with('?') {
            true => (value, format!("{leaf}?")),
            false => (value, leaf),
        }
    }

    fn normalized(&self, type_name: &str) -> String {
        let type_name = type_name.replace(' ', "");
        let (base, optional) = match type_name.strip_suffix('?') {
            Some(base) => (base.to_string(), "?"),
            None => (type_name.clone(), ""),
        };
        let aliased = self.generator.config.record_aliases.get(self.language.key()).and_then(|aliases| aliases.get(&base)).cloned().unwrap_or(base);
        format!("{aliased}{optional}")
    }

    /// A model value as the record field stores it.
    fn stored(&self, value: &str, model_type: &str, field: &Declared, path: &str) -> String {
        let (from, to) = (self.normalized(model_type), self.normalized(&field.type_name));
        if from == to || to == format!("{from}?") {
            return value.to_string();
        }
        self.integer_cast(value, &from, &to)
            .unwrap_or_else(|| panic!("{}: {path} is {from} and {}.{} stores {to}; no rule converts one to the other", self.model, self.record, field.name))
    }

    /// A record field as the model field reads it.
    fn read(&self, field: &Declared, model_type: &str, path: &str, unwrapped: bool) -> String {
        let (from, to) = (self.normalized(&field.type_name), self.normalized(model_type));
        let value = self.language.reference(&field.name);
        let from = match unwrapped {
            true => from.trim_end_matches('?').to_string(),
            false => from,
        };
        if from == to || to == format!("{from}?") {
            return value;
        }
        if from == format!("{to}?") {
            return match self.app.default(path) {
                Some(default) => self.language.fallback(&value, default),
                None if self.language.is_list(&to) => self.language.fallback(&value, self.language.empty_list()),
                None => panic!("{}: {}.{} is optional and {path} is not; give {path} a default", self.model, self.record, field.name),
            };
        }
        self.integer_cast(&value, &from, &to)
            .unwrap_or_else(|| panic!("{}: {}.{} stores {from} and {path} is {to}; no rule converts one to the other", self.model, self.record, field.name))
    }

    /// An integer converted to another integer type, where the target keeps any optionality.
    fn integer_cast(&self, value: &str, from: &str, to: &str) -> Option<String> {
        let (from_base, to_base) = (from.trim_end_matches('?'), to.trim_end_matches('?'));
        let optional = from.ends_with('?');
        (self.language.is_integer(from_base) && self.language.is_integer(to_base) && (to.ends_with('?') || !optional)).then(|| self.language.cast(value, to_base, optional))
    }

    /// The model built from the record fields bound to paths under `prefix`.
    fn construct(&self, name: &str, prefix: &str, unwrapped: bool) -> String {
        let model = self.generator.model(name);
        let arguments = model
            .fields
            .iter()
            .map(|field| {
                let path = format!("{prefix}{}", field.name);
                self.language.argument(&self.language.reference(&field.name), &self.model_field(field, &path, unwrapped))
            })
            .collect::<Vec<_>>();
        let constructor = self.language.model_name(name, model);
        match (prefix.is_empty(), self.language) {
            (true, Language::Swift) => format!("{constructor}(\n{}        )", arguments.iter().map(|argument| format!("            {argument},\n")).collect::<String>()),
            (true, Language::Kotlin) => format!("{constructor}(\n{})", arguments.iter().map(|argument| format!("    {argument},\n")).collect::<String>()),
            (false, _) => format!("{constructor}({})", arguments.join(", ")),
        }
    }

    fn model_field(&self, field: &AppField, path: &str, unwrapped: bool) -> String {
        let model_type = self.language.app_type(field);
        if let Some(record_field) = self.declared.iter().find(|declared| self.bound_path(declared).as_deref() == Some(path)) {
            return self.read(record_field, model_type, path, unwrapped);
        }
        if let Some(default) = self.app.default(path) {
            return default.to_string();
        }
        let nested = format!("{path}.");
        let bound = self.declared.iter().filter(|declared| self.bound_path(declared).is_some_and(|bound| bound.starts_with(&nested))).collect::<Vec<_>>();
        let held = field.holds.as_deref().filter(|held| self.generator.models.contains_key(*held));
        match (held, bound.is_empty(), model_type.ends_with('?')) {
            (Some(held), false, false) => self.construct(held, &nested, unwrapped),
            (Some(held), false, true) => match self.language {
                Language::Kotlin => {
                    let checks = bound.iter().filter(|declared| declared.type_name.ends_with('?')).map(|declared| format!("{} != null", declared.name)).collect::<Vec<_>>();
                    assert!(!checks.is_empty(), "{}: {path} is optional and flattened into fields that are never null", self.model);
                    let checks = checks.join(" && ");
                    format!("if ({checks}) {} else null", self.construct(held, &nested, true))
                }
                Language::Swift => panic!("{}: {path} is optional and flattened, which the Swift mapper does not build", self.model),
            },
            _ => panic!("{}: {} stores nothing for {path}; bind a field to it or give it a default", self.model, self.record),
        }
    }
}

fn source_files(directory: &Path, extension: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files = entries
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            match path.is_dir() {
                true => source_files(&path, extension),
                false if path.extension().is_some_and(|found| found == extension) => vec![path],
                false => Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn braces(line: &str) -> isize {
    line.matches('{').count() as isize - line.matches('}').count() as isize
}

/// Every Swift struct's stored properties, in declaration order, by struct name.
fn swift_records(directory: &Path) -> BTreeMap<String, Vec<Declared>> {
    let mut records = BTreeMap::new();
    for path in source_files(directory, "swift") {
        let Ok(source) = fs::read_to_string(&path) else { continue };
        let mut lines = source.lines();
        while let Some(line) = lines.next() {
            let Some(name) = swift_struct(line.trim()) else { continue };
            let mut depth = braces(line);
            let mut fields = Vec::new();
            let mut explicit_init = false;
            while depth > 0 {
                let Some(line) = lines.next() else { break };
                let trimmed = line.trim();
                if depth == 1 {
                    explicit_init |= trimmed.starts_with("init(") || trimmed.starts_with("public init(");
                    fields.extend(swift_property(trimmed));
                }
                depth += braces(trimmed);
            }
            if !explicit_init {
                records.insert(name, fields);
            }
        }
    }
    records
}

fn swift_struct(line: &str) -> Option<String> {
    let rest = line.strip_prefix("public ").unwrap_or(line).strip_prefix("struct ")?;
    let name = rest.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn swift_property(line: &str) -> Option<Declared> {
    let line = line.split("//").next()?.trim();
    let line = line.strip_prefix("public ").unwrap_or(line);
    let (is_var, rest) = match (line.strip_prefix("var "), line.strip_prefix("let ")) {
        (Some(rest), _) => (true, rest),
        (_, Some(rest)) => (false, rest),
        _ => return None,
    };
    if rest.contains('{') {
        return None;
    }
    let (name, rest) = rest.split_once(':')?;
    let (type_name, default) = match rest.split_once(" = ") {
        Some((type_name, default)) => (type_name.trim(), Some(default)),
        None => (rest.trim(), None),
    };
    Some(Declared {
        name: name.trim().to_string(),
        type_name: type_name.to_string(),
        has_default: default.is_some() || (is_var && type_name.ends_with('?')),
    })
}

/// Every Kotlin data class's constructor properties, in declaration order, by class name.
fn kotlin_records(directory: &Path) -> BTreeMap<String, Vec<Declared>> {
    let mut records = BTreeMap::new();
    for path in source_files(directory, "kt") {
        let Ok(source) = fs::read_to_string(&path) else { continue };
        let source = source.lines().map(|line| line.split(" //").next().unwrap_or_default()).collect::<Vec<_>>().join("\n");
        for (start, _) in source.match_indices("data class ") {
            let rest = &source[start + "data class ".len()..];
            let name = rest.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect::<String>();
            let Some(parameters) = rest[name.len()..].trim_start().strip_prefix('(').map(balanced) else { continue };
            records.insert(name, split_top_level(parameters).iter().filter_map(|parameter| kotlin_property(parameter)).collect());
        }
    }
    records
}

/// The text up to the parenthesis that closes one already open.
fn balanced(text: &str) -> &str {
    let mut depth = 1usize;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 1 => return &text[..index],
            ')' => depth -= 1,
            _ => {}
        }
    }
    text
}

fn split_top_level(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '"' => quoted = !quoted,
            '(' | '<' | '[' if !quoted => depth += 1,
            ')' | '>' | ']' if !quoted => depth = depth.saturating_sub(1),
            ',' if depth == 0 && !quoted => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(character);
    }
    parts.push(current);
    parts
}

fn kotlin_property(parameter: &str) -> Option<Declared> {
    let mut rest = parameter.trim();
    while let Some(annotation) = rest.strip_prefix('@') {
        let name_end = annotation
            .find(|character: char| !(character.is_alphanumeric() || character == '_' || character == '.' || character == ':'))
            .unwrap_or(annotation.len());
        rest = annotation[name_end..].trim_start();
        if let Some(arguments) = rest.strip_prefix('(') {
            rest = arguments[balanced(arguments).len() + 1..].trim_start();
        }
    }
    let rest = rest.strip_prefix("val ").or_else(|| rest.strip_prefix("var "))?;
    let (name, rest) = rest.split_once(':')?;
    let (type_name, default) = match rest.split_once(" = ") {
        Some((type_name, default)) => (type_name.trim(), Some(default)),
        None => (rest.trim(), None),
    };
    Some(Declared {
        name: name.trim().to_string(),
        type_name: type_name.to_string(),
        has_default: default.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::expect_generated;
    use std::path::PathBuf;

    fn testdata() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    fn mock(language: Language, sources: &str) -> Generator {
        let yaml = fs::read_to_string(testdata().join("remote_types.yml")).unwrap();
        Generator::parse(&yaml, &testdata().join("primitives"), language, &testdata().join("records").join(sources))
    }

    #[test]
    fn test_swift_record_mappers_match_the_expected_file() {
        expect_generated("RecordMappers.swift", mock(Language::Swift, "ios").render());
    }

    #[test]
    fn test_kotlin_record_mappers_match_the_expected_file() {
        expect_generated("EntityMappers.kt", mock(Language::Kotlin, "android").render());
    }

    #[test]
    fn test_generated_record_mappers_match_the_committed_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (platform, directory) in [(Platform::IOS, root.join("../ios/Packages")), (Platform::Android, root.join("../android/gemcore/src/main/kotlin/com/wallet/core"))] {
            let generator = Generator::load(&root, platform, &directory);
            let path = generator.path(&directory);
            assert_eq!(fs::read_to_string(&path).unwrap_or_default(), generator.render(), "{} is out of date; run just generate-models", path.display());
        }
    }

    #[test]
    fn test_kotlin_properties_drop_annotations_and_keep_defaults() {
        let property = kotlin_property(r#"@ColumnInfo("is_earn_enabled", defaultValue = "0") val isEarnEnabled: Boolean = false"#).unwrap();
        assert_eq!(property.name, "isEarnEnabled");
        assert_eq!(property.type_name, "Boolean");
        assert!(property.has_default);
        assert_eq!(split_top_level("val a: Map<String, Int>, @ColumnInfo(\"b,c\") val b: String").len(), 2);
    }

    #[test]
    fn test_swift_properties_skip_static_and_computed_members() {
        assert!(swift_property("static let databaseTableName: String = \"accounts\"").is_none());
        assert!(swift_property("var id: String { assetId.identifier }").is_none());
        assert!(swift_property("public var imageUrl: String?").unwrap().has_default);
        assert!(!swift_property("let imageUrl: String?").unwrap().has_default);
    }
}
