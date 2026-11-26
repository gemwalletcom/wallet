use std::error::Error;
use std::process::ExitCode;

use gem_args::Args;
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage, generate};

const USAGE: &str = "Usage: uniffi-bindgen generate --language <kotlin|swift> --out-dir <dir> [--crate <name>] [--no-format] <library>";

fn target_language(language: String) -> Result<TargetLanguage, String> {
    match language.as_str() {
        "kotlin" => Ok(TargetLanguage::Kotlin),
        "swift" => Ok(TargetLanguage::Swift),
        _ => Err(format!("unsupported language: {language}")),
    }
}

fn generate_options(mut args: Args) -> Result<GenerateOptions, Box<dyn Error + Send + Sync>> {
    let languages = args.options(&["--language", "-l"])?.into_iter().map(target_language).collect::<Result<_, _>>()?;
    let out_dir = args.required(&["--out-dir", "-o"])?;
    let crate_filter = args.option(&["--crate"])?;
    let format = !args.flag(&["--no-format", "-n"]);
    let command = args.positional("command")?;
    if command != "generate" {
        return Err(format!("unsupported command: {command}").into());
    }
    let source = args.positional("library")?;
    args.finish()?;
    Ok(GenerateOptions {
        languages,
        source: source.into(),
        out_dir: out_dir.into(),
        format,
        crate_filter,
        ..GenerateOptions::default()
    })
}

fn main() -> ExitCode {
    let mut args = Args::from_env();
    if args.flag(&["--help", "-h"]) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let options = match generate_options(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match generate(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Args {
        Args::new(values.iter().map(|value| value.to_string()))
    }

    #[test]
    fn test_generate_options() {
        let options = generate_options(args(&["generate", "--language", "swift", "--crate", "gemstone", "--no-format", "-o", "generated/swift", "target/debug/libgemstone.dylib"])).unwrap();

        assert_eq!(format!("{:?}", options.languages), "[Swift]");
        assert_eq!(options.source.as_str(), "target/debug/libgemstone.dylib");
        assert_eq!(options.out_dir.as_str(), "generated/swift");
        assert_eq!(options.crate_filter.as_deref(), Some("gemstone"));
        assert!(!options.format);

        assert_eq!(format!("{:?}", generate_options(args(&["generate", "-l", "kotlin", "-o", "out", "lib"])).unwrap().languages), "[Kotlin]");
        assert!(generate_options(args(&["generate", "-l", "python", "-o", "out", "lib"])).is_err());
        assert!(generate_options(args(&["generate", "-l", "swift", "lib"])).is_err());
        assert!(generate_options(args(&["scaffolding", "-l", "swift", "-o", "out", "lib"])).is_err());
        assert!(generate_options(args(&["generate", "-l", "swift", "-o", "out", "lib", "--unknown"])).is_err());
    }
}
