use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum TonApiTarget {
    Rates { tokens: Vec<String> },
}

impl Target for TonApiTarget {
    fn path(&self) -> String {
        match self {
            Self::Rates { tokens } => build_path_with_query("/v2/rates", &[("tokens", tokens.join(",")), ("currencies", "usd".to_string())]),
        }
    }
}

#[derive(Clone, Debug)]
pub enum StonfiTarget {
    Assets,
}

impl Target for StonfiTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets => "/v1/assets/query".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            TonApiTarget::Rates {
                tokens: vec!["ton".into(), "EQAB".into()]
            }
            .path(),
            "/v2/rates?tokens=ton%2CEQAB&currencies=usd"
        );
    }
}
