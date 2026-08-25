package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.device.coordinators.EnableDevicePush
import com.gemwallet.android.cases.device.SwitchPushEnabled

class EnableDevicePushImpl(
    private val switchPushEnabled: SwitchPushEnabled,
) : EnableDevicePush {

    override suspend fun invoke() {
        switchPushEnabled.switchPushEnabled(true)
    }
}
