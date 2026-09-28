package com.gemwallet.android.ui.components.chart

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCandleTickFormat
import java.time.ZoneOffset
import java.util.Locale

class ChartTimeLabelsTest {

    @Test
    fun `times are labelled in the device zone and locale`() {
        assertEquals(listOf("00:02", "00:03"), chartTimeLabels(listOf(120_000L, 180_000L), GemCandleTickFormat.TIME, ZoneOffset.UTC, Locale.UK))
        assertEquals(listOf("02:02", "02:03"), chartTimeLabels(listOf(120_000L, 180_000L), GemCandleTickFormat.TIME, ZoneOffset.ofHours(2), Locale.UK))
    }
}
