package com.gemwallet.android.testkit

import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Release

fun mockRelease() = Release(
    version = "3.0.0",
    store = PlatformStore.GooglePlay,
    upgradeRequired = false,
)
