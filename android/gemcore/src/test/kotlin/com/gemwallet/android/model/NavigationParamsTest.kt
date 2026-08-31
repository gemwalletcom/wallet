package com.gemwallet.android.model

import uniffi.gemstone.GemTransferService
import org.junit.Assert.assertNull
import org.junit.Test

class NavigationParamsTest {

    private val transferService = GemTransferService()

    @Test
    fun amountParamsUnpack_returnsNullForInvalidRoutePayload() {
        assertNull(AmountParams.unpack("invalid"))
    }

    @Test
    fun confirmParamsUnpack_returnsNullForInvalidRoutePayload() {
        assertNull(ConfirmParams.unpack("invalid", transferService))
    }
}
