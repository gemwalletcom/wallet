package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.SwapConfig
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncFiatAssetsImplTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val getBuyableFiatAssets = mockk<GetBuyableFiatAssets>()
    private val getSellableFiatAssets = mockk<GetSellableFiatAssets>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)

    private val subject = SyncFiatAssetsImpl(
        configStore = configStore,
        getRemoteConfig = getRemoteConfig,
        getBuyableFiatAssets = getBuyableFiatAssets,
        getSellableFiatAssets = getSellableFiatAssets,
        assetsRepository = assetsRepository,
        prefetchAssets = prefetchAssets,
    )

    @Test
    fun syncFiatAssets_usesRemoteConfigToRefreshBuyAndSellAssets() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(
            fiatOnRampAssets = 2,
            fiatOffRampAssets = 3,
        )
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 1
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 2
        coEvery { getBuyableFiatAssets() } returns FiatAssets(5u, listOf("bitcoin"))
        coEvery { getSellableFiatAssets() } returns FiatAssets(7u, listOf("ethereum"))

        subject()

        coVerify { getRemoteConfig.getRemoteConfig() }
        coVerify {
            prefetchAssets.prefetchAssets(
                listOf(
                    AssetId(Chain.Bitcoin),
                    AssetId(Chain.Ethereum),
                )
            )
        }
        coVerify { assetsRepository.updateBuyAvailable(listOf("bitcoin")) }
        coVerify { assetsRepository.updateSellAvailable(listOf("ethereum")) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 5, "") }
        verify { configStore.putInt("fiat-off-ramp-assets-version", 7, "") }
    }

    @Test
    fun syncFiatAssets_skipsRefreshWhenVersionsAreCurrent() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(
            fiatOnRampAssets = 2,
            fiatOffRampAssets = 3,
        )
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 2
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 3

        subject()

        coVerify(exactly = 0) { getBuyableFiatAssets() }
        coVerify(exactly = 0) { getSellableFiatAssets() }
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { assetsRepository.updateBuyAvailable(any()) }
        coVerify(exactly = 0) { assetsRepository.updateSellAvailable(any()) }
    }

    @Test
    fun syncFiatAssets_updatesVersionsWhenRemoteAssetsAreEmpty() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(
            fiatOnRampAssets = 2,
            fiatOffRampAssets = 3,
        )
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 1
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 2
        coEvery { getBuyableFiatAssets() } returns FiatAssets(5u, emptyList())
        coEvery { getSellableFiatAssets() } returns FiatAssets(7u, emptyList())

        subject()

        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify { assetsRepository.updateBuyAvailable(emptyList()) }
        coVerify { assetsRepository.updateSellAvailable(emptyList()) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 5, "") }
        verify { configStore.putInt("fiat-off-ramp-assets-version", 7, "") }
    }

    private fun remoteConfig(
        fiatOnRampAssets: Int,
        fiatOffRampAssets: Int,
    ) = ConfigResponse(
        releases = emptyList(),
        versions = ConfigVersions(
            fiatOnRampAssets = fiatOnRampAssets,
            fiatOffRampAssets = fiatOffRampAssets,
            swapAssets = 0,
        ),
        swap = SwapConfig(
            enabledProviders = emptyList(),
        ),
    )
}
