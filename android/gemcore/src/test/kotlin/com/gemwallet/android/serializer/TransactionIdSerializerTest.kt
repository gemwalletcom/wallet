package com.gemwallet.android.serializer

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Test

class TransactionIdSerializerTest {

    @Test
    fun `serializes and deserializes TransactionId as string`() {
        val transactionId = TransactionId(chain = Chain.Ethereum, hash = "0xabc123")
        val json = jsonEncoder.encodeToString(transactionId)

        assertEquals("\"ethereum_0xabc123\"", json)

        val decoded = jsonEncoder.decodeFromString<TransactionId>(json)
        assertEquals(transactionId, decoded)
    }

    @Test
    fun `deserializes TransactionId from object payload`() {
        val json = """{"chain":"solana","hash":"solhash456"}"""

        val decoded = jsonEncoder.decodeFromString<TransactionId>(json)

        assertEquals(TransactionId(chain = Chain.Solana, hash = "solhash456"), decoded)
    }
}
