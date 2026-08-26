package com.gemwallet.android.services

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.application.swap.coordinators.SyncSwapAssets
import com.gemwallet.android.cases.device.SyncDevice
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.SwapConfig
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.Test
import java.io.IOException

class SyncServiceTest {
    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val syncFiatAssets = mockk<SyncFiatAssets>(relaxed = true)
    private val syncSwapAssets = mockk<SyncSwapAssets>(relaxed = true)
    private val syncDevice = mockk<SyncDevice>(relaxed = true)

    private val subject = SyncService(
        getRemoteConfig = getRemoteConfig,
        syncFiatAssets = syncFiatAssets,
        syncSwapAssets = syncSwapAssets,
        syncDevice = syncDevice,
    )

    @Test
    fun sync_fetchesConfigOnceAndSharesVersions() = runBlocking {
        val versions = ConfigVersions(fiatOnRampAssets = 1, fiatOffRampAssets = 2, swapAssets = 3)
        coEvery { getRemoteConfig.getRemoteConfig() } returns ConfigResponse(
            releases = emptyList(),
            versions = versions,
            swap = SwapConfig(enabledProviders = emptyList()),
        )

        subject.sync()

        coVerify(exactly = 1) { getRemoteConfig.getRemoteConfig() }
        coVerify { syncFiatAssets(versions) }
        coVerify { syncSwapAssets(versions) }
        coVerify { syncDevice.syncDevice() }
    }

    @Test
    fun sync_skipsAssetSyncWhenConfigFails() = runBlocking {
        coEvery { getRemoteConfig.getRemoteConfig() } throws IOException("offline")

        subject.sync()

        coVerify(exactly = 0) { syncFiatAssets(any()) }
        coVerify(exactly = 0) { syncSwapAssets(any()) }
        coVerify { syncDevice.syncDevice() }
    }
}
