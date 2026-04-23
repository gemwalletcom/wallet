package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.model.TransactionsResponse
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncTransactionsImplTest {

    private val walletPreferencesFactory = mockk<WalletPreferencesFactory>()
    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val saveTransactions = mockk<SaveTransactions>()
    private val saveAddressNames = mockk<SaveAddressNames>()
    private val prefetchAssets = mockk<PrefetchAssets>()
    private val ensureWalletAssets = mockk<EnsureWalletAssets>()
    private val preferences = mockk<WalletPreferences>(relaxed = true)

    private val subject = SyncTransactionsImpl(
        walletPreferencesFactory = walletPreferencesFactory,
        gemDeviceApiClient = gemDeviceApiClient,
        saveTransactions = saveTransactions,
        saveAddressNames = saveAddressNames,
        prefetchAssets = prefetchAssets,
        ensureWalletAssets = ensureWalletAssets,
    )

    @Test
    fun syncTransactions_persistsAddressNamesFromResponse() = runTest {
        val wallet = mockWallet()
        val addressNames = listOf(
            AddressName(
                chain = Chain.Bitcoin,
                address = "validator-address",
                name = "Validator",
                type = null,
                status = VerificationStatus.Verified,
            )
        )
        val response = TransactionsResponse(
            transactions = listOf(mockTransaction()),
            addressNames = addressNames,
        )

        every { walletPreferencesFactory.create(wallet.id) } returns preferences
        every { preferences.transactionsTimestamp } returns 0L
        coEvery { gemDeviceApiClient.getTransactions(wallet.id, 0L) } returns response
        coEvery { prefetchAssets.prefetchAssets(any()) } just Runs
        coEvery { ensureWalletAssets.ensureWalletAssets(any(), any()) } just Runs
        coEvery { saveTransactions.saveTransactions(any(), any()) } just Runs
        coEvery { saveAddressNames.saveAddressNames(any()) } just Runs

        subject.syncTransactions(wallet)

        coVerify(exactly = 1) { saveTransactions.saveTransactions(wallet.id, response.transactions) }
        coVerify(exactly = 1) { saveAddressNames.saveAddressNames(addressNames) }
    }
}
