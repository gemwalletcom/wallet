package com.gemwallet.android.services

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.SwapConfig
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import okio.IOException
import org.junit.Test
import uniffi.gemstone.GemAssetsService

class SyncServiceTest {

    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val assetsService = mockk<GemAssetsService>(relaxed = true)
    private val syncDevice = mockk<SyncDevice>(relaxed = true)

    private val subject = SyncService(
        getRemoteConfig = getRemoteConfig,
        assetsService = assetsService,
        syncDevice = syncDevice,
    )

    @Test
    fun sync_passesConfigVersionsToAssetsSync() = runBlocking {
        val versions = ConfigVersions(fiatOnRampAssets = 1, fiatOffRampAssets = 2, swapAssets = 3)
        coEvery { getRemoteConfig.getRemoteConfig() } returns ConfigResponse(
            releases = emptyList(),
            versions = versions,
            swap = SwapConfig(enabledProviders = emptyList()),
        )

        subject.sync()

        coVerify { assetsService.syncAvailability(versions.toJson()) }
        coVerify { syncDevice.syncDevice() }
    }

    @Test
    fun sync_skipsAssetSyncWhenConfigFails() = runBlocking {
        coEvery { getRemoteConfig.getRemoteConfig() } throws IOException("offline")

        subject.sync()

        coVerify(exactly = 0) { assetsService.syncAvailability(any()) }
        coVerify { syncDevice.syncDevice() }
    }
}
