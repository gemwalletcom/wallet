pub trait OptionStringExt {
    fn non_empty(self) -> Self;
}

impl<T: AsRef<str>> OptionStringExt for Option<T> {
    fn non_empty(self) -> Self {
        self.filter(|value| !value.as_ref().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty() {
        assert_eq!(Some("gem").non_empty(), Some("gem"));
        assert_eq!(Some(String::new()).non_empty(), None);
        assert_eq!(None::<&str>.non_empty(), None);
    }
}
