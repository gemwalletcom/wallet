package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.FiatAssets
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
    private val getSwapAssets = mockk<GetSwapAssets>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val availabilityService = mockk<AssetsAvailabilityService>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)

    private val subject = SyncSwapAssetsImpl(
        configStore = configStore,
        getSwapAssets = getSwapAssets,
        assetsRepository = assetsRepository,
        availabilityService = availabilityService,
        prefetchAssets = prefetchAssets,
    )

    @Test
    fun syncSwapAssets_marksAssetsSwappableAndStoresVersion() = runTest {
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } returns FiatAssets(495776u, listOf("bitcoin", "ethereum"))
        prefetchSucceeds()

        subject(versions(swapAssets = 495776))

        coVerify { prefetchAssets.prefetchAssets(listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum))) }
        coVerify { availabilityService.updateSwapAvailable(listOf("bitcoin", "ethereum")) }
        verify { configStore.putInt(SWAP_ASSETS_VERSION, 495776, "") }
    }

    @Test
    fun syncSwapAssets_skipsRequestWhenVersionIsCurrent() = runTest {
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495776

        subject(versions(swapAssets = 495776))

        coVerify(exactly = 0) { getSwapAssets() }
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { availabilityService.updateSwapAvailable(any()) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    @Test
    fun syncSwapAssets_keepsEveryQueryUnderSqliteVariableLimit() = runTest {
        val assetIds = List(1180) { "ethereum_0x$it" }
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 0
        coEvery { getSwapAssets() } returns FiatAssets(495776u, assetIds)
        prefetchSucceeds()

        val prefetched = mutableListOf<List<AssetId>>()
        val marked = mutableListOf<List<String>>()

        subject(versions(swapAssets = 495776))

        coVerify { prefetchAssets.prefetchAssets(capture(prefetched)) }
        coVerify { availabilityService.updateSwapAvailable(capture(marked)) }

        assertTrue(prefetched.all { it.size <= SQLITE_VARIABLE_LIMIT })
        assertTrue(marked.all { it.size <= SQLITE_VARIABLE_LIMIT })
        assertEquals(assetIds, marked.flatten())
    }

    @Test
    fun syncSwapAssets_keepsStoredVersionWhenRequestFails() = runTest {
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } throws RuntimeException("network down")

        subject(versions(swapAssets = 495776))

        coVerify(exactly = 0) { availabilityService.updateSwapAvailable(any()) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    @Test
    fun syncSwapAssets_keepsStoredVersionWhenAssetsAreMissingAfterPrefetch() = runTest {
        every { configStore.getInt(SWAP_ASSETS_VERSION) } returns 495775
        coEvery { getSwapAssets() } returns FiatAssets(495776u, listOf("bitcoin", "ethereum"))
        coEvery { assetsRepository.hasAssets(any()) } returns setOf(AssetId(Chain.Bitcoin))

        subject(versions(swapAssets = 495776))

        coVerify { availabilityService.updateSwapAvailable(listOf("bitcoin", "ethereum")) }
        verify(exactly = 0) { configStore.putInt(SWAP_ASSETS_VERSION, any(), any()) }
    }

    private fun prefetchSucceeds() {
        coEvery { assetsRepository.hasAssets(any()) } answers { firstArg<List<AssetId>>().toSet() }
    }

    private fun versions(swapAssets: Int) = ConfigVersions(
        fiatOnRampAssets = 0,
        fiatOffRampAssets = 0,
        swapAssets = swapAssets,
    )

    private companion object {
        const val SWAP_ASSETS_VERSION = "swap-assets-version"
        const val SQLITE_VARIABLE_LIMIT = 999
    }
}
