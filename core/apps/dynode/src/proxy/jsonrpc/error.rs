use std::{error::Error, fmt};

use serde_json::Error as JsonError;
use serde_json::error::Category;

#[derive(Debug)]
pub(crate) enum ResponseError {
    Decode { status: u16, category: Category, line: usize, column: usize },
    InvalidBatch { status: u16, detail: &'static str },
}

impl ResponseError {
    pub(super) fn decode(status: u16, error: JsonError) -> Self {
        Self::Decode {
            status,
            category: error.classify(),
            line: error.line(),
            column: error.column(),
        }
    }

    pub(crate) fn status(&self) -> u16 {
        match self {
            Self::Decode { status, .. } | Self::InvalidBatch { status, .. } => *status,
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode { status, category, line, column } => {
                let category = match category {
                    Category::Io => "io",
                    Category::Syntax => "syntax",
                    Category::Data => "data",
                    Category::Eof => "eof",
                };
                write!(formatter, "response_decode status={status} category={category} line={line} column={column}")
            }
            Self::InvalidBatch { status, detail } => write!(formatter, "invalid_rpc_batch status={status} detail={detail}"),
        }
    }
}

impl Error for ResponseError {}
