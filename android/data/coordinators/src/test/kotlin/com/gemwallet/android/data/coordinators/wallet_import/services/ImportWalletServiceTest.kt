package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.nft.SyncNfts
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

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletServiceTest {

    private val discoveryService = mockk<GemAssetDiscoveryService> {
        coEvery { discover(any()) } returns emptyList()
    }
    private val syncDevice = mockk<SyncDevice>(relaxed = true)
    private val syncTransactions = mockk<SyncTransactions>(relaxed = true)
    private val syncNfts = mockk<SyncNfts>(relaxed = true)
    private val walletConfigurationSync = mockk<SyncWalletConfiguration>(relaxed = true)

    @Test
    fun sync_discoversAssetsTransactionsAndNfts() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerify { discoveryService.discover("wallet-1") }
        coVerify { syncTransactions.syncTransactions(wallet) }
        coVerify { syncNfts.sync(wallet.id) }
    }

    @Test
    fun sync_syncsWalletConfigurationAfterSubscriptions() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        subject.sync(wallet)
        advanceUntilIdle()

        coVerifyOrder {
            syncDevice.syncDevice()
            walletConfigurationSync.sync(wallet.id)
        }
    }

    private fun TestScope.service() = ImportWalletService(
        discoveryService = discoveryService,
        syncDevice = syncDevice,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        walletConfigurationSync = walletConfigurationSync,
        scope = this,
    )
}
