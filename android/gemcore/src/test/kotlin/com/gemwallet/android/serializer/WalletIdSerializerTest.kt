package com.gemwallet.android.serializer

import com.wallet.core.primitives.WalletId
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletIdSerializerTest {

    @Test
    fun `serializes and deserializes WalletId as string`() {
        val walletId = WalletId(id = "test-wallet-123")
        val json = jsonEncoder.encodeToString(walletId)

        assertEquals("\"test-wallet-123\"", json)

        val decoded = jsonEncoder.decodeFromString<WalletId>(json)
        assertEquals(walletId, decoded)
    }
}
