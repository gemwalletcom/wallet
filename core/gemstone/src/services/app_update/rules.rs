use primitives::{PlatformStore, Release};

#[uniffi::export]
pub fn is_version_higher(new: String, current: String) -> bool {
    let new: Vec<u32> = new.split('.').filter_map(|part| part.parse().ok()).collect();
    let current: Vec<u32> = current.split('.').filter_map(|part| part.parse().ok()).collect();
    for (new_part, current_part) in new.iter().zip(current.iter()) {
        if new_part != current_part {
            return new_part > current_part;
        }
    }
    new.len() > current.len()
}

pub fn newest_release(releases: &[Release], store: PlatformStore, current_version: &str) -> Option<Release> {
    releases
        .iter()
        .find(|release| release.store == store && is_version_higher(release.version.clone(), current_version.to_string()))
        .cloned()
}

pub fn available_update(releases: &[Release], store: PlatformStore, current_version: &str, skipped_version: Option<&str>) -> Option<Release> {
    newest_release(releases, store, current_version).filter(|release| release.upgrade_required || skipped_version != Some(release.version.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_version_higher() {
        assert!(is_version_higher("1.2.3".into(), "1.0.0".into()));
        assert!(is_version_higher("2.1.3.4".into(), "2.1.3".into()));
        assert!(is_version_higher("0.1".into(), "0.0.1".into()));
        assert!(is_version_higher("2.27".into(), "2.0.27".into()));
        assert!(!is_version_higher("1.0.0".into(), "1.2.3".into()));
        assert!(!is_version_higher("1.2.3".into(), "1.2.3".into()));
        assert!(!is_version_higher("2.1.3".into(), "2.1.3.4".into()));
        assert!(!is_version_higher("1".into(), "2".into()));
        assert!(!is_version_higher("1.3.100".into(), "2.0.12".into()));
    }

    #[test]
    fn test_available_update() {
        let releases = vec![
            Release::new(PlatformStore::Huawei, "2.0.0".into(), false),
            Release::new(PlatformStore::GooglePlay, "3.0.0".into(), false),
            Release::new(PlatformStore::AppStore, "3.0.0".into(), true),
        ];

        assert_eq!(
            available_update(&releases, PlatformStore::GooglePlay, "1.0.0", None).map(|r| r.version),
            Some("3.0.0".into())
        );
        assert!(available_update(&releases, PlatformStore::Fdroid, "1.0.0", None).is_none());
        assert!(available_update(&releases, PlatformStore::GooglePlay, "3.0.0", None).is_none());
        assert!(available_update(&releases, PlatformStore::GooglePlay, "1.0.0", Some("3.0.0")).is_none());
        assert!(available_update(&releases, PlatformStore::AppStore, "1.0.0", Some("3.0.0")).is_some());
        assert!(newest_release(&releases, PlatformStore::GooglePlay, "1.0.0").is_some());
        assert!(newest_release(&releases, PlatformStore::GooglePlay, "3.0.0").is_none());
    }
}
