package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.data.repositories.assets.visibleByDefault
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.StreamMessage
import com.wallet.core.primitives.StreamMessagePrices
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class StreamSubscriptionService(
    private val assetsDao: AssetsDao,
    private val priceAlertsDao: PriceAlertsDao,
) {
    private val outgoing = Channel<StreamMessage>(Channel.UNLIMITED)
    val messages: ReceiveChannel<StreamMessage> = outgoing

    private val mutex = Mutex()
    private val subscribedAssetIds = mutableSetOf<AssetId>()
    private var currentWalletId: String? = null

    suspend fun setupAssets(walletId: WalletId) {
        currentWalletId = walletId.id
        resubscribe()
    }

    suspend fun resubscribe() {
        val walletId = currentWalletId ?: return
        try {
            val assets = observableAssets(walletId)
            mutex.withLock {
                subscribedAssetIds.clear()
                subscribedAssetIds.addAll(assets)
                outgoing.send(StreamMessage.SubscribePrices(StreamMessagePrices(assets)))
            }
        } catch (err: Throwable) {
            Log.e(TAG, "Resubscribe error", err)
        }
    }

    suspend fun addAssetIds(ids: List<AssetId>) {
        mutex.withLock {
            val newIds = ids.filter { subscribedAssetIds.add(it) }
            if (newIds.isNotEmpty()) {
                outgoing.send(StreamMessage.AddPrices(StreamMessagePrices(newIds)))
            }
        }
    }

    private suspend fun observableAssets(walletId: String): List<AssetId> {
        val ids = assetsDao.getAssetsPriceUpdate(walletId).mapNotNull { it.toAssetId() }
        val priceAlerts = priceAlertsDao.getAlerts().firstOrNull()
            ?.mapNotNull { it.assetId.toAssetId() } ?: emptyList()
        return (ids + priceAlerts).takeIf { it.isNotEmpty() }
            ?: visibleByDefault.map { AssetId(it) }
    }

    companion object {
        private const val TAG = "StreamSubscriptionService"
    }
}
