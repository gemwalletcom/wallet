use std::iter::once;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum RoutePath<T> {
    Direct,
    Intermediary(T),
}

impl<T: Clone> RoutePath<T> {
    pub fn all(from: &T, to: &T, intermediaries: impl IntoIterator<Item = T>, matches: impl Fn(&T, &T) -> bool) -> Vec<Self> {
        once(Self::Direct)
            .chain(intermediaries.into_iter().filter(|intermediary| !matches(intermediary, from) && !matches(intermediary, to)).map(Self::Intermediary))
            .collect()
    }

    pub fn pairs(&self, from: &T, to: &T) -> Vec<(T, T)> {
        match self {
            Self::Direct => vec![(from.clone(), to.clone())],
            Self::Intermediary(intermediary) => vec![(from.clone(), intermediary.clone()), (intermediary.clone(), to.clone())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let paths = |from: &'static str, to: &'static str| RoutePath::all(&from, &to, ["sol", "usdc"], |a, b| a == b);

        assert_eq!(paths("sol", "usdc"), vec![RoutePath::Direct]);
        assert_eq!(paths("bonk", "sol"), vec![RoutePath::Direct, RoutePath::Intermediary("usdc")]);
        assert_eq!(paths("bonk", "jup"), vec![RoutePath::Direct, RoutePath::Intermediary("sol"), RoutePath::Intermediary("usdc")]);
    }

    #[test]
    fn test_pairs() {
        assert_eq!(RoutePath::Direct.pairs(&"bonk", &"jup"), vec![("bonk", "jup")]);
        assert_eq!(RoutePath::Intermediary("sol").pairs(&"bonk", &"jup"), vec![("bonk", "sol"), ("sol", "jup")]);
    }
}
