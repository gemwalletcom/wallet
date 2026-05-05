package com.gemwallet.android.model

import org.junit.Assert.assertNull
import org.junit.Test

class NavigationParamsTest {

    @Test
    fun amountParamsUnpack_returnsNullForInvalidRoutePayload() {
        assertNull(AmountParams.unpack("invalid"))
    }

    @Test
    fun confirmParamsUnpack_returnsNullForInvalidRoutePayload() {
        assertNull(ConfirmParams.unpack("invalid"))
    }
}
