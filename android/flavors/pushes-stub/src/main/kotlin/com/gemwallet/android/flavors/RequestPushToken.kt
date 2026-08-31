package com.gemwallet.android.flavors

import com.gemwallet.android.application.device.cases.RequestPushToken

class StoreRequestPushToken : RequestPushToken {

    override suspend fun requestToken(callback: (String) -> Unit) {
        callback("")
    }
}

fun isNotificationsAvailable() = false