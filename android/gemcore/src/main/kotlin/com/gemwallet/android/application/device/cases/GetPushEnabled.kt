package com.gemwallet.android.application.device.cases

import kotlinx.coroutines.flow.Flow

interface GetPushEnabled {
    fun getPushEnabled(): Flow<Boolean>
}