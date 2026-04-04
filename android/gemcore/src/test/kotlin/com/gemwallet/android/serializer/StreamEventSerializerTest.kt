package com.gemwallet.android.serializer

import com.wallet.core.primitives.StreamEvent
import org.junit.Assert.assertTrue
import org.junit.Test

class StreamEventSerializerTest {

    @Test
    fun `transforms event key to type and passes through type key`() {
        val withEventKey = """{"event": "nft", "data": {"walletId": "w1"}}"""
        val withTypeKey = """{"type": "nft", "data": {"walletId": "w1"}}"""

        assertTrue(jsonEncoder.decodeFromString(StreamEventSerializer, withEventKey) is StreamEvent.Nft)
        assertTrue(jsonEncoder.decodeFromString(StreamEventSerializer, withTypeKey) is StreamEvent.Nft)
    }
}
