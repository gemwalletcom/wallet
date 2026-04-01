package com.gemwallet.android.domains.price

import org.junit.Assert.assertEquals
import org.junit.Test

class PriceStateTest {

    @Test
    fun toPriceState_handlesNullableValues() {
        val nullValue: Double? = null
        assertEquals(PriceState.None, nullValue.toPriceState())
        assertEquals(PriceState.Up, 1.0.toPriceState())
        assertEquals(PriceState.Up, 0.0006.toPriceState())
        assertEquals(PriceState.Down, (-1.0).toPriceState())
        assertEquals(PriceState.Down, (-0.0006).toPriceState())
        assertEquals(PriceState.None, 0.0.toPriceState())
        assertEquals(PriceState.None, Double.NaN.toPriceState())
        assertEquals(PriceState.None, Double.POSITIVE_INFINITY.toPriceState())
        assertEquals(PriceState.None, Double.NEGATIVE_INFINITY.toPriceState())
    }

    @Test
    fun toPriceState_usesRawValueSign() {
        assertEquals(PriceState.Up, 0.0006.toPriceState())
        assertEquals(PriceState.Down, (-0.0006).toPriceState())
    }
}
