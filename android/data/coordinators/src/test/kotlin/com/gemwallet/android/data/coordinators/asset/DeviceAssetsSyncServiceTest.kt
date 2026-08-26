package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import io.mockk.coEvery
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import uniffi.gemstone.GemTransactionsService
import com.gemwallet.android.serializer.toJson
import org.junit.Test

class DeviceAssetsSyncServiceTest {

    private val walletPreferences = mockk<WalletPreferences>(relaxed = true)
    private val walletPreferencesFactory = mockk<WalletPreferencesFactory> {
        every { create(any()) } returns walletPreferences
    }
    private val transactionsService = mockk<GemTransactionsService>()
    private val syncDevice = mockk<SyncDevice>(relaxed = true)

    private val subject = DeviceAssetsSyncService(
        walletPreferencesFactory = walletPreferencesFactory,
        transactionsService = transactionsService,
        prefetchAssets = mockk(relaxed = true),
        ensureWalletAssets = mockk(relaxed = true),
        enableAsset = mockk(relaxed = true),
        assetsRepository = mockk(relaxed = true),
        walletsRepository = mockk(relaxed = true),
        syncDevice = syncDevice,
    )

    @Test
    fun sync_synchronizesDeviceBeforeRequestingAssets() = runTest {
        coEvery { transactionsService.getAssetsList(any(), any()) } returns emptyList()

        subject.sync("wallet-1")

        coVerifyOrder {
            syncDevice.syncDevice()
            transactionsService.getAssetsList(any(), any())
        }
    }
}
