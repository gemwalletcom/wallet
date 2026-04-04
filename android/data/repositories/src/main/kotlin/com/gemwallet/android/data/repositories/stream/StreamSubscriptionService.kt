package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.data.repositories.assets.visibleByDefault
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.StreamMessage
import com.wallet.core.primitives.StreamMessagePrices
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch

class StreamSubscriptionService(
    private val assetsDao: AssetsDao,
    private val priceAlertsDao: PriceAlertsDao,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private val _messageFlow = MutableSharedFlow<StreamMessage>()
    val messageFlow: SharedFlow<StreamMessage> = _messageFlow

    private val subscribedAssetIds = mutableSetOf<AssetId>()
    private var currentWalletId: String? = null

    fun setupAssets(walletId: String) {
        currentWalletId = walletId
        resubscribe()
    }

    fun resubscribe() = scope.launch {
        val walletId = currentWalletId ?: return@launch
        try {
            val assets = observableAssets(walletId)
            subscribedAssetIds.clear()
            subscribedAssetIds.addAll(assets)
            _messageFlow.emit(
                StreamMessage.SubscribePrices(data = StreamMessagePrices(assets = assets))
            )
        } catch (err: Throwable) {
            Log.e(TAG, "Resubscribe error", err)
        }
    }

    private suspend fun observableAssets(walletId: String): List<AssetId> {
        val ids = assetsDao.getAssetsPriceUpdate(walletId).mapNotNull { it.toAssetId() }
        val priceAlerts = priceAlertsDao.getAlerts().firstOrNull()
            ?.mapNotNull { it.assetId.toAssetId() } ?: emptyList()
        return (ids + priceAlerts).takeIf { it.isNotEmpty() }
            ?: visibleByDefault.map { AssetId(it) }
    }

    fun addAssetIds(ids: List<AssetId>) = scope.launch {
        val newIds = ids.filter { subscribedAssetIds.add(it) }
        if (newIds.isNotEmpty()) {
            _messageFlow.emit(
                StreamMessage.AddPrices(data = StreamMessagePrices(assets = newIds))
            )
        }
    }

    companion object {
        private const val TAG = "StreamSubscriptionService"
    }
}
