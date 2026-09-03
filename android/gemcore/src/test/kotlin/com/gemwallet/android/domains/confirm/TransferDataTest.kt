package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAsset
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService
import java.math.BigInteger

class TransferDataTest {

    private val transferService = GemTransferService()

    @Test
    fun theRoutePayloadKeepsTheMemoAndReferences() {
        val transfer = GemTransferData(
            inputType = GemTransactionInputType.transfer(mockAsset()),
            recipient = GemRecipient(address = "destination", memo = "memo", references = listOf("reference")),
            value = BigInteger.ONE.toString(),
        )

        val decoded = requireNotNull(transferService.unpack(requireNotNull(transferService.pack(transfer))))

        assertEquals("destination", decoded.recipient.address)
        assertEquals("memo", decoded.recipient.memo)
        assertEquals(listOf("reference"), decoded.recipient.references)
        assertEquals(BigInteger.ONE.toString(), decoded.value)
    }

    @Test
    fun anInvalidRoutePayloadDecodesToNothing() {
        assertNull(transferService.unpack("invalid"))
    }
}
