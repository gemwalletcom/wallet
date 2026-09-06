use crate::remote_mappers::{HEADER, source_files};
use std::fs;
use std::path::Path;

pub const JSON_BRIDGE_PATH: &str = "gemstone/src/models/json_bridge.rs";

/// Types the JSON bridge carries whose Rust enum has data-carrying variants, so serde tags them.
/// Kotlin infers the concrete subtype for a generic `T.toJson()`, which drops that tag, and Core
/// rejects the payload at runtime. Each one needs an overload whose receiver is the base type.
pub fn tagged_bridge_types(bridge: &Path, primitives: &Path) -> Vec<String> {
    let sources: Vec<String> = source_files(primitives).iter().filter_map(|path| fs::read_to_string(path).ok()).collect();

    let mut tagged: Vec<String> = bridge_types(bridge)
        .into_iter()
        .filter(|name| {
            sources
                .iter()
                .any(|source| enum_variants(source, name).is_some_and(|variants| variants.iter().any(|variant| !variant.chars().all(|c| c.is_alphanumeric() || c == '_'))))
        })
        .collect();
    tagged.sort();
    tagged.dedup();
    tagged
}

/// Every type the JSON bridge carries as a serialized string, in declaration order.
pub fn bridge_types(bridge: &Path) -> Vec<String> {
    let Ok(bridge) = fs::read_to_string(bridge) else {
        return Vec::new();
    };
    let Some(list) = bridge.split("json_bridge!(").nth(1).and_then(|rest| rest.split(");").next()) else {
        return Vec::new();
    };
    list.lines()
        .map(|line| line.trim().trim_end_matches(','))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn swift_json_bridge(types: &[String]) -> String {
    let header = HEADER.replace("core/bin/generate/remote_types.yml", JSON_BRIDGE_PATH);
    let conformances: String = types.iter().map(|name| format!("extension Primitives.{name}: JsonCodable {{}}\n")).collect();
    format!("{header}\nimport Primitives\n\n{conformances}")
}

fn enum_variants(source: &str, name: &str) -> Option<Vec<String>> {
    let start = source.find(&format!("pub enum {name} {{"))?;
    let body = source[start..].split_once('{')?.1.split_once("\n}")?.0;
    Some(
        body.lines()
            .map(|line| line.trim().trim_end_matches(',').to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("//"))
            .collect(),
    )
}

pub fn kotlin_tagged_bridge(types: &[String]) -> String {
    let header = HEADER.replace("core/bin/generate/remote_types.yml", JSON_BRIDGE_PATH);
    let imports: String = types.iter().map(|name| format!("import com.wallet.core.primitives.{name}\n")).collect();
    let overloads: String = types
        .iter()
        .map(|name| format!("\nfun {name}.toJson(): String = jsonEncoder.encodeToString<{name}>(this)\n"))
        .collect();
    format!("{header}\npackage com.gemwallet.android.serializer\n\nimport kotlinx.serialization.encodeToString\n{imports}{overloads}")
}
