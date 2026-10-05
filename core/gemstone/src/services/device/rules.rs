use primitives::Device;

pub fn device_changed(current: &Device, other: &Device) -> bool {
    current.id != other.id
        || current.token != other.token
        || current.locale != other.locale
        || current.version != other.version
        || current.currency != other.currency
        || current.is_push_enabled != other.is_push_enabled
        || current.is_price_alerts_enabled != other.is_price_alerts_enabled
        || current.subscriptions_version != other.subscriptions_version
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::currency::Currency;

    #[test]
    fn test_device_changed_tracks_synced_fields_only() {
        let remote = Device::mock();

        assert!(!device_changed(&remote, &remote.clone()));
        assert!(device_changed(&remote, &Device { currency: Currency::EUR, ..remote.clone() }));
        assert!(device_changed(&remote, &Device { subscriptions_version: 2, ..remote.clone() }));
        assert!(!device_changed(&remote, &Device { model: "Other".into(), ..remote.clone() }));
    }
}
