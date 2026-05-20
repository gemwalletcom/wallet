package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockPerpetualReduceData
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModel.Action
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PerpetualConfirmDetailsUIModelFactoryTest {

    @Test
    fun `open exposes direction leverage and no pnl`() {
        val data = mockPerpetualConfirmData(
            direction = PerpetualDirection.Long,
            leverage = 5u,
            slippage = 0.5,
            marketPrice = 123.45,
            entryPrice = null,
            marginAmount = 100.0,
            fiatValue = 500.0,
        )

        val model = create(PerpetualType.Open(data))!!

        assertEquals(Action.Open, model.action)
        assertEquals(PerpetualDirection.Long, model.direction)
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
    fun `close with pnl exposes pnl and entry price`() {
        val data = mockPerpetualConfirmData(
            direction = PerpetualDirection.Short,
            leverage = 3u,
            pnl = 25.0,
            marginAmount = 100.0,
            entryPrice = 99.0,
        )

        val model = create(PerpetualType.Close(data))!!

        assertEquals(Action.Close, model.action)
        val pnl = model.pnl
        assertNotNull(pnl)
        assertEquals(ValueDirection.Up, pnl!!.direction)
        assertTrue(pnl.text.startsWith("+\$25.00"))
        assertTrue(pnl.text.contains("(+25"))
        assertEquals("$99.00", model.entryPriceText)
    }

    @Test
    fun `close without pnl drops pnl field`() {
        val model = create(PerpetualType.Close(mockPerpetualConfirmData(pnl = null)))!!

        assertEquals(Action.Close, model.action)
        assertNull(model.pnl)
    }

    @Test
    fun `increase keeps data direction`() {
        val model = create(
            PerpetualType.Increase(mockPerpetualConfirmData(direction = PerpetualDirection.Long))
        )!!

        assertEquals(Action.Increase, model.action)
        assertEquals(PerpetualDirection.Long, model.direction)
    }

    @Test
    fun `reduce uses position direction not data direction`() {
        val reduce = mockPerpetualReduceData(
            data = mockPerpetualConfirmData(direction = PerpetualDirection.Short),
            positionDirection = PerpetualDirection.Long,
        )

        val model = create(PerpetualType.Reduce(reduce))!!

        assertEquals(Action.Reduce, model.action)
        assertEquals(PerpetualDirection.Long, model.direction)
    }

    @Test
    fun `modify returns null`() {
        val modify = PerpetualModifyConfirmData(
            baseAsset = mockPerpetualConfirmData().baseAsset,
            assetIndex = 0,
            modifyTypes = emptyList(),
        )

        assertNull(create(PerpetualType.Modify(modify)))
    }

    @Test
    fun `autoclose exposes formatted take profit and stop loss`() {
        val data = mockPerpetualConfirmData(takeProfit = "150.0", stopLoss = "80.0")

        val autoclose = create(PerpetualType.Open(data))!!.autoclose
        assertNotNull(autoclose)
        assertEquals("$150.00", autoclose!!.takeProfitText)
        assertEquals("$80.00", autoclose.stopLossText)
    }

    @Test
    fun `autoclose omits missing side`() {
        val tpOnly = create(PerpetualType.Open(mockPerpetualConfirmData(takeProfit = "150.0")))!!
        val slOnly = create(PerpetualType.Open(mockPerpetualConfirmData(stopLoss = "80.0")))!!
        val none = create(PerpetualType.Open(mockPerpetualConfirmData()))!!

        assertEquals("$150.00", tpOnly.autoclose!!.takeProfitText)
        assertNull(tpOnly.autoclose.stopLossText)
        assertNull(slOnly.autoclose!!.takeProfitText)
        assertEquals("$80.00", slOnly.autoclose.stopLossText)
        assertNull(none.autoclose)
    }

    private fun create(type: PerpetualType) = PerpetualConfirmDetailsUIModelFactory.create(type)
}
