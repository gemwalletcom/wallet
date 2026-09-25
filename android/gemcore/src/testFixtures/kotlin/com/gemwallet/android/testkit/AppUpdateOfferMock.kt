package com.gemwallet.android.testkit

import uniffi.gemstone.GemAppUpdateOffer

fun mockAppUpdateOffer(canSkip: Boolean = true, apkUrl: String? = null) = GemAppUpdateOffer(
    version = "2.0.0",
    canSkip = canSkip,
    apkUrl = apkUrl,
)
