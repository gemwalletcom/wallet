package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PaymentRoutePayloadTest {

    @Test
    fun pack_carriesEveryFieldTheRecipientScreenPrefillsFrom() {
        val request = PaymentRequest(
            address = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh",
            amount = PaymentAmount.ExactValue("10"),
            memo = "12345",
            assetId = AssetId(Chain.Xrp),
        )

        val packed = requireNotNull(request.pack())

        assertEquals(request, unpackPaymentRequest(packed))
    }

    @Test
    fun unpack_returnsNullForInvalidRoutePayload() {
        assertNull(unpackPaymentRequest("invalid"))
    }
}
