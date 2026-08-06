package com.gemwallet.android.data.repositories.stream

import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.CoreListItem
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.StreamEvent
import com.wallet.core.primitives.StreamNotificationUpdate
import com.wallet.core.primitives.StreamPriceAlertUpdate
import com.wallet.core.primitives.StreamTransactionsUpdate
import com.wallet.core.primitives.StreamWalletUpdate
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class StreamEventHandlerTest {

    private val pricesRepository = mockk<PricesRepository>(relaxed = true)
    private val syncTransactions = mockk<dagger.Lazy<SyncTransactions>>()
    private val syncNfts = mockk<SyncNfts>(relaxed = true)
    private val updatePriceAlerts = mockk<UpdatePriceAlerts>(relaxed = true)
    private val syncFiatTransactions = mockk<dagger.Lazy<SyncFiatTransactions>>()
    private val walletsRepository = mockk<WalletsRepository>()
    private val updateBalances = mockk<UpdateBalances>(relaxed = true)
    private val inAppNotificationsRepository = mockk<InAppNotificationsRepository>(relaxed = true)
    private val supportChatRepository = mockk<SupportChatRepository>(relaxed = true)

    private val handler = StreamEventHandler(
        pricesRepository = pricesRepository,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        updatePriceAlerts = updatePriceAlerts,
        syncFiatTransactions = syncFiatTransactions,
        walletsRepository = walletsRepository,
        updateBalances = updateBalances,
        inAppNotificationsRepository = inAppNotificationsRepository,
        supportChatRepository = supportChatRepository,
    )

    private val walletId = mockWalletId("w1")
    private val wallet = mockWallet(id = "w1")

    @Test
    fun `transactions event syncs wallet transactions`() = runTest {
        val sync = mockk<SyncTransactions>(relaxed = true)
        val assetId = mockAssetId()
        every { syncTransactions.get() } returns sync
        coEvery { walletsRepository.getWallet(walletId) } returns flowOf(wallet)

        handler.handle(
            StreamEvent.Transactions(
                StreamTransactionsUpdate(
                    walletId = walletId,
                    transactions = listOf(mockTransactionId(Chain.Bitcoin, "tx1")),
                    assetIds = listOf(assetId),
                )
            )
        )

        coVerify { sync.syncTransactions(wallet) }
        coVerify { updateBalances.updateBalances(walletId.id, listOf(assetId.toIdentifier())) }
    }

    @Test
    fun `price alerts event updates alerts`() = runTest {
        handler.handle(StreamEvent.PriceAlerts(StreamPriceAlertUpdate(assets = emptyList())))

        coVerify { updatePriceAlerts.update() }
    }

    @Test
    fun `nft event syncs wallet nfts`() = runTest {
        handler.handle(StreamEvent.Nft(StreamWalletUpdate(walletId = walletId)))

        coVerify { syncNfts.sync(walletId) }
        coVerify(exactly = 0) { walletsRepository.getWallet(any()) }
    }

    @Test
    fun `fiat transaction event syncs by wallet id`() = runTest {
        val syncFiat = mockk<SyncFiatTransactions>(relaxed = true)
        every { syncFiatTransactions.get() } returns syncFiat

        handler.handle(StreamEvent.FiatTransaction(StreamWalletUpdate(walletId = walletId)))

        coVerify { syncFiat(walletId) }
        coVerify(exactly = 0) { walletsRepository.getWallet(any()) }
    }

    @Test
    fun `in-app notification event stores notification`() = runTest {
        val notification = InAppNotification(
            walletId = walletId,
            readAt = null,
            createdAt = 1_000L,
            item = CoreListItem(id = "n1", title = "Title"),
        )

        handler.handle(
            StreamEvent.InAppNotification(
                StreamNotificationUpdate(walletId = walletId, notification = notification)
            )
        )

        coVerify { inAppNotificationsRepository.addNotification(notification) }
    }

    @Test
    fun `unknown wallet does not call service`() = runTest {
        val sync = mockk<SyncTransactions>(relaxed = true)
        every { syncTransactions.get() } returns sync
        coEvery { walletsRepository.getWallet(mockWalletId("unknown")) } returns flowOf(null)

        handler.handle(
            StreamEvent.Transactions(
                StreamTransactionsUpdate(
                    walletId = mockWalletId("unknown"),
                    transactions = listOf(mockTransactionId()),
                    assetIds = emptyList(),
                )
            )
        )

        coVerify(exactly = 0) { sync.syncTransactions(any()) }
    }
}
