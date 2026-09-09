package com.gemwallet.android.serializer

import com.wallet.core.primitives.StreamEvent
import com.wallet.core.primitives.StreamWalletUpdate
import com.gemwallet.android.testkit.mockWalletId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TaggedJsonBridgeTest {

    private val nft = StreamEvent.Nft(StreamWalletUpdate(mockWalletId("wallet-1")))
    private val perpetual = StreamEvent.Perpetual(StreamWalletUpdate(mockWalletId("wallet-2")))

    @Test
    fun `a variant built inline keeps its discriminator`() {
        for (event in listOf(nft, perpetual)) {
            val json = event.toJson()

            assertTrue("Core cannot lift a payload without a discriminator: $json", json.contains("\"type\""))
        }
    }

    @Test
    fun `a variant built inline round trips`() {
        val decoded = nft.toJson().decodeJson<StreamEvent>()

        assertEquals(nft, decoded)
    }

    @Test
    fun `widening by hand and letting the overload widen agree`() {
        val event: StreamEvent = perpetual

        assertEquals(event.toJson(), StreamEvent.Perpetual(StreamWalletUpdate(mockWalletId("wallet-2"))).toJson())
    }
}
