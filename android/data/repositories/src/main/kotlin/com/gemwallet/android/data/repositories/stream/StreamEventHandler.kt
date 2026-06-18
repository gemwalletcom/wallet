package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.support.SupportTypingState
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
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
    private val pricesDao: PricesDao,
    private val sessionRepository: SessionRepository,
    private val syncTransactions: dagger.Lazy<SyncTransactions>,
    private val syncNfts: SyncNfts,
    private val updatePriceAlerts: UpdatePriceAlerts,
    private val syncFiatTransactions: dagger.Lazy<SyncFiatTransactions>,
    private val walletsRepository: WalletsRepository,
    private val assetsDao: AssetsDao,
    private val updateBalances: UpdateBalances,
    private val inAppNotificationsDao: InAppNotificationsDao,
    private val supportMessagesDao: SupportMessagesDao,
    private val supportTypingState: SupportTypingState,
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
        val currency = sessionRepository.getCurrentCurrency()
        updateRates(payload.rates, currency)
        val rate = pricesDao.getRates(currency).firstOrNull()?.toDTO() ?: return
        pricesDao.insert(payload.prices.toRecord(rate))
    }

    private suspend fun updateRates(newRates: List<FiatRate>, currency: Currency) {
        pricesDao.setRates(newRates.toRecord())
        newRates.firstOrNull { it.symbol == currency.string }?.let { rate ->
            pricesDao.updateValues(currency.string, rate.rate)
        }
    }

    private suspend fun handleBalances(update: StreamBalanceUpdate) {
        val walletId = update.walletId.id
        val assetIds = listOf(update.assetId.toIdentifier())
        assetsDao.getAssetsInfo(walletId, assetIds)
            .toAssetInfoModel()
            .firstOrNull()
            ?.groupBy { it.asset.chain }
            ?.mapKeys { it.value.firstOrNull()?.owner }
            ?.forEach { (account, assetInfos) ->
                val owner: Account = account ?: return@forEach
                updateBalances.updateBalances(walletId, owner, assetInfos.map { it.asset })
            }
    }

    private suspend fun handleTransactions(update: StreamTransactionsUpdate) {
        val wallet = walletsRepository.getWallet(update.walletId).firstOrNull() ?: return
        syncTransactions.get().syncTransactions(wallet)
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
        inAppNotificationsDao.put(listOf(update.notification.toRecord()))
    }

    private suspend fun handleSupport(event: SupportStreamEvent) {
        when (event) {
            is SupportStreamEvent.Message -> {
                supportMessagesDao.addMessages(listOf(event.data.toRecord()))
                when (event.data.sender) {
                    is SupportMessageSender.User -> { }
                    is SupportMessageSender.Agent -> supportTypingState.clear()
                }
            }
            is SupportStreamEvent.Typing -> supportTypingState.update(event.data)
        }
    }

    companion object {
        private const val TAG = "StreamEventHandler"
    }
}
