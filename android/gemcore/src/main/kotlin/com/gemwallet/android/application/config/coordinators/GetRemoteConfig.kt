package com.gemwallet.android.application.config.coordinators

import com.wallet.core.primitives.ConfigResponse

interface GetRemoteConfig {
    suspend fun getRemoteConfig(): ConfigResponse
}
