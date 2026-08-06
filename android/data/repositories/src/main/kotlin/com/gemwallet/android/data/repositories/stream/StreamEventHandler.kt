package com.gemwallet.android.data.repositories.stream

import android.util.Log
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
import com.wallet.core.primitives.StreamBalanceUpdate
import com.wallet.core.primitives.StreamEvent
import com.wallet.core.primitives.StreamNotificationUpdate
import com.wallet.core.primitives.StreamTransactionsUpdate
import com.wallet.core.primitives.StreamWalletUpdate
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportStreamEvent
import com.wallet.core.primitives.WebSocketPricePayload
import kotlinx.coroutines.flow.firstOrNull

class StreamEventHandler(
    private val pricesRepository: PricesRepository,
    private val syncTransactions: dagger.Lazy<SyncTransactions>,
    private val syncNfts: SyncNfts,
    private val updatePriceAlerts: UpdatePriceAlerts,
    private val syncFiatTransactions: dagger.Lazy<SyncFiatTransactions>,
    private val walletsRepository: WalletsRepository,
    private val updateBalances: UpdateBalances,
    private val inAppNotificationsRepository: InAppNotificationsRepository,
    private val supportChatRepository: SupportChatRepository,
) {

    suspend fun handle(event: StreamEvent) {
        when (event) {
            is StreamEvent.Prices -> perform { handlePrices(event.data) }
            is StreamEvent.Balances -> perform { handleBalances(event.data) }
            is StreamEvent.Transactions -> perform { handleTransactions(event.data) }
            is StreamEvent.PriceAlerts -> perform { handlePriceAlerts() }
            is StreamEvent.Nft -> perform { handleNft(event.data) }
            is StreamEvent.Perpetual -> { }
            is StreamEvent.InAppNotification -> perform { handleInAppNotification(event.data) }
            is StreamEvent.FiatTransaction -> perform { handleFiatTransaction(event.data) }
            is StreamEvent.Support -> perform { handleSupport(event.data) }
        }
    }

    private suspend fun perform(block: suspend () -> Unit) {
        try {
            block()
        } catch (err: Throwable) {
            Log.e(TAG, "Event handler error", err)
        }
    }

    private suspend fun handlePrices(payload: WebSocketPricePayload) {
        pricesRepository.updatePrices(payload)
    }

    private suspend fun handleBalances(update: StreamBalanceUpdate) {
        updateBalances.updateBalances(update.walletId.id, update.assetIds.map { it.toIdentifier() })
    }

    private suspend fun handleTransactions(update: StreamTransactionsUpdate) {
        val wallet = walletsRepository.getWallet(update.walletId).firstOrNull() ?: return
        syncTransactions.get().syncTransactions(wallet)
        updateBalances.updateBalances(update.walletId.id, update.assetIds.map { it.toIdentifier() })
    }

    private suspend fun handlePriceAlerts() {
        updatePriceAlerts.update()
    }

    private suspend fun handleNft(update: StreamWalletUpdate) {
        syncNfts.sync(update.walletId)
    }

    private suspend fun handleFiatTransaction(update: StreamWalletUpdate) {
        syncFiatTransactions.get()(update.walletId)
    }

    private suspend fun handleInAppNotification(update: StreamNotificationUpdate) {
        inAppNotificationsRepository.addNotification(update.notification)
    }

    private suspend fun handleSupport(event: SupportStreamEvent) {
        when (event) {
            is SupportStreamEvent.Message -> {
                supportChatRepository.addMessages(listOf(event.data))
                when (event.data.sender) {
                    is SupportMessageSender.User -> { }
                    is SupportMessageSender.Agent -> supportChatRepository.clearTyping()
                }
            }
            is SupportStreamEvent.Typing -> supportChatRepository.updateTyping(event.data)
        }
    }

    companion object {
        private const val TAG = "StreamEventHandler"
    }
}
