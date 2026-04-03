package com.gemwallet.android.data.repositories.stream

import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.cases.nft.LoadNFTCase
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.StreamEvent
import com.wallet.core.primitives.StreamPriceAlertUpdate
import com.wallet.core.primitives.StreamTransactionsUpdate
import com.wallet.core.primitives.StreamWalletUpdate
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Test

class StreamEventHandlerTest {

    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)
    private val syncTransactions = mockk<dagger.Lazy<SyncTransactions>>()
    private val loadNFTCase = mockk<LoadNFTCase>(relaxed = true)
    private val updatePriceAlerts = mockk<UpdatePriceAlerts>(relaxed = true)
    private val buyRepository = mockk<dagger.Lazy<BuyRepository>>()
    private val walletsRepository = mockk<WalletsRepository>()
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val updateBalances = mockk<UpdateBalances>(relaxed = true)

    private val handler = StreamEventHandler(
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
        syncTransactions = syncTransactions,
        loadNFTCase = loadNFTCase,
        updatePriceAlerts = updatePriceAlerts,
        buyRepository = buyRepository,
        walletsRepository = walletsRepository,
        assetsDao = assetsDao,
        updateBalances = updateBalances,
    )

    private val wallet = mockWallet(id = "w1")

    @Test
    fun `dispatches events to correct services`() = runBlocking {
        val sync = mockk<SyncTransactions>(relaxed = true)
        every { syncTransactions.get() } returns sync
        val buyRepo = mockk<BuyRepository>(relaxed = true)
        every { buyRepository.get() } returns buyRepo
        coEvery { walletsRepository.getWallet("w1") } returns flowOf(wallet)

        handler.handle(StreamEvent.Transactions(StreamTransactionsUpdate(walletId = WalletId("w1"), transactions = listOf("tx1"))))
        coVerify { sync.syncTransactions(wallet) }

        handler.handle(StreamEvent.PriceAlerts(StreamPriceAlertUpdate(assets = emptyList())))
        coVerify { updatePriceAlerts.update() }

        handler.handle(StreamEvent.Nft(StreamWalletUpdate(walletId = WalletId("w1"))))
        coVerify { loadNFTCase.loadNFT(wallet) }

        handler.handle(StreamEvent.FiatTransaction(StreamWalletUpdate(walletId = WalletId("w1"))))
        coVerify { buyRepo.updateFiatTransactions(wallet) }

        handler.handle(StreamEvent.Perpetual(StreamWalletUpdate(walletId = WalletId("w1"))))
    }

    @Test
    fun `unknown wallet does not call service`() = runBlocking {
        val sync = mockk<SyncTransactions>(relaxed = true)
        every { syncTransactions.get() } returns sync
        coEvery { walletsRepository.getWallet("unknown") } returns flowOf(null)

        handler.handle(StreamEvent.Transactions(StreamTransactionsUpdate(walletId = WalletId("unknown"), transactions = emptyList())))

        coVerify(exactly = 0) { sync.syncTransactions(any()) }
    }
}
