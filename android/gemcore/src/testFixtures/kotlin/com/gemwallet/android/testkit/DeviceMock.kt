package com.gemwallet.android.testkit

import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Device
import com.wallet.core.primitives.DeviceLocale
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore

fun mockDevice(
    id: String = "test-device-id",
    platform: Platform = Platform.Android,
    platformStore: PlatformStore = PlatformStore.GooglePlay,
    os: String = "Android 15 (SDK 35)",
    model: String = "Pixel 10",
    token: String = "test-token-123",
    locale: DeviceLocale = DeviceLocale.EN,
    version: String = "1.0.0",
    currency: Currency = Currency.USD,
    isPushEnabled: Boolean = true,
    isPriceAlertsEnabled: Boolean? = true,
    subscriptionsVersion: Int = 1,
) = Device(
    id = id,
    platform = platform,
    platformStore = platformStore,
    os = os,
    model = model,
    token = token,
    locale = locale,
    version = version,
    currency = currency,
    isPushEnabled = isPushEnabled,
    isPriceAlertsEnabled = isPriceAlertsEnabled,
    subscriptionsVersion = subscriptionsVersion,
)
