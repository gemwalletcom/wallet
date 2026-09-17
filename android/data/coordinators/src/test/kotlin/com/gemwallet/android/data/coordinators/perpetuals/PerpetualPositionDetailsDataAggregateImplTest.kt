package com.gemwallet.android.data.coordinators.perpetuals

import uniffi.gemstone.GemValueTone
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.gemwallet.android.testkit.mockPerpetualTriggerOrder
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPosition
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
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
    fun fundingPayments_negativeSmallValue_usesIosPrecisionAndDownState() {
        val aggregate = aggregate(mockPerpetualPosition(funding = -0.7006f))

        assertEquals("-\$0.7006", aggregate.fundingPayments)
        assertEquals(GemValueTone.NEGATIVE, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun fundingPayments_positiveSmallValue_usesPlusSignAndUpState() {
        val aggregate = aggregate(mockPerpetualPosition(funding = 0.7006f))

        assertEquals("+\$0.7006", aggregate.fundingPayments)
        assertEquals(GemValueTone.POSITIVE, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun fundingPayments_missingValue_usesPlaceholderAndNeutralState() {
        val aggregate = aggregate(mockPerpetualPosition(funding = null))

        assertEquals("-", aggregate.fundingPayments)
        assertEquals(GemValueTone.NEUTRAL, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun marginType_usesPositionMarginType() {
        val aggregate = aggregate(mockPerpetualPosition(marginType = PerpetualMarginType.Isolated))

        assertEquals(PerpetualMarginType.Isolated, aggregate.marginType)
    }

    @Test
    fun size_usesPositionSizeValue() {
        val aggregate = aggregate(mockPerpetualPosition(sizeValue = 2522.16))

        assertEquals("\$2,522.16", aggregate.size)
    }

    @Test
    fun entryPrice_usesDynamicPrecision() {
        val aggregate = aggregate(mockPerpetualPosition(entryPrice = 0.003597))

        assertEquals("\$0.003597", aggregate.entryPrice)
    }

    @Test
    fun liquidationPrice_usesDynamicPrecision() {
        val aggregate = aggregate(mockPerpetualPosition(liquidationPrice = 0.003597))

        assertEquals("\$0.003597", aggregate.liquidationPrice)
    }

    @Test
    fun liquidationPrice_zeroValue_isHidden() {
        val aggregate = aggregate(mockPerpetualPosition(liquidationPrice = 0.0))

        assertEquals("", aggregate.liquidationPrice)
    }

    @Test
    fun autoCloseValues_useTriggerOrders() {
        val aggregate = aggregate(mockPerpetualPosition(takeProfit = mockPerpetualTriggerOrder(price = 2.57), stopLoss = mockPerpetualTriggerOrder(price = 1.23)))

        assertEquals(2.57, aggregate.takeProfit ?: 0.0, 0.0)
        assertEquals(1.23, aggregate.stopLoss ?: 0.0, 0.0)
    }

    @Test
    fun pnlWithPercentage_zeroMargin_usesZeroPercent() {
        val aggregate = aggregate(mockPerpetualPosition(marginAmount = 0.0, pnl = 20.47))

        assertEquals("+\$20.47 (+0.00%)", aggregate.pnlWithPercentage)
    }

    private fun aggregate(position: PerpetualPosition) =
        PerpetualPositionDetailsDataAggregateImpl(mockPerpetualPositionData(position = position))
}
