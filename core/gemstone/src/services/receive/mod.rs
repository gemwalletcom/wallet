pub mod model;
pub mod rules;

use primitives::Chain;

pub use model::GemMemoWarning;

#[derive(Default, uniffi::Object)]
pub struct GemReceiveService {}

#[uniffi::export]
impl GemReceiveService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn memo_warning(&self, chain: Chain) -> GemMemoWarning {
        rules::memo_warning(chain)
    }
}
