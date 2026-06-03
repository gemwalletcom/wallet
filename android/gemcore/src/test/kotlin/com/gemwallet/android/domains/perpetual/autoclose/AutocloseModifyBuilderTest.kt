package com.gemwallet.android.domains.perpetual.autoclose

import com.gemwallet.android.testkit.mockAutocloseField
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.TpslType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class AutocloseModifyBuilderTest {

    private val builder = AutocloseModifyBuilder(direction = PerpetualDirection.Long)

    @Test
    fun canBuildWithValidChange() {
        val takeProfit = mockAutocloseField(TpslType.TakeProfit, price = 110.0, originalPrice = 100.0, error = null)
        val stopLoss = mockAutocloseField(TpslType.StopLoss)
        assertTrue(builder.canBuild(takeProfit, stopLoss))
    }

    @Test
    fun canBuildClearingExistingField() {
        val takeProfit = mockAutocloseField(TpslType.TakeProfit, price = null, originalPrice = 100.0)
        val stopLoss = mockAutocloseField(TpslType.StopLoss)
        assertTrue(builder.canBuild(takeProfit, stopLoss))
    }

    @Test
    fun cannotBuildWithoutChanges() {
        val takeProfit = mockAutocloseField(TpslType.TakeProfit, price = 100.0, originalPrice = 100.0, error = null)
        val stopLoss = mockAutocloseField(TpslType.StopLoss, price = 90.0, originalPrice = 90.0, error = null)
        assertFalse(builder.canBuild(takeProfit, stopLoss))
    }

    @Test
    fun cannotBuildWithInvalidPrice() {
        val takeProfit = mockAutocloseField(TpslType.TakeProfit, price = 50.0, originalPrice = 100.0, error = AutocloseError.TriggerMustBeHigher)
        val stopLoss = mockAutocloseField(TpslType.StopLoss)
        assertFalse(builder.canBuild(takeProfit, stopLoss))
    }

    @Test
    fun buildSetBothEmitsSingleTpslWithSizeZero() {
        val takeProfit = mockAutocloseField(TpslType.TakeProfit, price = 110.0, formattedPrice = "110.0", error = null)
        val stopLoss = mockAutocloseField(TpslType.StopLoss, price = 90.0, formattedPrice = "90.0", error = null)

        val result = builder.build(assetIndex = 5, takeProfit = takeProfit, stopLoss = stopLoss)

        val tpsl = result.single() as PerpetualModifyPositionType.Tpsl
        assertEquals("110.0", tpsl.content.takeProfit)
        assertEquals("90.0", tpsl.content.stopLoss)
        assertEquals("0", tpsl.content.size)
        assertEquals(PerpetualDirection.Long, tpsl.content.direction)
    }

    @Test
    fun buildReplaceEmitsCancelBeforeTpsl() {
        val takeProfit = mockAutocloseField(
            TpslType.TakeProfit,
            price = 120.0,
            formattedPrice = "120.0",
            originalPrice = 100.0,
            error = null,
            orderId = 12345uL,
        )
        val stopLoss = mockAutocloseField(TpslType.StopLoss)

        val result = builder.build(assetIndex = 5, takeProfit = takeProfit, stopLoss = stopLoss)

        assertEquals(2, result.size)
        val cancel = result[0] as PerpetualModifyPositionType.Cancel
        assertEquals(12345L, cancel.content[0].orderId)
        assertEquals(5, cancel.content[0].assetIndex)
        val tpsl = result[1] as PerpetualModifyPositionType.Tpsl
        assertEquals("120.0", tpsl.content.takeProfit)
    }
}
