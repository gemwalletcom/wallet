package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransaction
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.FiatTransactionStatus
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncFiatTransactionsImplTest {

    private val walletId = WalletId("wallet-1")
    private val sessionRepository = mockk<SessionRepository>()
    private val getFiatTransactions = mockk<GetFiatTransactions>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)
    private val fiatTransactionsDao = mockk<FiatTransactionsDao>(relaxed = true)

    private val subject = SyncFiatTransactionsImpl(
        sessionRepository = sessionRepository,
        getFiatTransactions = getFiatTransactions,
        prefetchAssets = prefetchAssets,
        fiatTransactionsDao = fiatTransactionsDao,
    )

    @Test
    fun syncFiatTransactions_prefetchesDistinctAssetIds() = runTest {
        val ethereum = mockAssetEthereum()
        val transaction = fiatTransaction(assetId = ethereum.id)

        coEvery { getFiatTransactions(walletId) } returns listOf(transaction, transaction)

        subject(walletId)

        coVerify { prefetchAssets.prefetchAssets(listOf(ethereum.id)) }
        verify { fiatTransactionsDao.insert(any()) }
    }

    @Test
    fun syncFiatTransactions_withoutSession_skipsWork() = runTest {
        every { sessionRepository.session() } returns MutableStateFlow(null)

        subject()

        coVerify(exactly = 0) { getFiatTransactions(any()) }
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        verify(exactly = 0) { fiatTransactionsDao.insert(any()) }
    }

    @Test
    fun syncFiatTransactions_usesCurrentSessionWallet() = runTest {
        val wallet = mockWallet(id = walletId.id)
        every { sessionRepository.session() } returns MutableStateFlow(mockSession(wallet = wallet))
        coEvery { getFiatTransactions(walletId) } returns emptyList()

        subject()

        coVerify { getFiatTransactions(walletId) }
        coVerify { prefetchAssets.prefetchAssets(emptyList()) }
        verify { fiatTransactionsDao.insert(any()) }
    }

    private fun fiatTransaction(assetId: com.wallet.core.primitives.AssetId) = FiatTransactionData(
        transaction = FiatTransaction(
            id = "tx-1",
            assetId = assetId,
            transactionType = FiatQuoteType.Buy,
            provider = FiatProviderName.MoonPay,
            status = FiatTransactionStatus.Pending,
            fiatAmount = 100.0,
            fiatCurrency = "USD",
            value = "1000000000000000000",
            createdAt = 1,
        ),
    )
}
