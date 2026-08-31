use primitives::Chain;

use crate::method::GemApiMethod;

#[derive(Clone, Copy, Debug)]
pub enum GemStaticApiTarget {
    GetValidators(Chain),
}

impl GemStaticApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::GetValidators(_) => GemApiMethod::Get,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::GetValidators(chain) => format!("/blockchains/{}/validators.json", chain.as_ref()),
        }
    }
}
