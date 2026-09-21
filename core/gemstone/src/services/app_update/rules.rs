use primitives::{PlatformStore, Release, is_version_higher};

pub fn newest_release(releases: &[Release], store: PlatformStore, current_version: &str) -> Option<Release> {
    releases.iter().find(|release| release.store == store && is_version_higher(release.version.clone(), current_version.to_string())).cloned()
}

pub fn available_update(releases: &[Release], store: PlatformStore, current_version: &str, skipped_version: Option<&str>) -> Option<Release> {
    newest_release(releases, store, current_version).filter(|release| release.upgrade_required || skipped_version != Some(release.version.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_update() {
        let releases = vec![
            Release::new(PlatformStore::Huawei, "2.0.0".into(), false),
            Release::new(PlatformStore::GooglePlay, "3.0.0".into(), false),
            Release::new(PlatformStore::AppStore, "3.0.0".into(), true),
        ];

        assert_eq!(available_update(&releases, PlatformStore::GooglePlay, "1.0.0", None).map(|r| r.version), Some("3.0.0".into()));
        assert!(available_update(&releases, PlatformStore::Fdroid, "1.0.0", None).is_none());
        assert!(available_update(&releases, PlatformStore::GooglePlay, "3.0.0", None).is_none());
        assert!(available_update(&releases, PlatformStore::GooglePlay, "1.0.0", Some("3.0.0")).is_none());
        assert!(available_update(&releases, PlatformStore::AppStore, "1.0.0", Some("3.0.0")).is_some());
        assert!(newest_release(&releases, PlatformStore::GooglePlay, "1.0.0").is_some());
        assert!(newest_release(&releases, PlatformStore::GooglePlay, "3.0.0").is_none());
    }
}
