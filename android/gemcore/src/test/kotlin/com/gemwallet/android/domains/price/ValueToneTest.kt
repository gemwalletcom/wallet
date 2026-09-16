package com.gemwallet.android.domains.price

import uniffi.gemstone.GemValueTone
import org.junit.Assert.assertEquals
import org.junit.Test

class ValueToneTest {

    @Test
    fun tone_handlesNullableValues() {
        val nullValue: Double? = null
        assertEquals(GemValueTone.NEUTRAL, nullValue.tone())
        assertEquals(GemValueTone.POSITIVE, 1.0.tone())
        assertEquals(GemValueTone.POSITIVE, 0.0006.tone())
        assertEquals(GemValueTone.NEGATIVE, (-1.0).tone())
        assertEquals(GemValueTone.NEGATIVE, (-0.0006).tone())
        assertEquals(GemValueTone.NEUTRAL, 0.0.tone())
        assertEquals(GemValueTone.NEUTRAL, Double.NaN.tone())
        assertEquals(GemValueTone.NEUTRAL, Double.POSITIVE_INFINITY.tone())
        assertEquals(GemValueTone.NEUTRAL, Double.NEGATIVE_INFINITY.tone())
    }

    @Test
    fun tone_usesRawValueSign() {
        assertEquals(GemValueTone.POSITIVE, 0.0006.tone())
        assertEquals(GemValueTone.NEGATIVE, (-0.0006).tone())
    }
}
