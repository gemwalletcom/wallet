package com.gemwallet.android.data.coordinators.config

import com.gemwallet.android.application.config.cases.GetRemoteConfig
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.ConfigResponse
import uniffi.gemstone.GemConfigService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class GetRemoteConfigImpl(
    private val configService: GemConfigService,
) : GetRemoteConfig {
    override suspend fun getRemoteConfig(): ConfigResponse = withContext(Dispatchers.IO) {
        configService.updateConfig().decodeJson()
    }
}
