package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemDeviceService

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletServiceTest {

    private val discoveryService = mockk<GemAssetDiscoveryService> {
        coEvery { discover(any()) } returns Unit
    }
    private val deviceService = mockk<GemDeviceService>(relaxed = true)

    @Test
    fun sync_discoversAssets() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerify { discoveryService.discover("wallet-1") }
    }

    @Test
    fun sync_discoversAssetsAfterDeviceSync() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerifyOrder {
            deviceService.synchronizeIfNeeded()
            discoveryService.discover("wallet-1")
        }
    }

    private fun TestScope.service() = ImportWalletService(
        discoveryService = discoveryService,
        deviceService = deviceService,
        scope = this,
    )
}
