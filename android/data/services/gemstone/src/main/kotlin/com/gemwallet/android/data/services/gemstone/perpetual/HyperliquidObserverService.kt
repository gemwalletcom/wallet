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
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.produceIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPerpetualStreamServiceInterface
import uniffi.gemstone.GemPerpetualSubscription

@OptIn(ExperimentalCoroutinesApi::class)
class HyperliquidObserverService(
    private val observePerpetualWallet: ObservePerpetualWallet,
    private val perpetualService: GemPerpetualServiceInterface,
    private val streamService: GemPerpetualStreamServiceInterface,
    private val connection: WebSocketConnectable,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : PerpetualObserver {

    private val foreground = MutableStateFlow(false)

    private val chartFlow = MutableSharedFlow<ChartCandleUpdate>(
        extraBufferCapacity = CHART_BUFFER_CAPACITY,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    override val chartUpdates: Flow<ChartCandleUpdate> = chartFlow.asSharedFlow()

    init {
        scope.launch {
            foreground
                .flatMapLatest { isForeground -> if (isForeground) observePerpetualWallet() else flowOf(null) }
                .distinctUntilChangedBy { it?.id?.id }
                .collectLatest { wallet ->
                    wallet ?: return@collectLatest
                    coroutineScope {
                        val events = connection.connect().produceIn(this)
                        val account = runCatchingCancellable { perpetualService.connection(wallet.toGem()) }
                            .onFailure { Log.e(TAG, "Perpetual connection failed", it) }
                            .getOrNull()
                        if (account == null) {
                            events.cancel()
                            return@coroutineScope
                        }
                        observeConnection(events, wallet.id, account.address, account.mode.toPrimitives())
                    }
                }
        }
    }

    fun start() {
        foreground.value = true
    }

    fun stop() {
        foreground.value = false
    }

    override suspend fun subscribe(subscription: GemPerpetualSubscription) {
        send { streamService.subscribe(subscription) }
    }

    override suspend fun unsubscribe(subscription: GemPerpetualSubscription) {
        send { streamService.unsubscribe(subscription) }
    }

    private suspend fun observeConnection(events: ReceiveChannel<WebSocketEvent>, walletId: WalletId, address: String, mode: PerpetualAccountMode) {
        try {
            for (event in events) {
                when (event) {
                    WebSocketEvent.Connected -> send { streamService.connected(address, mode.toGem()) }
                    is WebSocketEvent.Message -> onMessage(walletId, mode, event.text)
                    WebSocketEvent.Disconnected -> streamService.disconnected()
                }
            }
        } finally {
            withContext(NonCancellable) { streamService.disconnected() }
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
