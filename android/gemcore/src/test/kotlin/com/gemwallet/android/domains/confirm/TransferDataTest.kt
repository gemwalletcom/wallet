package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAccount
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
        val input = GemTransferData(
            inputType = GemTransactionInputType.transfer(mockAsset()),
            recipient = GemRecipient(address = "destination", memo = "memo", references = listOf("reference")),
            value = BigInteger.ONE.toString(),
        ).confirmInput(mockAccount())

        val decoded = requireNotNull(transferService.unpack(requireNotNull(transferService.pack(input))))

        assertEquals("destination", decoded.transfer.recipient.address)
        assertEquals("memo", decoded.transfer.recipient.memo)
        assertEquals(listOf("reference"), decoded.transfer.recipient.references)
        assertEquals(BigInteger.ONE.toString(), decoded.transfer.value)
    }

    @Test
    fun anInvalidRoutePayloadDecodesToNothing() {
        assertNull(transferService.unpack("invalid"))
    }
}
