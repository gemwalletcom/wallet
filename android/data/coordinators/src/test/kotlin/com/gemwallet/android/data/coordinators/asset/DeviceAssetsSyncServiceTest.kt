package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.cases.device.SyncDevice
import io.mockk.coEvery
import io.mockk.coVerifyOrder
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Currency
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemAssetDiscoveryService

class DeviceAssetsSyncServiceTest {

    private val discoveryService = mockk<GemAssetDiscoveryService>()
    private val sessionRepository = mockk<SessionRepository> {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }
    private val syncDevice = mockk<SyncDevice>(relaxed = true)

    private val subject = DeviceAssetsSyncService(
        syncDevice = syncDevice,
        discoveryService = discoveryService,
        sessionRepository = sessionRepository,
    )

    @Test
    fun sync_synchronizesDeviceBeforeDiscoveringAssets() = runTest {
        coEvery { discoveryService.discover("wallet-1") } returns emptyList()

        subject.sync("wallet-1")

        coVerifyOrder {
            syncDevice.syncDevice()
            discoveryService.discover("wallet-1")
        }
    }
}
