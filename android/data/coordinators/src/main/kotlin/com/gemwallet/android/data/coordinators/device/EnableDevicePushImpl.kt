package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.device.cases.EnableDevicePush
import com.gemwallet.android.application.device.cases.SwitchPushEnabled

class EnableDevicePushImpl(
    private val switchPushEnabled: SwitchPushEnabled,
) : EnableDevicePush {

    override suspend fun invoke() {
        switchPushEnabled.switchPushEnabled(true)
    }
}
