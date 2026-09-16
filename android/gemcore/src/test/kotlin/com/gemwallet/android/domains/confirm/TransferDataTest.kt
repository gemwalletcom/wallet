package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockGemTransferData
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemRecipient
import java.math.BigInteger

class TransferDataTest {

    @Test
    fun theRoutePayloadKeepsTheMemoAndReferences() {
        val transfer = mockGemTransferData(
            recipient = GemRecipient(address = "destination", memo = "memo", references = listOf("reference")),
        )

        val decoded = requireNotNull(unpackTransferData(requireNotNull(transfer.pack())))

        assertEquals("destination", decoded.recipient.address)
        assertEquals("memo", decoded.recipient.memo)
        assertEquals(listOf("reference"), decoded.recipient.references)
        assertEquals(BigInteger.ONE, decoded.value)
    }

    @Test
    fun anInvalidRoutePayloadDecodesToNothing() {
        assertNull(unpackTransferData("invalid"))
    }
}
