package com.gemwallet.android.ui.components.chart

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ChartRangeTest {

    @Test
    fun `range exposes ordered edges`() {
        val range = ChartRange(low = 2f, high = 1f)

        assertEquals(1f, range.low)
        assertEquals(2f, range.high)
    }

    @Test
    fun `range exposes a viewport only for a finite positive span`() {
        assertNull(ChartRange(low = 1f, high = 1f).viewport)
        assertNull(ChartRange(low = 1f, high = Float.POSITIVE_INFINITY).viewport)

        val viewport = ChartRange(low = 1f, high = 3f).viewport!!
        assertEquals(1f, viewport.low)
        assertEquals(2f, viewport.span)
        assertEquals(2f, viewport.y(value = 2f, height = 4f))
    }
}
