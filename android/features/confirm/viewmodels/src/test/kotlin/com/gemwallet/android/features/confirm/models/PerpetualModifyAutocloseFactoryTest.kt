package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.testkit.mockCancel
import com.gemwallet.android.testkit.mockPerpetualModifyConfirmData
import com.gemwallet.android.testkit.mockTpslOrder
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualModifyAutocloseFactoryTest {

    @Test
    fun setBothPrices() {
        val element = PerpetualModifyAutocloseFactory.create(
            mockPerpetualModifyConfirmData(
                modifyTypes = listOf(mockTpslOrder(takeProfit = "65000", stopLoss = "55000")),
            ),
        )
        assertEquals("$65,000.00", element?.takeProfitText)
        assertEquals("$55,000.00", element?.stopLossText)
    }

    @Test
    fun cancelExistingShowsDash() {
        val element = PerpetualModifyAutocloseFactory.create(
            mockPerpetualModifyConfirmData(
                modifyTypes = listOf(mockCancel(orderIds = listOf(111L, 222L))),
                takeProfitOrderId = 111L,
                stopLossOrderId = 222L,
            ),
        )
        assertEquals("-", element?.takeProfitText)
        assertEquals("-", element?.stopLossText)
    }

    @Test
    fun cancelAndSetShowsNewPriceNotDash() {
        val element = PerpetualModifyAutocloseFactory.create(
            mockPerpetualModifyConfirmData(
                modifyTypes = listOf(
                    mockCancel(orderIds = listOf(111L)),
                    mockTpslOrder(takeProfit = "70000"),
                ),
                takeProfitOrderId = 111L,
            ),
        )
        assertEquals("$70,000.00", element?.takeProfitText)
    }
}
