package com.gemwallet.android.serializer

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class RoutePayloadTest {
    @Test
    fun paymentRequestRoundTrip() {
        val request = PaymentRequest(
            address = "rEb8TK3gBgk5auZkwc6SHnwGVJ8DCR2X2f",
            amount = PaymentAmount.ExactValue("10"),
            memo = "12345",
            assetId = AssetId(Chain.Xrp, null),
        )
        val packed = requireNotNull(request.packRoutePayload())

        assertEquals(request, unpackRoutePayload<PaymentRequest>(packed))
    }

    @Test
    fun invalidPayloadReturnsNull() {
        assertNull(unpackRoutePayload<PaymentRequest>("invalid"))
    }
}
