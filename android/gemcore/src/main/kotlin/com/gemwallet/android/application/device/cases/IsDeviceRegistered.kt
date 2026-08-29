package com.gemwallet.android.application.device.cases

interface IsDeviceRegistered {
    suspend fun isDeviceRegistered(): Boolean
}
