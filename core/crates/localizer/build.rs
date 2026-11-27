use std::error::Error;
use std::path::Path;
use std::{env, fs};

const DIRECTORY: &str = "i18n";
const FALLBACK: &str = "en";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed={DIRECTORY}");
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR")?).join(DIRECTORY);
    let file = format!("{}.ftl", env::var("CARGO_PKG_NAME")?);

    let mut directories = fs::read_dir(&root)?.map(|entry| entry.map(|entry| entry.path())).collect::<Result<Vec<_>, _>>()?;
    directories.sort();

    let (mut sources, mut ids) = (String::new(), None);
    for directory in directories {
        let path = directory.join(&file);
        let Some(language) = directory.file_name().and_then(|name| name.to_str()).filter(|_| path.is_file()) else {
            continue;
        };
        let source = fs::read_to_string(&path)?;
        let messages = gem_fluent::parse(&source).map_err(|error| format!("{}: {error}", path.display()))?;
        if language == FALLBACK {
            ids = Some(messages.iter().map(|message| format!("    pub(crate) const {0}: &str = {0:?};\n", message.id)).collect::<String>());
        }
        sources.push_str(&format!("    ({language:?}, include_str!({path:?})),\n"));
    }
    let ids = ids.ok_or(format!("{} has no {FALLBACK}/{file}", root.display()))?;

    fs::write(
        Path::new(&env::var("OUT_DIR")?).join("i18n.rs"),
        format!("pub(crate) const FALLBACK: &str = {FALLBACK:?};\n\npub(crate) const SOURCES: &[(&str, &str)] = &[\n{sources}];\n\n#[allow(non_upper_case_globals)]\npub(crate) mod id {{\n{ids}}}\n"),
    )?;
    Ok(())
}
