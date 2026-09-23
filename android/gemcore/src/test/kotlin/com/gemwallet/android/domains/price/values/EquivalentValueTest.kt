package com.gemwallet.android.domains.price.values

import com.gemwallet.android.testkit.mockEquivalentValue
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemValueTone

class EquivalentValueTest {

    @Test
    fun `a value the price cannot express formats as nothing`() {
        assertEquals("", mockEquivalentValue(value = null).valueFormatted)
        assertEquals("", mockEquivalentValue(value = Double.NaN).valueFormatted)
        assertEquals("", mockEquivalentValue(value = Double.POSITIVE_INFINITY).valueFormatted)

        assertTrue(mockEquivalentValue(value = 0.0).valueFormatted.isNotEmpty())
        assertTrue(mockEquivalentValue(value = -100.0).valueFormatted.isNotEmpty())
    }

    @Test
    fun `a missing change has no tone and no percentage`() {
        val missing = mockEquivalentValue(changePercentage = null)

        assertEquals(GemValueTone.NEUTRAL, missing.state)
        assertEquals("", missing.changePercentageFormatted)

        val present = mockEquivalentValue(changePercentage = 2.5)

        assertEquals(GemValueTone.POSITIVE, present.state)
        assertEquals("+2.50%", present.changePercentageFormatted)
    }
}
