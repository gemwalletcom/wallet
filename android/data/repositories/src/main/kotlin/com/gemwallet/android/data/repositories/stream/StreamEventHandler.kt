package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
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
import com.wallet.core.primitives.StreamTransactionsUpdate
import com.wallet.core.primitives.StreamWalletUpdate
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
) {

    suspend fun handle(event: StreamEvent) {
        when (event) {
            is StreamEvent.Prices -> perform { handlePrices(event.data) }
            is StreamEvent.Balances -> perform { handleBalances(event.data) }
            is StreamEvent.Transactions -> perform { handleTransactions(event.data) }
            is StreamEvent.PriceAlerts -> perform { handlePriceAlerts() }
            is StreamEvent.Nft -> perform { handleNft(event.data) }
            is StreamEvent.Perpetual -> { }
            is StreamEvent.InAppNotification -> { }
            is StreamEvent.FiatTransaction -> perform { handleFiatTransaction(event.data) }
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
            pricesDao.getAll().firstOrNull()?.map {
                it.copy(value = (it.usdValue ?: 0.0) * rate.rate)
            }?.let { pricesDao.insert(it) }
        }
    }

    private suspend fun handleBalances(update: StreamBalanceUpdate) {
        val assetIds = listOf(update.assetId.toIdentifier())
        assetsDao.getAssetsInfo(assetIds)
            .toAssetInfoModel()
            .firstOrNull()
            ?.groupBy { it.walletId }
            ?.forEach { (walletId, assets) ->
                walletId ?: return@forEach
                assets.groupBy { it.asset.chain }
                    .mapKeys { it.value.firstOrNull()?.owner }
                    .forEach { (account, assetInfos) ->
                        val owner: Account = account ?: return@forEach
                        updateBalances.updateBalances(
                            walletId, owner, assetInfos.map { it.asset }
                        )
                    }
            }
    }

    private suspend fun handleTransactions(update: StreamTransactionsUpdate) {
        val wallet = walletsRepository.getWallet(update.walletId.id).firstOrNull() ?: return
        syncTransactions.get().syncTransactions(wallet)
    }

    private suspend fun handlePriceAlerts() {
        updatePriceAlerts.update()
    }

    private suspend fun handleNft(update: StreamWalletUpdate) {
        val wallet = walletsRepository.getWallet(update.walletId.id).firstOrNull() ?: return
        syncNfts.syncNfts(wallet)
    }

    private suspend fun handleFiatTransaction(update: StreamWalletUpdate) {
        syncFiatTransactions.get()(update.walletId)
    }

    companion object {
        private const val TAG = "StreamEventHandler"
    }
}
