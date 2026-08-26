package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.application.wallet_import.coordinators.SetupWallet
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Currency
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemAssetDiscoveryService

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletServiceTest {

    private val discoveryService = mockk<GemAssetDiscoveryService> {
        coEvery { discover(any(), any()) } returns emptyList()
    }
    private val sessionRepository = mockk<SessionRepository> {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }
    private val syncDevice = mockk<SyncDevice>(relaxed = true)
    private val setupWallet = mockk<SetupWallet>(relaxed = true)

    @Test
    fun sync_discoversAssets() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerify { discoveryService.discover("wallet-1", any()) }
    }

    @Test
    fun sync_syncsWalletConfigurationAfterSubscriptions() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerifyOrder {
            syncDevice.syncDevice()
            setupWallet.setup(wallet)
        }
    }

    private fun TestScope.service() = ImportWalletService(
        discoveryService = discoveryService,
        sessionRepository = sessionRepository,
        syncDevice = syncDevice,
        setupWallet = setupWallet,
        scope = this,
    )
}
