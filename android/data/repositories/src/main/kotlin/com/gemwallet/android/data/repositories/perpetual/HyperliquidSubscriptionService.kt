package com.gemwallet.android.data.repositories.perpetual

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemSubscriptionMethod
import java.util.concurrent.ConcurrentHashMap

class HyperliquidSubscriptionService(
    private val encodeRequest: (GemSubscriptionMethod, GemPerpetualSubscription) -> String,
) {
    private val outgoing = Channel<String>(Channel.UNLIMITED)
    val messages: ReceiveChannel<String> = outgoing

    private val activeSubscriptions = ConcurrentHashMap.newKeySet<GemPerpetualSubscription>()

    fun subscribe(subscription: GemPerpetualSubscription) {
        activeSubscriptions.add(subscription)
        outgoing.trySend(encodeRequest(GemSubscriptionMethod.SUBSCRIBE, subscription))
    }

    fun unsubscribe(subscription: GemPerpetualSubscription) {
        activeSubscriptions.remove(subscription)
        outgoing.trySend(encodeRequest(GemSubscriptionMethod.UNSUBSCRIBE, subscription))
    }

    suspend fun resubscribe(address: String) {
        (defaultSubscriptions(address) + activeSubscriptions).distinct().forEach {
            outgoing.send(encodeRequest(GemSubscriptionMethod.SUBSCRIBE, it))
        }
    }

    private fun defaultSubscriptions(address: String): List<GemPerpetualSubscription> = listOf(
        GemPerpetualSubscription.AccountState(address),
        GemPerpetualSubscription.OpenOrders(address),
    )
}
