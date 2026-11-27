use std::process::ExitCode;

use gem_args::{Args, ArgsError};
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage, generate};

fn target_language(language: String) -> Result<TargetLanguage, ArgsError> {
    match language.as_str() {
        "kotlin" => Ok(TargetLanguage::Kotlin),
        "swift" => Ok(TargetLanguage::Swift),
        _ => Err(ArgsError::invalid(format!("unsupported language: {language}"))),
    }
}

fn generate_options(mut args: Args) -> Result<GenerateOptions, ArgsError> {
    let languages = args.options(&["--language", "-l"], "kotlin|swift", "Language to generate bindings for, repeatable");
    let out_dir = args.required(&["--out-dir", "-o"], "DIR", "Directory for the generated files");
    let crate_filter = args.option(&["--crate"], "NAME", "Only generate bindings for this crate");
    let no_format = args.flag(&["--no-format", "-n"], "Skip formatting the generated code");
    let command = args.positional("command", "generate");
    let source = args.positional("library", "Path to the built library");
    args.finish("Generate UniFFI bindings from a built library")?;
    if command != "generate" {
        return Err(ArgsError::invalid(format!("unsupported command: {command}")));
    }
    Ok(GenerateOptions {
        languages: languages.into_iter().map(target_language).collect::<Result<_, _>>()?,
        source: source.into(),
        out_dir: out_dir.into(),
        format: !no_format,
        crate_filter,
        ..GenerateOptions::default()
    })
}

fn main() -> ExitCode {
    let options = match generate_options(Args::from_env()) {
        Ok(options) => options,
        Err(error) => return error.report(),
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
        Args::new("uniffi-bindgen", values.iter().map(|value| value.to_string()))
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
