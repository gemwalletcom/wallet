package com.gemwallet.android.domains.percentage

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Locale

class PercentageFormatterTest {

    @Test
    fun formatAsPercentage_usLocale_matchesIosPercentFormatter() {
        assertEquals("-1.23%", (-1.23).formatAsPercentage(locale = Locale.US))
        assertEquals("+11.12%", 11.12.formatAsPercentage(locale = Locale.US))
        assertEquals("+11.00%", 11.0.formatAsPercentage(locale = Locale.US))
        assertEquals("+12,000,123.00%", 12_000_123.0.formatAsPercentage(locale = Locale.US))
    }

    @Test
    fun formatAsPercentage_ukLocale_matchesIosPercentFormatter() {
        assertEquals("-1.23%", (-1.23).formatAsPercentage(locale = Locale.UK))
        assertEquals("+11.12%", 11.12.formatAsPercentage(locale = Locale.UK))
        assertEquals("+11.00%", 11.0.formatAsPercentage(locale = Locale.UK))
        assertEquals("+12,000,123.00%", 12_000_123.0.formatAsPercentage(locale = Locale.UK))
    }

    @Test
    fun formatAsPercentage_signless_matchesIosPercentSignLessFormatter() {
        assertEquals("1.23%", (-1.23).formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess, locale = Locale.US))
        assertEquals("11.12%", 11.12.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess, locale = Locale.US))
        assertEquals("11.00%", 11.0.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess, locale = Locale.US))
        assertEquals("12,000,123.00%", 12_000_123.0.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess, locale = Locale.US))
    }

    @Test
    fun formatAsPercentage_matchesIosHalfEvenRounding() {
        assertEquals("+2.34%", 2.345.formatAsPercentage(locale = Locale.US))
        assertEquals("+2.36%", 2.355.formatAsPercentage(locale = Locale.US))
        assertEquals("-5.21%", (-5.209).formatAsPercentage(locale = Locale.US))
        assertEquals("-5.20%", (-5.205).formatAsPercentage(locale = Locale.US))
        assertEquals("-0.00%", (-0.0006).formatAsPercentage(locale = Locale.US))
        assertEquals("+0.00%", 0.0006.formatAsPercentage(locale = Locale.US))
    }

    @Test
    fun formatAsPercentage_zero_matchesIosFormatter() {
        assertEquals("+0.00%", 0.0.formatAsPercentage(locale = Locale.US))
        assertEquals(
            "0%",
            0.0.formatAsPercentage(
                style = PercentageFormatterStyle.PercentSignLessCompact,
                locale = Locale.US,
            )
        )
    }

    @Test
    fun formatAsPercentage_compactStyle_isAvailableForNonUiCallers() {
        assertEquals("12.5%", 12.5.formatAsPercentage(
            style = PercentageFormatterStyle.PercentSignLessCompact,
            locale = Locale.US,
        ))
        assertEquals("12%", 12.0.formatAsPercentage(
            style = PercentageFormatterStyle.PercentSignLessCompact,
            locale = Locale.US,
        ))
    }

    @Test
    fun formatAsPercentage_nullAndNan_areEmpty() {
        val nullValue: Double? = null
        assertEquals("", nullValue.formatAsPercentage(locale = Locale.US))
        assertEquals("", Double.NaN.formatAsPercentage(locale = Locale.US))
        assertEquals("", Double.POSITIVE_INFINITY.formatAsPercentage(locale = Locale.US))
        assertEquals("", Double.NEGATIVE_INFINITY.formatAsPercentage(locale = Locale.US))
    }
}
