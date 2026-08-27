use std::collections::HashMap;

use crate::services::collections::stale;

use chrono::{DateTime, Utc};
use primitives::PriceAlert;

pub struct PriceAlertSync {
    pub delete_ids: Vec<String>,
    pub alerts: Vec<PriceAlert>,
}

pub fn reconcile(local: Vec<PriceAlert>, remote: Vec<PriceAlert>) -> PriceAlertSync {
    let delete_ids = stale(local.iter().map(PriceAlert::id), remote.iter().map(PriceAlert::id));
    let local_notified: HashMap<String, Option<DateTime<Utc>>> = local.iter().map(|alert| (alert.id(), alert.last_notified_at)).collect();
    let alerts = remote
        .into_iter()
        .filter(|alert| local_notified.get(&alert.id()) != Some(&alert.last_notified_at))
        .collect();
    PriceAlertSync { delete_ids, alerts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, currency::Currency};

    fn alert(price: f64, notified: Option<i64>) -> PriceAlert {
        PriceAlert {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            currency: Currency::USD,
            price: Some(price),
            price_percent_change: None,
            price_direction: None,
            last_notified_at: notified.and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0)),
            identifier: String::new(),
        }
    }

    #[test]
    fn test_reconcile_deletes_stale_and_keeps_changed_alerts() {
        let local = vec![alert(1.0, None), alert(2.0, Some(10)), alert(3.0, None)];
        let remote = vec![alert(1.0, None), alert(2.0, Some(20)), alert(4.0, None)];

        let sync = reconcile(local, remote);

        assert_eq!(sync.delete_ids, vec![alert(3.0, None).id()]);
        assert_eq!(sync.alerts.iter().map(|alert| alert.price).collect::<Vec<_>>(), vec![Some(2.0), Some(4.0)]);
    }
}
