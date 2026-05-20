package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale

class PerpetualDetailsDataAggregateImplTest {
    private val defaultLocale = Locale.getDefault()

    @Before
    fun setup() {
        Locale.setDefault(Locale.US)
    }

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun funding_annualizesHourlyRate() {
        assertEquals("+11.39%", aggregate(funding = 0.0013).funding)
        assertEquals("-3.50%", aggregate(funding = -0.0004).funding)
        assertEquals("+87.60%", aggregate(funding = 0.01).funding)
    }

    private fun aggregate(funding: Double) =
        PerpetualDetailsDataAggregateImpl(mockPerpetualData(perpetual = mockPerpetual(funding = funding)))
}
