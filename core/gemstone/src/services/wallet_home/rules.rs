pub fn shows_initial_loading(initial_load_completed: bool, assets_timestamp: u64) -> bool {
    !initial_load_completed && assets_timestamp == 0
}

#[cfg(test)]
mod tests {
    use super::shows_initial_loading;

    #[test]
    fn test_shows_initial_loading_only_before_the_first_discovery() {
        assert!(shows_initial_loading(false, 0));
        assert!(!shows_initial_loading(true, 0));
        assert!(!shows_initial_loading(false, 1));
        assert!(!shows_initial_loading(true, 1));
    }
}
