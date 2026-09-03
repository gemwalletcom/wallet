use gem_client::Target;
use primitives::Chain;

#[derive(Clone, Copy, Debug)]
pub enum GemStaticApiTarget {
    GetValidators { chain: Chain },
}

impl Target for GemStaticApiTarget {
    fn path(&self) -> String {
        match self {
            Self::GetValidators { chain } => format!("/blockchains/{}/validators.json", chain.as_ref()),
        }
    }
}
