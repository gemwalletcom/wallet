use std::cmp::Ordering;
use std::ops::{Mul, Sub};

pub trait PrioritizedProvider {
    fn provider_id(&self) -> &str;
    fn priority(&self) -> i32;
    fn threshold_bps(&self) -> i32;
}

pub fn sort_by_priority_then_amount<P, A>(a_id: &str, b_id: &str, a_amount: &A, b_amount: &A, providers: &[P], ascending: bool) -> Ordering
where
    P: PrioritizedProvider,
    A: PartialOrd + Clone + Sub<Output = A> + Mul<Output = A> + From<i32>,
{
    let a_provider = providers.iter().find(|p| p.provider_id() == a_id);
    let b_provider = providers.iter().find(|p| p.provider_id() == b_id);
    let a_pri = a_provider.map(|p| p.priority()).filter(|&p| p > 0);
    let b_pri = b_provider.map(|p| p.priority()).filter(|&p| p > 0);

    let cmp_amount = |x: &A, y: &A| x.partial_cmp(y).unwrap_or(Ordering::Equal);

    let by_amount = || {
        let ord = cmp_amount(a_amount, b_amount);
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

fn exceeds_threshold<P, A>(provider: &P, a: &A, b: &A, ascending: bool) -> bool
where
    P: PrioritizedProvider,
    A: PartialOrd + Clone + Sub<Output = A> + Mul<Output = A> + From<i32>,
{
    let bps = provider.threshold_bps();
    if bps == 0 {
        return false;
    }
    let better = if ascending == (a < b) { a } else { b }.clone();
    let diff = if a > b { a.clone() - b.clone() } else { b.clone() - a.clone() };
    diff * A::from(10000) > A::from(bps) * better
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FiatProvider, FiatProviderName};

    #[test]
    fn test_no_priority_sorts_by_amount_desc() {
        let providers: Vec<FiatProvider> = vec![];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &100.0, &200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_priority_wins_over_amount() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &100.0, &200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_threshold_override() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(500)), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &100.0, &200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_threshold_not_exceeded() {
        let providers = vec![
            FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(5000)),
            FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None),
        ];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &100.0, &110.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_unprioritized_sorted_after_prioritized() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &50.0, &200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_ascending_order() {
        let providers: Vec<FiatProvider> = vec![];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &100.0, &200.0, &providers, true);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_same_priority_sorts_by_amount() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &200.0, &100.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_priority_zero_treated_as_unranked() {
        let providers = vec![FiatProvider::mock_with_priority(FiatProviderName::MoonPay, 0, None), FiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 1, None)];
        let result = sort_by_priority_then_amount("moonpay", "mercuryo", &200.0, &100.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }
}
