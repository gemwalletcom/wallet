package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.cases.device.SynchronizeDeviceIfNeeded
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import io.mockk.coEvery
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class DeviceAssetsSyncServiceTest {

    private val walletPreferences = mockk<WalletPreferences>(relaxed = true)
    private val walletPreferencesFactory = mockk<WalletPreferencesFactory> {
        every { create(any()) } returns walletPreferences
    }
    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val synchronizeDeviceIfNeeded = mockk<SynchronizeDeviceIfNeeded>(relaxed = true)

    private val subject = DeviceAssetsSyncService(
        walletPreferencesFactory = walletPreferencesFactory,
        gemDeviceApiClient = gemDeviceApiClient,
        prefetchAssets = mockk(relaxed = true),
        ensureWalletAssets = mockk(relaxed = true),
        enableAsset = mockk(relaxed = true),
        assetsRepository = mockk(relaxed = true),
        walletsRepository = mockk(relaxed = true),
        synchronizeDeviceIfNeeded = synchronizeDeviceIfNeeded,
    )

    @Test
    fun sync_synchronizesDeviceBeforeRequestingAssets() = runTest {
        coEvery { gemDeviceApiClient.getAssets(any(), any()) } returns emptyList()

        subject.sync("wallet-1")

        coVerifyOrder {
            synchronizeDeviceIfNeeded.synchronizeIfNeeded()
            gemDeviceApiClient.getAssets(any(), any())
        }
    }
}
