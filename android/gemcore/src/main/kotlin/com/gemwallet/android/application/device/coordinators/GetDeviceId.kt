package com.gemwallet.android.application.device.coordinators

interface GetDeviceId {
    suspend fun getDeviceId(): String

    suspend fun getDeviceKey(): String
}
