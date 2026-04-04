package com.gemwallet.android.data.coordinators.config

import com.gemwallet.android.data.services.gemapi.GemApiClient
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

class GetRemoteConfigImplTest {

    private val gemApiClient = mockk<GemApiClient>()

    private val subject = GetRemoteConfigImpl(
        gemApiClient = gemApiClient,
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
        coEvery { gemApiClient.getConfig() } returns config

        val result = subject.getRemoteConfig()

        assertEquals(config, result)
    }
}
