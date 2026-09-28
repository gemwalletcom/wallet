use gem_client::Target;
use primitives::Chain;

#[derive(Clone, Debug)]
pub enum StaticAssetsTarget {
    Validators { chain: Chain },
    Assets { chain: Chain },
}

impl Target for StaticAssetsTarget {
    fn path(&self) -> String {
        match self {
            Self::Validators { chain } => format!("/blockchains/{}/validators.json", chain.as_ref()),
            Self::Assets { chain } => format!("/blockchains/{}/assets.json", chain.as_ref()),
        }
    }
}
