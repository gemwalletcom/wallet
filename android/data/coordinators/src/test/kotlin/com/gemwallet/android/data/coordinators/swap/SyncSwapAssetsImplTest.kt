package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
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
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class SyncSwapAssetsImplTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val getSwapAssets = mockk<GetSwapAssets>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)

    private val subject = SyncSwapAssetsImpl(
        configStore = configStore,
        getRemoteConfig = getRemoteConfig,
        getSwapAssets = getSwapAssets,
        assetsRepository = assetsRepository,
        prefetchAssets = prefetchAssets,
    )

    @Test
    fun syncSwapAssets_marksAssetsSwappableAndStoresVersion() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(swapAssets = 495776)
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } returns FiatAssets(495776u, listOf("bitcoin", "ethereum"))
        prefetchSucceeds()

        subject()

        coVerify { prefetchAssets.prefetchAssets(listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum))) }
        coVerify { assetsRepository.updateSwapAvailable(listOf("bitcoin", "ethereum")) }
        verify { configStore.putInt(SWAP_ASSETS_VERSION, 495776, "") }
    }

    @Test
    fun syncSwapAssets_skipsRequestWhenVersionIsCurrent() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(swapAssets = 495776)
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495776

        subject()

        coVerify(exactly = 0) { getSwapAssets() }
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { assetsRepository.updateSwapAvailable(any()) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    @Test
    fun syncSwapAssets_keepsEveryQueryUnderSqliteVariableLimit() = runTest {
        val assetIds = List(1180) { "ethereum_0x$it" }
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(swapAssets = 495776)
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 0
        coEvery { getSwapAssets() } returns FiatAssets(495776u, assetIds)
        prefetchSucceeds()

        val prefetched = mutableListOf<List<AssetId>>()
        val marked = mutableListOf<List<String>>()

        subject()

        coVerify { prefetchAssets.prefetchAssets(capture(prefetched)) }
        coVerify { assetsRepository.updateSwapAvailable(capture(marked)) }

        assertTrue(prefetched.all { it.size <= SQLITE_VARIABLE_LIMIT })
        assertTrue(marked.all { it.size <= SQLITE_VARIABLE_LIMIT })
        assertEquals(assetIds, marked.flatten())
    }

    @Test
    fun syncSwapAssets_keepsStoredVersionWhenRequestFails() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(swapAssets = 495776)
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } throws RuntimeException("network down")

        subject()

        coVerify(exactly = 0) { assetsRepository.updateSwapAvailable(any()) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    @Test
    fun syncSwapAssets_keepsStoredVersionWhenAssetsAreMissingAfterPrefetch() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(swapAssets = 495776)
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } returns FiatAssets(495776u, listOf("bitcoin", "ethereum"))
        coEvery { assetsRepository.hasAssets(any()) } returns setOf(AssetId(Chain.Bitcoin))

        subject()

        coVerify { assetsRepository.updateSwapAvailable(listOf("bitcoin", "ethereum")) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    private fun prefetchSucceeds() {
        coEvery { assetsRepository.hasAssets(any()) } answers { firstArg<List<AssetId>>().toSet() }
    }

    private fun remoteConfig(swapAssets: Int) = ConfigResponse(
        releases = emptyList(),
        versions = ConfigVersions(
            fiatOnRampAssets = 0,
            fiatOffRampAssets = 0,
            swapAssets = swapAssets,
        ),
        swap = SwapConfig(
            enabledProviders = emptyList(),
        ),
    )

    private companion object {
        const val SWAP_ASSETS_VERSION = "swap-assets-version"
        const val SQLITE_VARIABLE_LIMIT = 999
    }
}
