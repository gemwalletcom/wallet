package com.gemwallet.android.testkit

import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer

fun mockAppUpdateOffer(canSkip: Boolean = true, channel: AppUpdateChannel = AppUpdateChannel.Store) = AppUpdateOffer(
    version = "2.0.0",
    canSkip = canSkip,
    channel = channel,
)
