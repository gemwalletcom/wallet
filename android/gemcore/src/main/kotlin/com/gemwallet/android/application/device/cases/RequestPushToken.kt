package com.gemwallet.android.application.device.cases

interface RequestPushToken {

    suspend fun requestToken(callback: (String) -> Unit)
}
