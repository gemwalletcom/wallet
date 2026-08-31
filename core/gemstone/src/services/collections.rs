use std::collections::HashSet;
use std::hash::Hash;

pub fn unique<T: Clone + Eq + Hash>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    unique_by(items, Clone::clone)
}

pub fn unique_by<T, K: Eq + Hash>(items: impl IntoIterator<Item = T>, key: impl Fn(&T) -> K) -> Vec<T> {
    let mut seen = HashSet::new();
    items.into_iter().filter(|item| seen.insert(key(item))).collect()
}

pub fn missing<T: Clone + Eq + Hash>(requested: impl IntoIterator<Item = T>, existing: impl IntoIterator<Item = T>) -> Vec<T> {
    missing_by(requested, existing, Clone::clone)
}

pub fn missing_by<T, K: Eq + Hash>(requested: impl IntoIterator<Item = T>, existing: impl IntoIterator<Item = K>, key: impl Fn(&T) -> K) -> Vec<T> {
    let existing: HashSet<K> = existing.into_iter().collect();
    unique_by(requested.into_iter().filter(|item| !existing.contains(&key(item))), &key)
}

pub fn stale<T: Eq + Hash>(existing: impl IntoIterator<Item = T>, incoming: impl IntoIterator<Item = T>) -> Vec<T> {
    let incoming: HashSet<T> = incoming.into_iter().collect();
    existing.into_iter().filter(|item| !incoming.contains(item)).collect()
}

pub fn stale_by<T, K: Eq + Hash>(existing: impl IntoIterator<Item = T>, incoming: impl IntoIterator<Item = K>, key: impl Fn(&T) -> K) -> Vec<T> {
    let incoming: HashSet<K> = incoming.into_iter().collect();
    existing.into_iter().filter(|item| !incoming.contains(&key(item))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_keeps_first_occurrence_order() {
        assert_eq!(unique(vec![2, 1, 2, 3, 1]), vec![2, 1, 3]);
        assert_eq!(unique_by(vec!["aa", "b", "cc"], |item| item.len()), vec!["aa", "b"]);
    }

    #[test]
    fn test_missing_and_stale_are_set_differences() {
        assert_eq!(missing(vec![1, 2, 2, 3], vec![2]), vec![1, 3]);
        assert_eq!(stale(vec![1, 2, 3], vec![2]), vec![1, 3]);
        assert!(missing(Vec::<i32>::new(), vec![1]).is_empty());
        assert_eq!(missing_by(vec!["aa", "b", "cc", "b"], vec![2], |item| item.len()), vec!["b"]);
        assert_eq!(stale_by(vec!["aa", "b"], vec![1], |item| item.len()), vec!["aa"]);
    }
}
