package com.gemwallet.android.model

import com.gemwallet.android.application.device.cases.RequestPushToken
import com.wallet.core.primitives.PlatformStore

data class BuildInfo(
    val platformStore: PlatformStore,
    val versionName: String,
    val versionCode: Int,
    val requestPushToken: RequestPushToken,
)
