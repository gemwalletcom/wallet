package com.gemwallet.android.data.coordinators.config

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.ConfigResponse

class GetRemoteConfigImpl(
    private val gemApiClient: GemApiClient,
) : GetRemoteConfig {
    override suspend fun getRemoteConfig(): ConfigResponse {
        return gemApiClient.getConfig()
    }
}
