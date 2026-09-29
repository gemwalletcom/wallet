package com.gemwallet.android.ui.components.chart

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCandleTick
import uniffi.gemstone.GemCandleTickFormat
import java.time.ZoneOffset
import java.util.Locale

class ChartTimeLabelsTest {

    @Test
    fun `times are labelled in the device zone and locale`() {
        val ticks = listOf(120_000L, 180_000L).map { GemCandleTick(date = it, format = GemCandleTickFormat.TIME) }
        assertEquals(listOf("00:02", "00:03"), chartTimeLabels(ticks, ZoneOffset.UTC, Locale.UK))
        assertEquals(listOf("02:02", "02:03"), chartTimeLabels(ticks, ZoneOffset.ofHours(2), Locale.UK))
    }
}
