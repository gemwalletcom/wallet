use gem_client::Target;

#[derive(Clone, Debug)]
pub(super) enum JupiterTarget {
    Build,
}

impl Target for JupiterTarget {
    fn path(&self) -> String {
        match self {
            Self::Build => "/swap/v2/build".to_string(),
        }
    }
}
