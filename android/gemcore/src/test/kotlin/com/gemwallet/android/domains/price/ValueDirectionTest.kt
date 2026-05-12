package com.gemwallet.android.domains.price

import org.junit.Assert.assertEquals
import org.junit.Test

class ValueDirectionTest {

    @Test
    fun toValueDirection_handlesNullableValues() {
        val nullValue: Double? = null
        assertEquals(ValueDirection.None, nullValue.toValueDirection())
        assertEquals(ValueDirection.Up, 1.0.toValueDirection())
        assertEquals(ValueDirection.Up, 0.0006.toValueDirection())
        assertEquals(ValueDirection.Down, (-1.0).toValueDirection())
        assertEquals(ValueDirection.Down, (-0.0006).toValueDirection())
        assertEquals(ValueDirection.None, 0.0.toValueDirection())
        assertEquals(ValueDirection.None, Double.NaN.toValueDirection())
        assertEquals(ValueDirection.None, Double.POSITIVE_INFINITY.toValueDirection())
        assertEquals(ValueDirection.None, Double.NEGATIVE_INFINITY.toValueDirection())
    }

    @Test
    fun toValueDirection_usesRawValueSign() {
        assertEquals(ValueDirection.Up, 0.0006.toValueDirection())
        assertEquals(ValueDirection.Down, (-0.0006).toValueDirection())
    }
}
