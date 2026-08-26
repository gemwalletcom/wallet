package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PerpetualAccountMode
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.HyperliquidSubscriptions

class HyperliquidSubscriptionServiceTest {

    @Test
    fun `requests reach the outgoing channel only once connected`() = runTest {
        val service = HyperliquidSubscriptionService(HyperliquidSubscriptions())

        service.subscribe(GemPerpetualSubscription.MarketPrices)
        assertTrue(service.messages.tryReceive().isFailure)

        service.connected(ADDRESS, PerpetualAccountMode.Standard.toJson())

        val sent = generateSequence { service.messages.tryReceive().getOrNull() }.toList()
        assertEquals(
            setOf(
                """{"method":"subscribe","subscription":{"type":"clearinghouseState","user":"0xabc"}}""",
                """{"method":"subscribe","subscription":{"type":"openOrders","user":"0xabc"}}""",
                """{"method":"subscribe","subscription":{"type":"allMids"}}""",
            ),
            sent.toSet(),
        )
        assertEquals(3, sent.size)
    }

    private companion object {
        const val ADDRESS = "0xabc"
    }
}
