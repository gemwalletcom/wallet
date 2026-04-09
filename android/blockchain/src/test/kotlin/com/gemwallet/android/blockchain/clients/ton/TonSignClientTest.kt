package com.gemwallet.android.blockchain.clients.ton

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class TonSignClientTest {

    @Test
    fun swapTransfer_usesQuoteValueForAttachedTon() {
        val destination = "EQAuaQQMX8BER-YJpXJ2LWfF7GVVvtXLs4o0w9Y-wDJuAMzF"
        val value = "241000000"
        val payload = "stonfi-payload"
        val params = mockk<ConfirmParams.SwapParams>()

        every { params.toAddress } returns destination
        every { params.value } returns value
        every { params.swapData } returns payload
        every { params.memo() } returns null

        val transfer = TonSignClient(Chain.Ton).swapTransfer(params)
        val attachedTon = BigInteger(transfer.amount.toByteArray())

        assertEquals(destination, transfer.dest)
        assertEquals(BigInteger(value), attachedTon)
        assertEquals(payload, transfer.customPayload)
    }
}
