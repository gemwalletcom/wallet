package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.util.Locale

class ChartHeaderUIModelTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    private val formatter: (Double) -> String = { "$%.2f".format(it) }
    private val changeFormatter: (Double) -> String =
        PriceChangeFormatter(CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD, locale = Locale.US))::string

    @Test
    fun buildPopulatesFromRawValues() {
        val model = ChartHeaderUIModel.build(
            price = 110.0,
            priceChangePercentage = 10.0,
            timestamp = 5_000L,
            priceFormatter = formatter,
            dateFormatter = { "@$it" },
        )
        assertEquals("$110.00", model.priceText)
        assertEquals(ValueDirection.Up, model.direction)
        assertEquals("@5000", model.dateText)
        assertNull(model.headerValueText)
    }

    @Test
    fun buildOmitsDateWhenTimestampNull() {
        val model = ChartHeaderUIModel.build(
            price = 50.0,
            priceChangePercentage = -5.0,
            priceFormatter = formatter,
        )
        assertEquals("$50.00", model.priceText)
        assertEquals(ValueDirection.Down, model.direction)
        assertNull(model.dateText)
    }

    @Test
    fun buildFormatsHeaderValueWithPriceFormatter() {
        val model = ChartHeaderUIModel.build(
            price = 50.0,
            priceChangePercentage = 0.0,
            headerValue = 1500.0,
            priceFormatter = formatter,
        )
        assertEquals("$1500.00", model.headerValueText)
        assertEquals(ValueDirection.None, model.direction)
    }

    @Test
    fun buildPriceChangeShowsSignedAmountValueAndParenthesizedPercent() {
        val model = ChartHeaderUIModel.build(
            price = 90.0,
            priceChangePercentage = 12.0,
            type = ChartValueType.PriceChange,
            headerValue = 190.0,
            priceFormatter = formatter,
            priceChangeFormatter = changeFormatter,
        )
        assertEquals("+$90.00", model.priceText)
        assertEquals("$190.00", model.headerValueText)
        assertEquals(ChartValueType.PriceChange, model.type)
        assertEquals(ValueDirection.Up, model.direction)
        assertTrue(model.changeText!!.startsWith("(") && model.changeText!!.endsWith(")"))
    }

    @Test
    fun buildPriceChangeOmitsPercentWhenNoHeaderValue() {
        val model = ChartHeaderUIModel.build(
            price = -40.0,
            priceChangePercentage = -8.0,
            type = ChartValueType.PriceChange,
            priceFormatter = formatter,
            priceChangeFormatter = changeFormatter,
        )
        assertEquals("-$40.00", model.priceText)
        assertNull(model.changeText)
        assertNull(model.headerValueText)
        assertEquals(ChartValueType.PriceChange, model.type)
        assertEquals(ValueDirection.Down, model.direction)
    }
}
