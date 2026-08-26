package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import uniffi.gemstone.PerpetualAccountMode as GemPerpetualAccountMode
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.HyperliquidSubscriptions

class HyperliquidSubscriptionService(
    private val subscriptions: HyperliquidSubscriptions,
) {
    private val outgoing = Channel<String>(Channel.UNLIMITED)
    val messages: ReceiveChannel<String> = outgoing

    fun subscribe(subscription: GemPerpetualSubscription) {
        send { subscriptions.subscribe(subscription) }
    }

    fun unsubscribe(subscription: GemPerpetualSubscription) {
        send { subscriptions.unsubscribe(subscription) }
    }

    fun connected(address: String, mode: GemPerpetualAccountMode) {
        send { subscriptions.connected(address, mode) }
    }

    fun disconnected() {
        subscriptions.disconnected()
    }

    private fun send(requests: () -> List<String>) {
        runCatching(requests)
            .onSuccess { it.forEach(outgoing::trySend) }
            .onFailure { Log.e(TAG, "Subscription request error", it) }
    }

    companion object {
        private const val TAG = "HyperliquidSubscriptions"
    }
}
