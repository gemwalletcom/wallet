use super::GemNavigationService;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;

impl GemNavigationService {
    pub fn mock(testkit: &DiscoveryTestkit) -> Self {
        Self::new(testkit.assets.clone(), testkit.balance.clone(), testkit.preferences.clone(), testkit.session.clone())
    }
}
