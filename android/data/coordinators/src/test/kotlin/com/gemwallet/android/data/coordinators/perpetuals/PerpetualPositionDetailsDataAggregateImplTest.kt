package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.wallet.core.primitives.PerpetualPosition
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemLocalizedText
import java.util.Locale

class PerpetualPositionDetailsDataAggregateImplTest {
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
    fun pnlWithPercentage_zeroMargin_usesZeroPercent() {
        val aggregate = aggregate(mockPerpetualPosition(marginAmount = 0.0, pnl = 20.47))

        val pnl = aggregate.pnl as GemLocalizedText.Pnl
        assertEquals(20.47, pnl.amount.value, 0.0)
        assertEquals(0.0, pnl.percent.value, 0.0)
    }

    private fun aggregate(position: PerpetualPosition) = PerpetualPositionDetailsDataAggregateImpl(mockPerpetualPositionData(position = position))
}
