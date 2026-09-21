package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.testkit.mockChartHeader
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemChartValueType
import uniffi.gemstone.GemValueTone
import java.util.Locale

class ChartHeaderUIModelTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun buildPopulatesFromTheCoreHeader() {
        val model = ChartHeaderUIModel.build(
            header = mockChartHeader(value = 110.0, base = 100.0),
            timestamp = 5_000L,
            dateFormatter = { "@$it" },
        )
        assertEquals("$110.00", model.priceText)
        assertEquals("+10.00%", model.changeText)
        assertEquals(GemValueTone.PLAIN, model.priceTone)
        assertEquals(GemValueTone.POSITIVE, model.changeTone)
        assertEquals("@5000", model.dateText)
        assertNull(model.headerValueText)
    }

    @Test
    fun buildOmitsDateWhenTimestampIsNull() {
        val model = ChartHeaderUIModel.build(header = mockChartHeader(value = 50.0, base = 100.0))

        assertEquals("$50.00", model.priceText)
        assertEquals(GemValueTone.NEGATIVE, model.changeTone)
        assertNull(model.dateText)
    }

    @Test
    fun buildPriceChangeSignsTheValueAndParenthesisesThePercent() {
        val model = ChartHeaderUIModel.build(
            header = mockChartHeader(
                value = 190.0,
                base = 100.0,
                showsSecondaryValue = true,
                valueType = GemChartValueType.PRICE_CHANGE,
            ),
            type = GemChartValueType.PRICE_CHANGE,
        )
        assertEquals("+$90.00", model.priceText)
        assertEquals("$190.00", model.headerValueText)
        assertEquals("(90.00%)", model.changeText)
        assertEquals(GemChartValueType.PRICE_CHANGE, model.type)
        assertEquals(GemValueTone.POSITIVE, model.priceTone)
        assertEquals(GemValueTone.POSITIVE, model.changeTone)
    }

    @Test
    fun buildPriceChangeOmitsThePercentWithoutASecondaryValue() {
        val model = ChartHeaderUIModel.build(
            header = mockChartHeader(value = 60.0, base = 100.0, valueType = GemChartValueType.PRICE_CHANGE),
            type = GemChartValueType.PRICE_CHANGE,
        )
        assertEquals("-$40.00", model.priceText)
        assertNull(model.changeText)
        assertNull(model.headerValueText)
        assertEquals(GemChartValueType.PRICE_CHANGE, model.type)
        assertEquals(GemValueTone.NEGATIVE, model.priceTone)
        assertEquals(GemValueTone.NEUTRAL, model.changeTone)
    }
}
