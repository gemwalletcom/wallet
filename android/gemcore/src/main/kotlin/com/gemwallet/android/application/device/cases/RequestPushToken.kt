package com.gemwallet.android.application.device.cases

import android.content.Context

interface RequestPushToken {

    fun initRequester(context: Context)

    suspend fun requestToken(callback: (String) -> Unit)
}