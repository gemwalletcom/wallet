use crate::{Asset, Banner, BannerEvent, BannerState, Chain};

impl Banner {
    pub fn mock(event: BannerEvent, state: BannerState) -> Self {
        Banner {
            wallet_id: None,
            asset: Some(Asset::from_chain(Chain::Ethereum)),
            event,
            state,
        }
    }
}
