package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetualSubscription

class HyperliquidObserverService(
    private val observePerpetualWallet: ObservePerpetualWallet,
    private val syncPerpetualPositions: SyncPerpetualPositions,
    private val eventHandler: HyperliquidEventHandler,
    private val subscriptionService: HyperliquidSubscriptionService,
    private val connection: WebSocketConnectable,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : PerpetualObserver {

    private val foreground = MutableStateFlow(false)

    override val chartUpdates: Flow<ChartCandleUpdate> = eventHandler.chartUpdates

    init {
        scope.launch {
            combine(foreground, observePerpetualWallet()) { isForeground, wallet ->
                wallet?.takeIf { isForeground }
            }
                .distinctUntilChangedBy { it?.id?.id }
                .collectLatest { wallet ->
                    val address = wallet?.hyperliquidAccount?.address ?: return@collectLatest
                    runCatching { syncPerpetualPositions.syncPerpetualPositions() }
                    observeConnection(wallet.id, address)
                }
        }
    }

    fun start() {
        foreground.value = true
    }

    fun stop() {
        foreground.value = false
    }

    override fun subscribe(subscription: GemPerpetualSubscription) {
        subscriptionService.subscribe(subscription)
    }

    override fun unsubscribe(subscription: GemPerpetualSubscription) {
        subscriptionService.unsubscribe(subscription)
    }

    private suspend fun observeConnection(walletId: WalletId, address: String) = coroutineScope {
        launch { sendSubscriptionRequests() }
        connection.connect().collect { event ->
            when (event) {
                WebSocketEvent.Connected -> subscriptionService.resubscribe(address)
                is WebSocketEvent.Message -> eventHandler.handle(walletId, event.text)
                WebSocketEvent.Disconnected -> Unit
            }
        }
    }

    private suspend fun sendSubscriptionRequests() {
        for (request in subscriptionService.messages) {
            runCatchingCancellable { connection.send(request) }
                .onFailure { Log.e(TAG, "Subscription request error", it) }
        }
    }

    companion object {
        private const val TAG = "HyperliquidObserver"
    }
}

internal fun String.toWebSocketUrl(): String {
    val base = removeSuffix("/").replaceFirst("http", "ws")
    return if (base.endsWith("/ws")) base else "$base/ws"
}
