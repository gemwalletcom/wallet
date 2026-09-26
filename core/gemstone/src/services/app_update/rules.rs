use super::{GemAppUpdateAction, GemAppUpdateOffer};
use crate::config::public::apk_download_url;
use crate::services::localization::GemLocalizedText;
use primitives::{PlatformStore, Release, is_version_higher};

pub fn newest_release(releases: &[Release], store: PlatformStore, current_version: &str) -> Option<Release> {
    releases.iter().find(|release| release.store == store && is_version_higher(release.version.clone(), current_version.to_string())).cloned()
}

pub fn available_update(releases: &[Release], store: PlatformStore, current_version: &str, skipped_version: Option<&str>) -> Option<Release> {
    newest_release(releases, store, current_version).filter(|release| release.upgrade_required || skipped_version != Some(release.version.as_str()))
}

pub fn update_offer(release: Release) -> GemAppUpdateOffer {
    GemAppUpdateOffer {
        title: GemLocalizedText::AppUpdateTitle,
        description: GemLocalizedText::AppUpdateDescription { version: release.version.clone() },
        actions: match release.upgrade_required {
            true => vec![GemAppUpdateAction::Update],
            false => vec![GemAppUpdateAction::Skip, GemAppUpdateAction::Update],
        },
        apk_url: (release.store == PlatformStore::ApkUniversal).then(|| apk_download_url(&release.version)),
        version: release.version,
    }
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

    #[test]
    fn test_update_offer_apk_url() {
        assert_eq!(
            update_offer(Release::new(PlatformStore::ApkUniversal, "2.29".into(), false)).apk_url,
            Some("https://apk.gemwallet.com/gem_wallet_universal_2.29.apk".into())
        );
        assert_eq!(update_offer(Release::new(PlatformStore::GooglePlay, "2.29".into(), true)).apk_url, None);
        assert_eq!(update_offer(Release::new(PlatformStore::Huawei, "2.29".into(), false)).apk_url, None);
    }

    #[test]
    fn test_an_update_prompt_names_the_version_and_offers_skip_only_when_optional() {
        let optional = update_offer(Release::new(PlatformStore::AppStore, "2.29".into(), false));
        let required = update_offer(Release::new(PlatformStore::AppStore, "2.29".into(), true));

        assert_eq!(optional.description, GemLocalizedText::AppUpdateDescription { version: "2.29".into() });
        assert_eq!(optional.actions, vec![GemAppUpdateAction::Skip, GemAppUpdateAction::Update]);
        assert!(optional.can_skip());
        assert_eq!(required.actions, vec![GemAppUpdateAction::Update]);
        assert!(!required.can_skip());
    }
}
