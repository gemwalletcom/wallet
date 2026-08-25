package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
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
import org.junit.Test

class SyncFiatAssetsImplTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val getBuyableFiatAssets = mockk<GetBuyableFiatAssets>()
    private val getSellableFiatAssets = mockk<GetSellableFiatAssets>()
    private val availabilityService = mockk<AssetsAvailabilityService>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)

    private val subject = SyncFiatAssetsImpl(
        configStore = configStore,
        getBuyableFiatAssets = getBuyableFiatAssets,
        getSellableFiatAssets = getSellableFiatAssets,
        availabilityService = availabilityService,
        prefetchAssets = prefetchAssets,
    )

    @Test
    fun syncFiatAssets_usesRemoteConfigToRefreshBuyAndSellAssets() = runTest {
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 1
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 2
        coEvery { getBuyableFiatAssets() } returns FiatAssets(5u, listOf("bitcoin"))
        coEvery { getSellableFiatAssets() } returns FiatAssets(7u, listOf("ethereum"))

        subject(versions(fiatOnRampAssets = 2, fiatOffRampAssets = 3))

        coVerify {
            prefetchAssets.prefetchAssets(
                listOf(
                    AssetId(Chain.Bitcoin),
                    AssetId(Chain.Ethereum),
                )
            )
        }
        coVerify { availabilityService.updateBuyAvailable(listOf("bitcoin")) }
        coVerify { availabilityService.updateSellAvailable(listOf("ethereum")) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 5, "") }
        verify { configStore.putInt("fiat-off-ramp-assets-version", 7, "") }
    }

    @Test
    fun syncFiatAssets_refreshesWhenRemoteVersionDiffersFromStored() = runTest {
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 9
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 3
        coEvery { getBuyableFiatAssets() } returns FiatAssets(2u, listOf("bitcoin"))

        subject(versions(fiatOnRampAssets = 2, fiatOffRampAssets = 3))

        coVerify { getBuyableFiatAssets() }
        coVerify(exactly = 0) { getSellableFiatAssets() }
        coVerify { availabilityService.updateBuyAvailable(listOf("bitcoin")) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 2, "") }
    }

    @Test
    fun syncFiatAssets_skipsRefreshWhenVersionsAreCurrent() = runTest {
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 2
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 3

        subject(versions(fiatOnRampAssets = 2, fiatOffRampAssets = 3))

        coVerify(exactly = 0) { getBuyableFiatAssets() }
        coVerify(exactly = 0) { getSellableFiatAssets() }
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { availabilityService.updateBuyAvailable(any()) }
        coVerify(exactly = 0) { availabilityService.updateSellAvailable(any()) }
    }

    @Test
    fun syncFiatAssets_updatesVersionsWhenRemoteAssetsAreEmpty() = runTest {
        every { configStore.getInt("fiat-on-ramp-assets-version") } returns 1
        every { configStore.getInt("fiat-off-ramp-assets-version") } returns 2
        coEvery { getBuyableFiatAssets() } returns FiatAssets(5u, emptyList())
        coEvery { getSellableFiatAssets() } returns FiatAssets(7u, emptyList())

        subject(versions(fiatOnRampAssets = 2, fiatOffRampAssets = 3))

        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify { availabilityService.updateBuyAvailable(emptyList()) }
        coVerify { availabilityService.updateSellAvailable(emptyList()) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 5, "") }
        verify { configStore.putInt("fiat-off-ramp-assets-version", 7, "") }
    }

    private fun versions(
        fiatOnRampAssets: Int,
        fiatOffRampAssets: Int,
    ) = ConfigVersions(
        fiatOnRampAssets = fiatOnRampAssets,
        fiatOffRampAssets = fiatOffRampAssets,
        swapAssets = 0,
    )
}
