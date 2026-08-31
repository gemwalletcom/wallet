package com.gemwallet.android.application.config.cases

import com.wallet.core.primitives.ConfigResponse

interface GetRemoteConfig {
    suspend fun getRemoteConfig(): ConfigResponse
}
