package com.gemwallet.android.application.device.cases

interface GetPushToken {
    suspend fun getPushToken(): String
}