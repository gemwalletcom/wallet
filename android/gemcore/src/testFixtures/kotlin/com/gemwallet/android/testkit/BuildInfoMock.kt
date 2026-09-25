package com.gemwallet.android.testkit

import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore

fun mockBuildInfo(platformStore: PlatformStore = PlatformStore.GooglePlay) = BuildInfo(
    platformStore = platformStore,
    versionName = "1.0.0",
    versionCode = 1,
)
