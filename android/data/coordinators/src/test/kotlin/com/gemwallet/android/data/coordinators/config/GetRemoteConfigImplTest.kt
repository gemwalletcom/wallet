package com.gemwallet.android.data.coordinators.config

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.Release
import com.wallet.core.primitives.SwapConfig
import com.wallet.core.primitives.PlatformStore
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfigService

class GetRemoteConfigImplTest {

    private val configService = mockk<GemConfigService>()

    private val subject = GetRemoteConfigImpl(
        configService = configService,
    )

    @Test
    fun getRemoteConfig_returnsGemApiConfig() = runTest {
        val config = ConfigResponse(
            releases = listOf(
                Release(
                    version = "2.0.13",
                    store = PlatformStore.GooglePlay,
                    upgradeRequired = false,
                )
            ),
            versions = ConfigVersions(
                fiatOnRampAssets = 1,
                fiatOffRampAssets = 2,
                swapAssets = 3,
            ),
            swap = SwapConfig(
                enabledProviders = emptyList(),
            ),
        )
        coEvery { configService.updateConfig() } returns config.toJson()

        val result = subject.getRemoteConfig()

        assertEquals(config, result)
    }
}
