use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    Missing(String),
    MissingValue(String),
    Unexpected(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "missing argument: {name}"),
            Self::MissingValue(name) => write!(f, "missing value for {name}"),
            Self::Unexpected(argument) => write!(f, "unexpected argument: {argument}"),
        }
    }
}

impl Error for ArgsError {}
