package com.gemwallet.android.application.device.cases

import uniffi.gemstone.GemPushState

interface SwitchPushEnabled {
    suspend fun switchPushEnabled(enabled: Boolean): GemPushState
}
