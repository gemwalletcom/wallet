package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockPerpetualDetails
import com.wallet.core.primitives.PerpetualDirection
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.PerpetualConfirmData

class PerpetualConfirmDetailsUIModelFactoryTest {

    @Test
    fun `amounts and slippage are formatted as usd`() {
        val model = create(
            mockPerpetualConfirmData(
                leverage = 5u,
                slippage = 0.5,
                marketPrice = 123.45,
                entryPrice = null,
                marginAmount = 100.0,
                fiatValue = 500.0,
            )
        )

        assertEquals(5, model.leverage)
        assertNull(model.pnl)
        assertNull(model.entryPriceText)
        assertNull(model.autoclose)
        assertEquals("$123.45", model.marketPriceText)
        assertEquals("$100.00", model.marginText)
        assertEquals("$500.00", model.sizeText)
        assertEquals("0.50%", model.slippageText)
    }

    @Test
    fun `pnl carries its share of the margin and its direction`() {
        val model = create(mockPerpetualConfirmData(pnl = 25.0, marginAmount = 100.0, entryPrice = 99.0))

        val pnl = model.pnl
        assertNotNull(pnl)
        assertEquals(ValueDirection.Up, pnl!!.direction)
        assertTrue(pnl.text.startsWith("+\$25.00"))
        assertTrue(pnl.text.contains("(+25"))
        assertEquals("$99.00", model.entryPriceText)
    }

    @Test
    fun `no pnl drops the field`() {
        assertNull(create(mockPerpetualConfirmData(pnl = null)).pnl)
    }

    @Test
    fun `the model keeps the direction core resolved`() {
        val model = PerpetualConfirmDetailsUIModelFactory.create(
            mockPerpetualDetails(
                direction = PerpetualDirection.Long,
                data = mockPerpetualConfirmData(direction = PerpetualDirection.Short),
            )
        )

        assertEquals(PerpetualDirection.Long, model.direction)
    }

    @Test
    fun `autoclose formats both sides and omits a missing one`() {
        val both = create(mockPerpetualConfirmData(takeProfit = "150.0", stopLoss = "80.0")).autoclose
        assertNotNull(both)
        assertEquals("$150.00", both!!.takeProfitText)
        assertEquals("$80.00", both.stopLossText)

        val takeProfitOnly = create(mockPerpetualConfirmData(takeProfit = "150.0")).autoclose
        assertEquals("$150.00", takeProfitOnly!!.takeProfitText)
        assertNull(takeProfitOnly.stopLossText)

        val stopLossOnly = create(mockPerpetualConfirmData(stopLoss = "80.0")).autoclose
        assertNull(stopLossOnly!!.takeProfitText)
        assertEquals("$80.00", stopLossOnly.stopLossText)

        assertNull(create(mockPerpetualConfirmData()).autoclose)
    }

    private fun create(data: PerpetualConfirmData) =
        PerpetualConfirmDetailsUIModelFactory.create(mockPerpetualDetails(data = data))
}
