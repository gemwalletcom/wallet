package com.gemwallet.android.data.coordinators.asset

import io.mockk.coEvery
import io.mockk.coVerifyOrder
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.wallet.core.primitives.Currency
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemDeviceService

class DeviceAssetsSyncServiceTest {

    private val discoveryService = mockk<GemAssetDiscoveryService>()
    private val getCurrentCurrency = mockk<GetCurrentCurrency> {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }
    private val deviceService = mockk<GemDeviceService>(relaxed = true)

    private val subject = DeviceAssetsSyncService(
        deviceService = deviceService,
        discoveryService = discoveryService,
    )

    @Test
    fun sync_synchronizesDeviceBeforeDiscoveringAssets() = runTest {
        coEvery { discoveryService.discover("wallet-1") } returns emptyList()

        subject.sync("wallet-1")

        coVerifyOrder {
            deviceService.synchronizeIfNeeded()
            discoveryService.discover("wallet-1")
        }
    }
}
