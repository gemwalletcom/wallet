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
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartValueType
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
    fun buildPopulatesFromTheCoreHeader() {
        val model = ChartHeaderUIModel.build(
            header = GemChartHeader(value = 110.0, secondaryValue = null, changePercentage = 10.0),
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
            header = GemChartHeader(value = 50.0, secondaryValue = null, changePercentage = -5.0),
            priceFormatter = formatter,
        )
        assertEquals("$50.00", model.priceText)
        assertEquals(ValueDirection.Down, model.direction)
        assertNull(model.dateText)
    }

    @Test
    fun buildFormatsSecondaryValueWithPriceFormatter() {
        val model = ChartHeaderUIModel.build(
            header = GemChartHeader(value = 50.0, secondaryValue = 1500.0, changePercentage = null),
            priceFormatter = formatter,
        )
        assertEquals("$1500.00", model.headerValueText)
        assertNull(model.changeText)
        assertEquals(ValueDirection.None, model.direction)
    }

    @Test
    fun buildPriceChangeShowsSignedAmountValueAndParenthesizedPercent() {
        val model = ChartHeaderUIModel.build(
            header = GemChartHeader(value = 90.0, secondaryValue = 190.0, changePercentage = 12.0),
            type = GemChartValueType.PRICE_CHANGE,
            priceFormatter = formatter,
            priceChangeFormatter = changeFormatter,
        )
        assertEquals("+$90.00", model.priceText)
        assertEquals("$190.00", model.headerValueText)
        assertEquals(GemChartValueType.PRICE_CHANGE, model.type)
        assertEquals(ValueDirection.Up, model.direction)
        assertTrue(model.changeText!!.startsWith("(") && model.changeText!!.endsWith(")"))
    }

    @Test
    fun buildPriceChangeOmitsPercentWhenCoreWithholdsIt() {
        val model = ChartHeaderUIModel.build(
            header = GemChartHeader(value = -40.0, secondaryValue = null, changePercentage = null),
            type = GemChartValueType.PRICE_CHANGE,
            priceFormatter = formatter,
            priceChangeFormatter = changeFormatter,
        )
        assertEquals("-$40.00", model.priceText)
        assertNull(model.changeText)
        assertNull(model.headerValueText)
        assertEquals(GemChartValueType.PRICE_CHANGE, model.type)
        assertEquals(ValueDirection.Down, model.direction)
    }
}
