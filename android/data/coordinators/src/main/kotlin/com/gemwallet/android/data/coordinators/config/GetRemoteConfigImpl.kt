package com.gemwallet.android.data.coordinators.config

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.ConfigResponse
import uniffi.gemstone.GemConfigService

class GetRemoteConfigImpl(
    private val configService: GemConfigService,
) : GetRemoteConfig {
    override suspend fun getRemoteConfig(): ConfigResponse = configService.getConfig().decodeJson()
}
