use std::error::Error;
use std::fmt;
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    Help(String),
    Invalid(String),
}

impl ArgsError {
    pub fn invalid(error: impl fmt::Display) -> Self {
        Self::Invalid(error.to_string())
    }

    pub fn report(&self) -> ExitCode {
        match self {
            Self::Help(usage) => {
                println!("{usage}");
                ExitCode::SUCCESS
            }
            Self::Invalid(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        }
    }
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help(text) | Self::Invalid(text) => f.write_str(text),
        }
    }
}

impl Error for ArgsError {}
