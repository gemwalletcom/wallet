use std::error::Error;

use clap::{Parser, Subcommand, ValueEnum};
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage, generate};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Generate {
        #[arg(long, short, value_enum)]
        language: Vec<Language>,
        #[arg(long, short)]
        out_dir: String,
        #[arg(long, short)]
        no_format: bool,
        #[arg(long = "crate")]
        crate_name: Option<String>,
        source: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Language {
    Kotlin,
    Swift,
}

impl From<Language> for TargetLanguage {
    fn from(language: Language) -> Self {
        match language {
            Language::Kotlin => Self::Kotlin,
            Language::Swift => Self::Swift,
        }
    }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let Command::Generate {
        language,
        out_dir,
        no_format,
        crate_name,
        source,
    } = Cli::parse().command;
    generate(GenerateOptions {
        languages: language.into_iter().map(TargetLanguage::from).collect(),
        source: source.into(),
        out_dir: out_dir.into(),
        format: !no_format,
        crate_filter: crate_name,
        ..GenerateOptions::default()
    })?;
    Ok(())
}
