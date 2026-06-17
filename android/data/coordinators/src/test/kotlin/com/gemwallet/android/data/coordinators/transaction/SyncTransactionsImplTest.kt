package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.TransactionsResponse
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncTransactionsImplTest {

    private val walletPreferences = mockk<WalletPreferences>(relaxed = true) {
        every { transactionsTimestamp } returns 42L
    }
    private val walletPreferencesFactory = mockk<WalletPreferencesFactory> {
        every { create(any()) } returns walletPreferences
    }
    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val saveTransactions = mockk<SaveTransactions>(relaxed = true)
    private val saveAddressNames = mockk<SaveAddressNames>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = SyncTransactionsImpl(
        walletPreferencesFactory = walletPreferencesFactory,
        gemDeviceApiClient = gemDeviceApiClient,
        saveTransactions = saveTransactions,
        saveAddressNames = saveAddressNames,
        prefetchAssets = prefetchAssets,
        assetsRepository = assetsRepository,
        sessionRepository = sessionRepository,
    )

    @Test
    fun syncTransactions_prefetchesAssetsAndSavesTransactions() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val solana = mockAssetSolana()
        val ethereum = mockAssetEthereum()
        val transaction = mockTransaction(
            assetId = solana.id,
            feeAssetId = ethereum.id,
        )

        coEvery {
            gemDeviceApiClient.getTransactions(wallet.id, 42L)
        } returns TransactionsResponse(
            transactions = listOf(transaction),
            addressNames = emptyList(),
        )
        coEvery { prefetchAssets.prefetchAssets(listOf(solana.id, ethereum.id)) } returns listOf(solana.id)

        subject.syncTransactions(wallet)

        coVerify { prefetchAssets.prefetchAssets(listOf(solana.id, ethereum.id)) }
        coVerify { assetsRepository.addBalancesIfMissing(wallet.id, listOf(solana.id)) }
        coVerify { saveTransactions.saveTransactions(wallet.id, listOf(transaction)) }
        coVerify { saveAddressNames.saveAddressNames(emptyList()) }
        verify { walletPreferences.transactionsTimestamp = any() }
    }
}
