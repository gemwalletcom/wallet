use std::cmp::Ordering;

use primitives::FiatProvider;

pub fn sort_by_priority_then_amount(a_id: &str, b_id: &str, a_amount: f64, b_amount: f64, providers: &[FiatProvider], ascending: bool) -> Ordering {
    let a_provider = providers.iter().find(|provider| provider.id.as_ref() == a_id);
    let b_provider = providers.iter().find(|provider| provider.id.as_ref() == b_id);
    let a_pri = a_provider.and_then(|provider| provider.priority).filter(|&priority| priority > 0);
    let b_pri = b_provider.and_then(|provider| provider.priority).filter(|&priority| priority > 0);

    let by_amount = || {
        let ord = a_amount.partial_cmp(&b_amount).unwrap_or(Ordering::Equal);
        if ascending { ord } else { ord.reverse() }
    };

    match (a_pri, b_pri) {
        (Some(a), Some(b)) if a != b => match if a < b { a_provider } else { b_provider } {
            Some(higher_pri) if !exceeds_threshold(higher_pri, a_amount, b_amount, ascending) => a.cmp(&b),
            _ => by_amount(),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        _ => by_amount(),
    }
}

fn exceeds_threshold(provider: &FiatProvider, a: f64, b: f64, ascending: bool) -> bool {
    let bps = provider.threshold_bps.unwrap_or(0);
    if bps == 0 {
        return false;
    }
    let better = if ascending == (a < b) { a } else { b };
    let diff = if a > b { a - b } else { b - a };
    diff * 10000.0 > f64::from(bps) * better
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::FiatProviderName;

    #[test]
    fn test_no_priority_sorts_by_amount_desc() {
        let providers: Vec<FiatProvider> = vec![];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_priority_wins_over_amount() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_threshold_override() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(500)), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_threshold_not_exceeded() {
        let providers = vec![
            FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(5000)),
            FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None),
        ];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 100.0, 110.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_unprioritized_sorted_after_prioritized() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 50.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_ascending_order() {
        let providers: Vec<FiatProvider> = vec![];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 100.0, 200.0, &providers, true);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_same_priority_sorts_by_amount() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 200.0, 100.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_priority_zero_treated_as_unranked() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 0, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", 200.0, 100.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }
}
