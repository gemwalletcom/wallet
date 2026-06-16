package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.ext.hasPerpetualsSupport
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemSubscriptionMethod

class HyperliquidObserverService(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
    private val syncPerpetualPositions: SyncPerpetualPositions,
    private val eventHandler: HyperliquidEventHandler,
    private val connection: WebSocketConnectable,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : PerpetualObserver {

    private val foreground = MutableStateFlow(false)
    private val commands = Channel<Pair<GemSubscriptionMethod, GemPerpetualSubscription>>(Channel.UNLIMITED)

    private val mutex = Mutex()
    private val activeSubscriptions = mutableSetOf<GemPerpetualSubscription>()

    override val chartUpdates: Flow<ChartCandleUpdate> = eventHandler.chartUpdates

    init {
        scope.launch {
            for ((method, subscription) in commands) {
                mutex.withLock {
                    when (method) {
                        GemSubscriptionMethod.SUBSCRIBE -> activeSubscriptions.add(subscription)
                        GemSubscriptionMethod.UNSUBSCRIBE -> activeSubscriptions.remove(subscription)
                    }
                }
                sendRequest(method, subscription)
            }
        }
        scope.launch {
            combine(
                foreground,
                sessionRepository.session(),
                userConfig.isPerpetualEnabled(),
            ) { isForeground, session, isEnabled ->
                session?.wallet?.takeIf { isForeground && isEnabled && it.hasPerpetualsSupport }
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
        commands.trySend(GemSubscriptionMethod.SUBSCRIBE to subscription)
    }

    override fun unsubscribe(subscription: GemPerpetualSubscription) {
        commands.trySend(GemSubscriptionMethod.UNSUBSCRIBE to subscription)
    }

    private suspend fun observeConnection(walletId: WalletId, address: String) {
        connection.connect().collect { event ->
            when (event) {
                WebSocketEvent.Connected -> subscribeAll(address)
                is WebSocketEvent.Message -> eventHandler.handle(walletId, event.text)
                WebSocketEvent.Disconnected -> Unit
            }
        }
    }

    private suspend fun subscribeAll(address: String) {
        val subscriptions = mutex.withLock {
            (defaultSubscriptions(address) + activeSubscriptions).distinct()
        }
        subscriptions.forEach { sendRequest(GemSubscriptionMethod.SUBSCRIBE, it) }
    }

    private suspend fun sendRequest(method: GemSubscriptionMethod, subscription: GemPerpetualSubscription) {
        runCatchingCancellable { connection.send(eventHandler.subscriptionRequest(method, subscription)) }
            .onFailure { Log.e(TAG, "Subscription request error", it) }
    }

    private fun defaultSubscriptions(address: String): List<GemPerpetualSubscription> = listOf(
        GemPerpetualSubscription.AccountState(address),
        GemPerpetualSubscription.OpenOrders(address),
    )

    companion object {
        private const val TAG = "HyperliquidObserver"
    }
}

internal fun String.toWebSocketUrl(): String {
    val base = trimEnd('/')
        .replaceFirst("https://", "wss://")
        .replaceFirst("http://", "ws://")
    return if (base.endsWith("/ws")) base else "$base/ws"
}
