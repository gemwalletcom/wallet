use crate::remote_mappers::{Field, Generator, HEADER, RemoteType, Variant, Wrapper, camel_case, uniffi_swift_case, uniffi_type_name, unwrap};

/// The syntax of one app's test mocks. The app side mocks the app models; the core side
/// mocks the records and enums UniFFI generates for gemstone.
pub(crate) struct MockSyntax {
    header: &'static str,
    core_header: &'static str,
    import: &'static str,
    open: &'static str,
    parameter: &'static str,
    body: &'static str,
    argument: &'static str,
    keywords: &'static str,
    close: &'static str,
    enum_mock: &'static str,
    reference: &'static str,
    to_core: &'static str,
    optional_type: &'static str,
    list_type: &'static str,
    map_type: &'static str,
    none: &'static str,
    empty_list: &'static str,
    empty_map: &'static str,
    app_module: &'static str,
    app_qualifier: &'static str,
    app_case: fn(&str) -> String,
    app_types: &'static [(&'static str, &'static str, &'static str)],
    app_type_imports: &'static [(&'static str, &'static str)],
    core_qualifier: &'static str,
    core_error_suffix: &'static str,
    core_types: &'static [(&'static str, &'static str, &'static str)],
    core_bytes: (&'static str, &'static str),
    core_identifiers: &'static [(&'static str, &'static str)],
    core_code: &'static str,
    core_imports: &'static [&'static str],
    unit_case: [fn(&str) -> String; 2],
    unit_value: [&'static str; 2],
    data_case: fn(&str) -> String,
    data_value: &'static str,
    named_argument: &'static str,
}

pub(crate) const SWIFT_MOCKS: MockSyntax = MockSyntax {
    header: "import BigInt\nimport Foundation\nimport Primitives\n",
    core_header: "import BigInt\nimport Foundation\nimport Gemstone\nimport GemstonePrimitives\nimport Primitives\nimport PrimitivesTestKit\n",
    import: "",
    open: "\npublic extension {type} {\n    static func mock(\n",
    parameter: "        {label}: {type} = {value},\n",
    body: "    ) -> {type} {\n        {type}(\n",
    argument: "            {label}: {name},\n",
    keywords: "as associatedtype break case catch class continue default defer deinit do else enum extension fallthrough false fileprivate for func guard if import in init inout internal is let nil operator private protocol public repeat rethrows return self static struct subscript super switch throw throws true try typealias var where while",
    close: "        )\n    }\n}\n",
    enum_mock: "\npublic extension {type} {\n    static func mock() -> {type} {\n        {value}\n    }\n}\n",
    reference: ".mock()",
    to_core: "Primitives.{type}.mock().toGem()",
    optional_type: "{}?",
    list_type: "[{}]",
    map_type: "[{key}: {value}]",
    none: "nil",
    empty_list: "[]",
    empty_map: "[:]",
    app_module: "Primitives",
    app_qualifier: "",
    app_case: crate::remote_mappers::swift_case,
    app_types: &[
        ("String", "String", "\"\""),
        ("bool", "Bool", "false"),
        ("i8", "Int8", "0"),
        ("i16", "Int16", "0"),
        ("i32", "Int32", "0"),
        ("i64", "Int64", "0"),
        ("u8", "UInt8", "0"),
        ("u16", "UInt16", "0"),
        ("u32", "UInt32", "0"),
        ("u64", "UInt64", "0"),
        ("f32", "Float", "0"),
        ("f64", "Double", "0"),
        ("BigInt", "String", "\"0\""),
        ("BigUint", "String", "\"0\""),
        ("DateTime<Utc>", "Date", "Date(timeIntervalSince1970: 0)"),
        ("UInt64", "UInt64", "0"),
        ("serde_json::Value", "AnyCodableValue", ""),
        ("BigIntValue", "BigIntValue", "0"),
    ],
    app_type_imports: &[],
    core_qualifier: "Gemstone.",
    core_error_suffix: "Error",
    core_types: &[
        ("String", "String", "\"\""),
        ("bool", "Bool", "false"),
        ("i8", "Int8", "0"),
        ("i16", "Int16", "0"),
        ("i32", "Int32", "0"),
        ("i64", "Int64", "0"),
        ("u8", "UInt8", "0"),
        ("u16", "UInt16", "0"),
        ("u32", "UInt32", "0"),
        ("u64", "UInt64", "0"),
        ("f32", "Float", "0"),
        ("f64", "Double", "0"),
        ("GemBigInt", "BigInt", "0"),
        ("GemBigUint", "BigUInt", "0"),
        ("GemJsonValue", "String", "\"\""),
        ("DateTimeUtc", "Date", "Date(timeIntervalSince1970: 0)"),
        ("DateTime<Utc>", "Date", "Date(timeIntervalSince1970: 0)"),
        ("chrono::DateTime<chrono::Utc>", "Date", "Date(timeIntervalSince1970: 0)"),
        ("primitives::UInt64", "UInt64", "0"),
        ("NaiveDateTime", "Int64", "0"),
        ("NaiveDateTimeUtc", "Int64", "0"),
    ],
    core_bytes: ("Data", "Data()"),
    core_identifiers: &[
        ("AssetId", "Primitives.AssetId.mock().identifier"),
        ("TransactionId", "Primitives.TransactionId.mock().identifier"),
        ("WalletId", "Primitives.WalletId.mock().id"),
    ],
    core_code: "Primitives.{type}.{case}.rawValue",
    core_imports: &[],
    unit_case: [uniffi_swift_case, uniffi_swift_case],
    unit_value: [".{case}", ".{case}"],
    data_case: uniffi_swift_case,
    data_value: ".{case}({arguments})",
    named_argument: "{label}: {value}",
};

pub(crate) const KOTLIN_MOCKS: MockSyntax = MockSyntax {
    header: "package com.gemwallet.android.testkit\n",
    core_header: "",
    import: "import {}\n",
    open: "\nfun mock{function}(\n",
    parameter: "    {name}: {type} = {value},\n",
    body: ") = {type}(\n",
    argument: "    {name} = {name},\n",
    keywords: "as break class continue do else false for fun if in interface is null object package return super this throw true try typealias typeof val var when while",
    close: ")\n",
    enum_mock: "\nfun mock{function}(): {type} = {value}\n",
    reference: "mock{function}()",
    to_core: "mock{function}().toGem()",
    optional_type: "{}?",
    list_type: "List<{}>",
    map_type: "Map<{key}, {value}>",
    none: "null",
    empty_list: "emptyList()",
    empty_map: "emptyMap()",
    app_module: "com.wallet.core.primitives",
    app_qualifier: "",
    app_case: str::to_string,
    app_types: &[
        ("String", "String", "\"\""),
        ("bool", "Boolean", "false"),
        ("i8", "Byte", "0"),
        ("i16", "Short", "0"),
        ("i32", "Int", "0"),
        ("i64", "Long", "0"),
        ("u8", "UByte", "0u"),
        ("u16", "UShort", "0u"),
        ("u32", "UInt", "0u"),
        ("u64", "ULong", "0u"),
        ("f32", "Float", "0f"),
        ("f64", "Double", "0.0"),
        ("BigInt", "String", "\"0\""),
        ("BigUint", "String", "\"0\""),
        ("DateTime<Utc>", "SerializedDate", "0L"),
        ("UInt64", "Long", "0"),
        ("serde_json::Value", "JsonValue", ""),
        ("BigIntValue", "SerializedBigInteger", "java.math.BigInteger.ZERO"),
    ],
    app_type_imports: &[("DateTime<Utc>", "SerializedDate"), ("serde_json::Value", "JsonValue"), ("BigIntValue", "SerializedBigInteger")],
    core_qualifier: "uniffi.gemstone.",
    core_error_suffix: "Exception",
    core_types: &[
        ("String", "String", "\"\""),
        ("bool", "Boolean", "false"),
        ("i8", "Byte", "0"),
        ("i16", "Short", "0"),
        ("i32", "Int", "0"),
        ("i64", "Long", "0"),
        ("u8", "UByte", "0u"),
        ("u16", "UShort", "0u"),
        ("u32", "UInt", "0u"),
        ("u64", "ULong", "0u"),
        ("f32", "Float", "0f"),
        ("f64", "Double", "0.0"),
        ("GemBigInt", "java.math.BigInteger", "java.math.BigInteger.ZERO"),
        ("GemBigUint", "java.math.BigInteger", "java.math.BigInteger.ZERO"),
        ("GemJsonValue", "String", "\"\""),
        ("DateTimeUtc", "Long", "0L"),
        ("DateTime<Utc>", "Long", "0L"),
        ("chrono::DateTime<chrono::Utc>", "Long", "0L"),
        ("primitives::UInt64", "ULong", "0u"),
        ("NaiveDateTime", "Long", "0L"),
        ("NaiveDateTimeUtc", "Long", "0L"),
    ],
    core_bytes: ("ByteArray", "byteArrayOf()"),
    core_identifiers: &[("AssetId", "mockAssetId().toIdentifier()"), ("TransactionId", "mockTransactionId().toIdentifier()"), ("WalletId", "mockWalletId().id")],
    core_code: "com.wallet.core.primitives.{type}.{case}.string",
    core_imports: &["com.gemwallet.android.ext.toGem", "com.gemwallet.android.ext.toIdentifier"],
    unit_case: [screaming_words, uniffi_type_name_case],
    unit_value: ["{type}.{case}", "{type}.{case}"],
    data_case: uniffi_type_name_case,
    data_value: "{type}.{case}({arguments})",
    named_argument: "{label} = {value}",
};

fn uniffi_type_name_case(variant: &str) -> String {
    uniffi_type_name(variant)
}

/// UniFFI writes a Kotlin enum entry in `SHOUTY_SNAKE_CASE`, splitting a run of capitals as one word.
pub(crate) fn screaming_words(variant: &str) -> String {
    let camel = uniffi_type_name(variant);
    let mut out = String::new();
    for (index, character) in camel.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            out.push('_');
        }
        out.push(character.to_ascii_uppercase());
    }
    out
}

/// Gemstone's custom types, which gemstone also declares as aliases of the primitives types they wrap.
const CORE_CUSTOM_TYPES: &[&str] = &["GemBigInt", "GemBigUint", "GemJsonValue", "DateTimeUtc", "NaiveDateTimeUtc"];

/// Which type system a mock is written for.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    App,
    Core,
}

impl Generator {
    pub fn swift_mocks(&self) -> String {
        self.mocks(&SWIFT_MOCKS, &[Side::App])
    }

    pub fn swift_core_mocks(&self) -> String {
        self.mocks(&SWIFT_MOCKS, &[Side::Core])
    }

    pub fn kotlin_mocks(&self) -> String {
        self.mocks(&KOTLIN_MOCKS, &[Side::App, Side::Core])
    }

    /// One `mock(...)` per type listed under `mocks:`, taking every field with the default the
    /// rules give its type: the app model where primitives declares one, otherwise the
    /// record or enum UniFFI generates for gemstone.
    fn mocks(&self, syntax: &MockSyntax, sides: &[Side]) -> String {
        let mut body = String::new();
        let mut imports = Vec::new();
        if sides.contains(&Side::App) {
            for mock in &self.mocked {
                let RemoteType::Record { name, fields, .. } = mock else { continue };
                imports.push(self.app_import(syntax, name));
                let fields = fields.iter().filter(|field| !field.skipped).collect::<Vec<_>>();
                let parameters = fields
                    .iter()
                    .map(|field| {
                        imports.extend(self.app_imports(syntax, &field.type_name));
                        let value = match self.config.mock_override(name, &field.rust) {
                            Some(value) => self.app_literal(syntax, name, field, value),
                            None => self.app_default(syntax, name, field),
                        };
                        (field.serialized.clone(), self.app_type(syntax, &field.type_name), value)
                    })
                    .collect::<Vec<_>>();
                body.push_str(&record_mock(syntax, name, &uniffi_type_name(name), &parameters));
            }
        }
        if sides.contains(&Side::Core) {
            for name in self.core_mocked() {
                let core_type = format!("{}{}", syntax.core_qualifier, uniffi_type_name(name));
                match self.core_declaration(name) {
                    Some(RemoteType::Record { fields, .. }) => {
                        let parameters = fields
                            .iter()
                            .filter(|field| !field.skipped)
                            .map(|field| (core_label(field), self.core_type(syntax, &field.type_name), self.core_value(syntax, name, field, &mut imports)))
                            .collect::<Vec<_>>();
                        body.push_str(&record_mock(syntax, &core_type, &uniffi_type_name(name), &parameters));
                    }
                    Some(RemoteType::Enum { variants, .. }) => {
                        let value = self.core_enum_value(syntax, name, variants, &mut imports);
                        body.push_str(&syntax.enum_mock.replace("{type}", &core_type).replace("{function}", &uniffi_type_name(name)).replace("{value}", &value));
                    }
                    _ => panic!("{name} is listed under mocks: but gemstone exports no record or enum by that name"),
                }
            }
        }
        let header = match sides {
            [Side::Core] => syntax.core_header,
            _ => syntax.header,
        };
        let mut out = format!("{HEADER}\n{header}");
        if !syntax.import.is_empty() {
            imports.retain(|import| !syntax.core_imports.contains(&import.as_str()) || import.rsplit('.').next().is_some_and(|function| body.contains(&format!(".{function}("))));
            imports.sort_unstable();
            imports.dedup();
            out.push('\n');
            for import in imports {
                out.push_str(&syntax.import.replace("{}", &import));
            }
        }
        out.push_str(&body);
        out
    }

    /// The names under `mocks:` that primitives does not declare as an app model struct, so the
    /// mock is written for the UniFFI type.
    fn core_mocked(&self) -> Vec<&str> {
        let mut names = self.config.mocked().into_iter().filter(|name| !self.mocked.iter().any(|mock| mock.name() == *name)).collect::<Vec<_>>();
        names.sort_unstable();
        names
    }

    fn core_declaration(&self, name: &str) -> Option<&RemoteType> {
        self.core_types.get(name).or_else(|| self.types.iter().find(|remote| remote.name() == name && !matches!(remote, RemoteType::Code { .. })))
    }

    fn app_import(&self, syntax: &MockSyntax, name: &str) -> String {
        match self.app_types.get(name).map(|app| app.module.as_str()).unwrap_or_default() {
            "" => format!("{}.{name}", syntax.app_module),
            module => format!("{}.{module}.{name}", syntax.app_module),
        }
    }

    /// The app types a field type names, which a Kotlin mock file imports.
    fn app_imports(&self, syntax: &MockSyntax, type_name: &str) -> Vec<String> {
        match unwrap(type_name) {
            (inner, Wrapper::Option | Wrapper::Vec) => self.app_imports(syntax, inner),
            (name, Wrapper::None) if self.app_types.contains_key(name) || self.config.identifiers.iter().any(|identifier| identifier == name) => vec![self.app_import(syntax, name)],
            (name, Wrapper::None) => syntax.app_type_imports.iter().filter(|(rust, _)| *rust == name).map(|(_, app)| format!("{}.{app}", syntax.app_module)).collect(),
        }
    }

    fn app_type(&self, syntax: &MockSyntax, type_name: &str) -> String {
        match unwrap(type_name) {
            (inner, Wrapper::Option) => syntax.optional_type.replace("{}", &self.app_type(syntax, inner)),
            (inner, Wrapper::Vec) => syntax.list_type.replace("{}", &self.app_type(syntax, inner)),
            (name, Wrapper::None) => syntax.app_types.iter().find(|(rust, ..)| *rust == name).map_or(format!("{}{name}", syntax.app_qualifier), |(_, app, _)| app.to_string()),
        }
    }

    fn app_default(&self, syntax: &MockSyntax, record: &str, field: &Field) -> String {
        let name = match unwrap(&field.type_name) {
            (_, Wrapper::Option) => return syntax.none.to_string(),
            (_, Wrapper::Vec) => return syntax.empty_list.to_string(),
            (name, Wrapper::None) => name,
        };
        if let Some((_, _, zero)) = syntax.app_types.iter().find(|(rust, _, zero)| *rust == name && !zero.is_empty()) {
            return zero.to_string();
        }
        if self.config.identifiers.iter().any(|identifier| identifier == name) || self.mocked.iter().any(|mock| mock.name() == name) {
            return syntax.reference.replace("{function}", &uniffi_type_name(name));
        }
        match self.app_types.get(name).and_then(|app| app.variants.as_ref()).and_then(|variants| variants.first()) {
            Some(variant) if variant.fields.is_empty() => syntax.unit_value[0].replace("{type}", name).replace("{case}", &(syntax.app_case)(&variant.name)),
            Some(variant) => panic!("{record}.{}: the first variant of {name}, {}, carries data; add an override under mocks:", field.rust, variant.name),
            None => panic!("{record}.{}: {name} has no mock and no default rule; list {name} under mocks: or add an override", field.rust),
        }
    }

    /// An override from `mocks:`, written as the field's type spells it: a string is quoted, an
    /// enum names its variant, a number or a flag stays as written.
    fn app_literal(&self, syntax: &MockSyntax, record: &str, field: &Field, value: &str) -> String {
        let name = match unwrap(&field.type_name) {
            (inner, Wrapper::Option) => inner,
            (_, Wrapper::Vec) => panic!("{record}.{} is a list; a mock override takes a string, number, flag or enum variant", field.rust),
            (name, Wrapper::None) => name,
        };
        match self.app_types.get(name).and_then(|app| app.variants.as_ref()) {
            Some(_) => syntax.unit_value[0].replace("{type}", name).replace("{case}", &(syntax.app_case)(value)),
            None if name == "String" => format!("{value:?}"),
            None if self.config.is_scalar(name) => value.to_string(),
            None => panic!("{record}.{}: {name} cannot be overridden; a mock override takes a string, number, flag or enum variant", field.rust),
        }
    }

    /// The name UniFFI gives a field type, after the declared renames of primitives types.
    fn core_name<'a>(&'a self, name: &'a str) -> &'a str {
        if CORE_CUSTOM_TYPES.contains(&name) {
            return name;
        }
        let name = self.config.declared(name).unwrap_or(name);
        let name = match name.contains('<') || name.starts_with("primitives::UInt64") {
            true => name,
            false => name.rsplit("::").next().unwrap_or(name),
        };
        if self.core_types.contains_key(name) {
            return name;
        }
        if let Some((alias, _)) = self.core_aliases.iter().find(|(alias, aliased)| aliased.rsplit("::").next() == Some(name) && self.core_types.contains_key(alias.as_str())) {
            return alias;
        }
        match self.core_aliases.get(name) {
            Some(aliased) if aliased.rsplit("::").next() != Some(name) => self.core_name(aliased),
            _ => name,
        }
    }

    fn core_type(&self, syntax: &MockSyntax, type_name: &str) -> String {
        if let Some((key, value)) = map_types(type_name) {
            return syntax.map_type.replace("{key}", &self.core_type(syntax, key)).replace("{value}", &self.core_type(syntax, value));
        }
        match unwrap(type_name) {
            ("u8", Wrapper::Vec) => syntax.core_bytes.0.to_string(),
            (inner, Wrapper::Option) => syntax.optional_type.replace("{}", &self.core_type(syntax, inner)),
            (inner, Wrapper::Vec) => syntax.list_type.replace("{}", &self.core_type(syntax, inner)),
            (name, Wrapper::None) => {
                let name = self.core_name(name);
                if let Some((_, core, _)) = syntax.core_types.iter().find(|(rust, ..)| *rust == name) {
                    return core.to_string();
                }
                if self.config.identifiers.iter().chain(&self.config.codes).any(|identifier| identifier == name) {
                    return "String".to_string();
                }
                match name.strip_suffix("Error").filter(|_| !self.core_types.contains_key(name) || self.core_errors.contains(name)) {
                    Some(stem) => format!("{}{}{}", syntax.core_qualifier, uniffi_type_name(stem), syntax.core_error_suffix),
                    None => format!("{}{}", syntax.core_qualifier, uniffi_type_name(name)),
                }
            }
        }
    }

    fn core_value(&self, syntax: &MockSyntax, record: &str, field: &Field, imports: &mut Vec<String>) -> String {
        match self.config.mock_override(record, &field.rust) {
            Some(value) => self.core_literal(syntax, record, field, value),
            None => self.core_default(syntax, record, field, imports),
        }
    }

    /// An override from `mocks:` for a gemstone field, spelled as in `app_literal`.
    fn core_literal(&self, syntax: &MockSyntax, record: &str, field: &Field, value: &str) -> String {
        let name = match unwrap(&field.type_name) {
            (inner, Wrapper::Option) => self.core_name(inner),
            (_, Wrapper::Vec) => panic!("{record}.{} is a list; a mock override takes a string, number, flag or enum variant", field.rust),
            (name, Wrapper::None) => self.core_name(name),
        };
        match self.core_variants(name) {
            Some(variants) if variants.iter().all(|variant| variant.fields.is_empty()) => syntax.unit_value[0]
                .replace("{type}", &format!("{}{}", syntax.core_qualifier, uniffi_type_name(name)))
                .replace("{case}", &(syntax.unit_case[0])(value)),
            None if name == "String" => format!("{value:?}"),
            None if self.config.is_scalar(name) => value.to_string(),
            _ => panic!("{record}.{}: {name} cannot be overridden; a mock override takes a string, number, flag or enum variant", field.rust),
        }
    }

    fn core_default(&self, syntax: &MockSyntax, record: &str, field: &Field, imports: &mut Vec<String>) -> String {
        if map_types(&field.type_name).is_some() {
            return syntax.empty_map.to_string();
        }
        let name = match unwrap(&field.type_name) {
            ("u8", Wrapper::Vec) => return syntax.core_bytes.1.to_string(),
            (_, Wrapper::Option) => return syntax.none.to_string(),
            (_, Wrapper::Vec) => return syntax.empty_list.to_string(),
            (name, Wrapper::None) => self.core_name(name),
        };
        let label = match field.rust.is_empty() {
            true => format!("{record}.{}", name),
            false => format!("{record}.{}", field.rust),
        };
        if let Some((_, _, zero)) = syntax.core_types.iter().find(|(rust, _, zero)| *rust == name && !zero.is_empty()) {
            return zero.to_string();
        }
        if let Some((_, identifier)) = syntax.core_identifiers.iter().find(|(identifier, _)| *identifier == name) {
            imports.extend(syntax.core_imports.iter().map(|import| import.to_string()));
            return identifier.to_string();
        }
        if self.config.codes.iter().any(|code| code == name) {
            let variant = self
                .app_types
                .get(name)
                .and_then(|app| app.variants.as_ref())
                .and_then(|variants| variants.first())
                .unwrap_or_else(|| panic!("{label}: {name} has no variants"));
            return syntax.core_code.replace("{type}", name).replace("{case}", &(syntax.app_case)(&variant.name));
        }
        let core_type = format!("{}{}", syntax.core_qualifier, uniffi_type_name(name));
        if self.core_mocked().contains(&name) {
            return syntax.reference.replace("{function}", &uniffi_type_name(name));
        }
        if self.mocked.iter().any(|mock| mock.name() == name) && self.types.iter().any(|remote| remote.name() == name && remote.app_model()) {
            imports.extend(syntax.core_imports.iter().map(|import| import.to_string()));
            return syntax.to_core.replace("{type}", name).replace("{function}", &uniffi_type_name(name));
        }
        match self.core_variants(name).and_then(|variants| variants.first().map(|first| (first, variants))) {
            Some((variant, variants)) if variant.fields.is_empty() => {
                let sealed = usize::from(variants.iter().any(|variant| !variant.fields.is_empty()));
                syntax.unit_value[sealed].replace("{type}", &core_type).replace("{case}", &(syntax.unit_case[sealed])(&variant.name))
            }
            Some((variant, _)) => panic!("{label}: the first variant of {name}, {}, carries data; list {name} under mocks:", variant.name),
            None => panic!("{label}: {name} has no mock and no default rule; list {name} under mocks: or add an override"),
        }
    }

    fn core_variants(&self, name: &str) -> Option<&Vec<Variant>> {
        match self.core_declaration(name) {
            Some(RemoteType::Enum { variants, .. }) => Some(variants),
            _ => None,
        }
    }

    /// A listed enum's mock is its first variant, carrying the default of each of its fields.
    fn core_enum_value(&self, syntax: &MockSyntax, name: &str, variants: &[Variant], imports: &mut Vec<String>) -> String {
        let core_type = format!("{}{}", syntax.core_qualifier, uniffi_type_name(name));
        let variant = variants.first().unwrap_or_else(|| panic!("{name} has no variants"));
        let sealed = usize::from(variants.iter().any(|variant| !variant.fields.is_empty()));
        if variant.fields.is_empty() {
            return syntax.unit_value[sealed].replace("{type}", &core_type).replace("{case}", &(syntax.unit_case[sealed])(&variant.name));
        }
        let arguments = variant
            .fields
            .iter()
            .map(|field| {
                let value = self.core_value(syntax, name, field, imports);
                match field.rust.is_empty() {
                    true => value,
                    false => syntax.named_argument.replace("{label}", &core_label(field)).replace("{value}", &value),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        syntax.data_value.replace("{type}", &core_type).replace("{case}", &(syntax.data_case)(&variant.name)).replace("{arguments}", &arguments)
    }
}

fn record_mock(syntax: &MockSyntax, type_name: &str, function: &str, parameters: &[(String, String, String)]) -> String {
    let mut out = syntax.open.replace("{type}", type_name).replace("{function}", function);
    for (label, parameter_type, value) in parameters {
        out.push_str(
            &syntax
                .parameter
                .replace("{label}", label)
                .replace("{name}", &identifier(syntax, label))
                .replace("{type}", parameter_type)
                .replace("{value}", value),
        );
    }
    out.push_str(&syntax.body.replace("{type}", type_name));
    for (label, ..) in parameters {
        out.push_str(&syntax.argument.replace("{label}", label).replace("{name}", &identifier(syntax, label)));
    }
    out.push_str(syntax.close);
    out
}

/// A label that is a keyword of the app's language, quoted so it reads as a name.
fn identifier(syntax: &MockSyntax, label: &str) -> String {
    match syntax.keywords.split(' ').any(|keyword| keyword == label) {
        true => format!("`{label}`"),
        false => label.to_string(),
    }
}

/// UniFFI names a field in lower camel case and drops the raw-identifier prefix.
fn core_label(field: &Field) -> String {
    camel_case(field.rust.trim_start_matches("r#"))
}

fn map_types(type_name: &str) -> Option<(&str, &str)> {
    let inner = type_name.strip_prefix("HashMap<").or_else(|| type_name.strip_prefix("BTreeMap<"))?.strip_suffix('>')?;
    let (key, value) = inner.split_once(", ")?;
    Some((key.trim(), value.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote_mappers::Config;
    use std::fs;
    use std::path::Path;

    fn expect_generated(name: &str, actual: String) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata").join("expected").join(name);
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            fs::write(&path, &actual).unwrap();
            return;
        }
        let expected = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is missing; run the tests once with UPDATE_GOLDEN=1", path.display()));
        assert_eq!(actual, expected, "{name} no longer matches testdata/expected/{name}; rerun with UPDATE_GOLDEN=1 once the diff is intended");
    }

    fn generator(mocks: &str) -> Generator {
        let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let yaml = include_str!("../testdata/remote_types.yml").replace("mocks:\n", &format!("mocks:\n{mocks}"));
        Generator::parse(Config::from_yaml(&yaml), &testdata.join("primitives"), &testdata.join("gemstone"))
    }

    #[test]
    fn test_swift_mocks_match_the_expected_file() {
        expect_generated("GeneratedMocks.swift", Generator::mock().swift_mocks());
    }

    #[test]
    fn test_swift_core_mocks_match_the_expected_file() {
        expect_generated("GeneratedCoreMocks.swift", Generator::mock().swift_core_mocks());
    }

    #[test]
    fn test_kotlin_mocks_match_the_expected_file() {
        expect_generated("GeneratedMocks.kt", Generator::mock().kotlin_mocks());
    }

    #[test]
    #[should_panic(expected = "Wallet.accounts is a list")]
    fn test_a_mock_override_on_a_list_names_the_field() {
        generator("  - Wallet:\n      accounts: none\n").kotlin_mocks();
    }

    #[test]
    #[should_panic(expected = "Delegation.validator: DelegationValidator has no mock and no default rule")]
    fn test_a_field_without_a_mock_or_a_rule_names_the_field() {
        generator("  - Delegation\n").swift_mocks();
    }

    #[test]
    #[should_panic(expected = "GemPayment.action: the first variant of GemPaymentAction, Send, carries data")]
    fn test_a_core_field_whose_enum_starts_with_data_names_the_field() {
        generator("  - GemPayment\n").swift_core_mocks();
    }

    #[test]
    fn test_kotlin_enum_entries_split_a_run_of_capitals_as_one_word() {
        assert_eq!(screaming_words("NotReachable"), "NOT_REACHABLE");
        assert_eq!(screaming_words("TransferNFT"), "TRANSFER_NFT");
        assert_eq!(screaming_words("ERC20"), "ERC20");
    }
}
