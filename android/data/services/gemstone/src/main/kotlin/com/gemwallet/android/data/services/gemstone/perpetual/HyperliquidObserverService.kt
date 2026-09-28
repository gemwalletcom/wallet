package com.gemwallet.android.data.services.gemstone.perpetual

import android.util.Log
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
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
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPerpetualStreamServiceInterface
import uniffi.gemstone.GemPerpetualSubscription

class HyperliquidObserverService(
    private val observePerpetualWallet: ObservePerpetualWallet,
    private val perpetualService: GemPerpetualServiceInterface,
    private val streamService: GemPerpetualStreamServiceInterface,
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
                    wallet ?: return@collectLatest
                    val connection = runCatchingCancellable { perpetualService.connection(wallet.toGem()) }
                        .onFailure { Log.e(TAG, "Perpetual connection failed", it) }
                        .getOrNull() ?: return@collectLatest
                    observeConnection(wallet.id, connection.address, connection.mode.toPrimitives())
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
                is WebSocketEvent.Message -> onMessage(walletId, mode, event.text)
                WebSocketEvent.Disconnected -> streamService.disconnected()
            }
        }
    }

    private suspend fun onMessage(walletId: WalletId, mode: PerpetualAccountMode, text: String) {
        runCatchingCancellable { streamService.candleUpdate(walletId.id, mode.toGem(), text.encodeToByteArray()) }
            .onSuccess { candle -> candle?.toPrimitives()?.let { chartFlow.emit(it) } }
            .onFailure { Log.e(TAG, "Socket message error: ${text.take(MESSAGE_LOG_LIMIT)}", it) }
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
