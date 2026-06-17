package com.gemwallet.android.data.repositories.perpetual

import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemSubscriptionMethod

class HyperliquidSubscriptionServiceTest {

    @Test
    fun `subscribe and unsubscribe encode matching commands`() = runTest {
        val encoded = mutableListOf<Pair<GemSubscriptionMethod, GemPerpetualSubscription>>()
        val service = HyperliquidSubscriptionService { method, subscription ->
            encoded += method to subscription
            REQUEST
        }

        service.subscribe(GemPerpetualSubscription.MarketPrices)
        assertEquals(REQUEST, service.messages.receive())

        service.unsubscribe(GemPerpetualSubscription.MarketPrices)
        assertEquals(REQUEST, service.messages.receive())

        assertEquals(
            listOf(
                GemSubscriptionMethod.SUBSCRIBE to GemPerpetualSubscription.MarketPrices,
                GemSubscriptionMethod.UNSUBSCRIBE to GemPerpetualSubscription.MarketPrices,
            ),
            encoded,
        )
    }

    @Test
    fun `resubscribe replays default subscriptions and the active set`() = runTest {
        val encoded = mutableListOf<Pair<GemSubscriptionMethod, GemPerpetualSubscription>>()
        val service = HyperliquidSubscriptionService { method, subscription ->
            encoded += method to subscription
            REQUEST
        }
        service.subscribe(GemPerpetualSubscription.MarketPrices)
        service.messages.receive() // drain the subscribe command
        encoded.clear()

        service.resubscribe(ADDRESS)
        repeat(3) { service.messages.receive() }

        assertEquals(
            setOf(
                GemSubscriptionMethod.SUBSCRIBE to GemPerpetualSubscription.AccountState(ADDRESS),
                GemSubscriptionMethod.SUBSCRIBE to GemPerpetualSubscription.OpenOrders(ADDRESS),
                GemSubscriptionMethod.SUBSCRIBE to GemPerpetualSubscription.MarketPrices,
            ),
            encoded.toSet(),
        )
    }

    private companion object {
        const val ADDRESS = "0xabc"
        const val REQUEST = "request"
    }
}
