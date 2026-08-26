use std::collections::HashSet;

use primitives::PriceAlert;

pub struct PriceAlertSync {
    pub delete_ids: Vec<String>,
    pub alerts: Vec<PriceAlert>,
}

pub fn reconcile(local: Vec<PriceAlert>, remote: Vec<PriceAlert>) -> PriceAlertSync {
    let remote_ids: HashSet<String> = remote.iter().map(PriceAlert::id).collect();
    let delete_ids = local.iter().map(PriceAlert::id).filter(|id| !remote_ids.contains(id)).collect();
    PriceAlertSync { delete_ids, alerts: remote }
}
