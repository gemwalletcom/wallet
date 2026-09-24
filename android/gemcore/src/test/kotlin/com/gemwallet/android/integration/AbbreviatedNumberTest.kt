package com.gemwallet.android.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.text
import junit.framework.TestCase.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemValueTone
import java.util.Locale

/**
 * The parity fixture for the abbreviated path, which needs android.icu.text.CompactDecimalFormat and
 * so cannot run as a plain JVM test. It mirrors FormattedNumberTests on iOS case for case.
 */
@RunWith(AndroidJUnit4::class)
class AbbreviatedNumberTest {

    private val us = Locale.US

    private fun abbreviated(value: Double, rounding: GemNumberRounding = GemNumberRounding.TO_NEAREST, notation: GemNumberNotation = GemNumberNotation.PLAIN, tone: GemValueTone = GemValueTone.PLAIN) = uniffi.gemstone.GemFormattedNumber(
        value = value,
        unit = GemNumberUnit.Currency("USD"),
        display = GemNumberDisplay.Abbreviated,
        notation = notation,
        tone = tone,
        rounding = rounding,
    ).text(us)

    @Test
    fun anAbbreviatedValueRoundsTheWayTheRecordAsks() {
        assertEquals("$1.24M", abbreviated(1_235_999.0))
        assertEquals("$1.23M", abbreviated(1_235_999.0, rounding = GemNumberRounding.TOWARD_ZERO))
    }

    @Test
    fun anAbbreviatedValueKeepsItsSign() {
        assertEquals("+$1.24M", abbreviated(1_235_999.0, notation = GemNumberNotation.SIGNED, tone = GemValueTone.POSITIVE))
        assertEquals("-$1.24M", abbreviated(-1_235_999.0, notation = GemNumberNotation.SIGNED))
        assertEquals("-$1.24M", abbreviated(-1_235_999.0))
    }
}
