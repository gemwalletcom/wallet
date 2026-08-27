use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use primitives::PriceAlert;

pub struct PriceAlertSync {
    pub delete_ids: Vec<String>,
    pub alerts: Vec<PriceAlert>,
}

pub fn reconcile(local: Vec<PriceAlert>, remote: Vec<PriceAlert>) -> PriceAlertSync {
    let remote_ids: HashSet<String> = remote.iter().map(PriceAlert::id).collect();
    let delete_ids = local.iter().map(PriceAlert::id).filter(|id| !remote_ids.contains(id)).collect();
    let local_notified: HashMap<String, Option<DateTime<Utc>>> = local.iter().map(|alert| (alert.id(), alert.last_notified_at)).collect();
    let alerts = remote
        .into_iter()
        .filter(|alert| local_notified.get(&alert.id()) != Some(&alert.last_notified_at))
        .collect();
    PriceAlertSync { delete_ids, alerts }
}
