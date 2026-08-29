package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.application.perpetual.cases.GetPerpetualAccountMode
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.perpetual.cases.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.SyncPerpetuals
import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId
import com.gemwallet.android.serializer.decodeJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPerpetualSubscription

class HyperliquidObserverService(
    private val observePerpetualWallet: ObservePerpetualWallet,
    private val syncPerpetuals: SyncPerpetuals,
    private val syncPerpetualPositions: SyncPerpetualPositions,
    private val getPerpetualAccountMode: GetPerpetualAccountMode,
    private val streamService: GemPerpetualStreamService,
    private val connection: WebSocketConnectable,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : PerpetualObserver {

    private val foreground = MutableStateFlow(false)

    private val chartFlow = MutableSharedFlow<ChartCandleUpdate>(
        extraBufferCapacity = CHART_BUFFER_CAPACITY,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    override val chartUpdates: Flow<ChartCandleUpdate> = chartFlow.asSharedFlow()

    init {
        scope.launch {
            combine(foreground, observePerpetualWallet()) { isForeground, wallet ->
                wallet?.takeIf { isForeground }
            }
                .distinctUntilChangedBy { it?.id?.id }
                .collectLatest { wallet ->
                    val address = wallet?.hyperliquidAccount?.address ?: return@collectLatest
                    val mode = syncPerpetualPositions.syncPerpetualPositions()
                        ?: getPerpetualAccountMode.getPerpetualAccountMode(wallet.id, address)
                    observeConnection(wallet.id, address, mode)
                }
        }
        scope.launch {
            observePerpetualWallet()
                .distinctUntilChangedBy { it?.id?.id }
                .collectLatest { wallet ->
                    if (wallet == null) return@collectLatest
                    runCatching { syncPerpetuals.syncPerpetuals() }
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
        scope.launch { send { streamService.subscribe(subscription) } }
    }

    override fun unsubscribe(subscription: GemPerpetualSubscription) {
        scope.launch { send { streamService.unsubscribe(subscription) } }
    }

    private suspend fun observeConnection(walletId: WalletId, address: String, mode: PerpetualAccountMode) {
        connection.connect().collect { event ->
            when (event) {
                WebSocketEvent.Connected -> send { streamService.connected(address, mode.toGem()) }
                is WebSocketEvent.Message -> handle(walletId, mode, event.text)
                WebSocketEvent.Disconnected -> streamService.disconnected()
            }
        }
    }

    private suspend fun handle(walletId: WalletId, mode: PerpetualAccountMode, text: String) {
        runCatchingCancellable { streamService.handle(walletId.id, mode.toGem(), text.encodeToByteArray()) }
            .onSuccess { candle -> candle?.decodeJson<ChartCandleUpdate>()?.let { chartFlow.emit(it) } }
            .onFailure { Log.e(TAG, "Handle message error: ${text.take(MESSAGE_LOG_LIMIT)}", it) }
    }

    private suspend fun send(request: suspend () -> Unit) {
        runCatchingCancellable(request).onFailure { Log.e(TAG, "Subscription request error", it) }
    }

    companion object {
        private const val TAG = "HyperliquidObserver"
        private const val CHART_BUFFER_CAPACITY = 64
        private const val MESSAGE_LOG_LIMIT = 100
    }
}

internal fun String.toWebSocketUrl(): String {
    val base = removeSuffix("/").replaceFirst("http", "ws")
    return if (base.endsWith("/ws")) base else "$base/ws"
}
