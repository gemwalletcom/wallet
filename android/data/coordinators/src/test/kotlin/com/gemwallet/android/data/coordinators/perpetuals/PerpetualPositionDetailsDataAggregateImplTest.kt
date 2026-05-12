package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualTriggerOrder
import io.mockk.every
import io.mockk.mockk
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
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
        val aggregate = aggregate(funding = -0.7006f)

        assertEquals("-\$0.7006", aggregate.fundingPayments)
        assertEquals(ValueDirection.Down, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun fundingPayments_positiveSmallValue_usesPlusSignAndUpState() {
        val aggregate = aggregate(funding = 0.7006f)

        assertEquals("+\$0.7006", aggregate.fundingPayments)
        assertEquals(ValueDirection.Up, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun fundingPayments_missingValue_usesPlaceholderAndNeutralState() {
        val aggregate = aggregate(funding = null)

        assertEquals("-", aggregate.fundingPayments)
        assertEquals(ValueDirection.None, aggregate.fundingPaymentsDirection)
    }

    @Test
    fun marginType_usesPositionMarginType() {
        val aggregate = aggregate(marginType = PerpetualMarginType.Isolated)

        assertEquals(PerpetualMarginType.Isolated, aggregate.marginType)
    }

    @Test
    fun size_usesPositionSizeValue() {
        val aggregate = aggregate(sizeValue = 2522.16)

        assertEquals("\$2,522.16", aggregate.size)
    }

    @Test
    fun entryPrice_usesDynamicPrecision() {
        val aggregate = aggregate(entryPrice = 0.003597)

        assertEquals("\$0.003597", aggregate.entryPrice)
    }

    @Test
    fun liquidationPrice_usesDynamicPrecision() {
        val aggregate = aggregate(liquidationPrice = 0.003597)

        assertEquals("\$0.003597", aggregate.liquidationPrice)
        assertEquals(0.003597, aggregate.liquidationValue ?: 0.0, 0.0)
    }

    @Test
    fun liquidationPrice_zeroValue_isHidden() {
        val aggregate = aggregate(liquidationPrice = 0.0)

        assertEquals("", aggregate.liquidationPrice)
        assertNull(aggregate.liquidationValue)
    }

    @Test
    fun autoCloseValues_useTriggerOrders() {
        val aggregate = aggregate(takeProfit = 2.57, stopLoss = 1.23)

        assertEquals(2.57, aggregate.takeProfit ?: 0.0, 0.0)
        assertEquals(1.23, aggregate.stopLoss ?: 0.0, 0.0)
    }

    @Test
    fun pnlWithPercentage_zeroMargin_usesZeroPercent() {
        val aggregate = aggregate(marginAmount = 0.0, pnl = 20.47)

        assertEquals("+\$20.47 (+0.00%)", aggregate.pnlWithPercentage)
    }

    private fun aggregate(
        funding: Float? = null,
        sizeValue: Double = 0.0,
        entryPrice: Double = 4.08,
        liquidationPrice: Double? = null,
        takeProfit: Double? = null,
        stopLoss: Double? = null,
        marginAmount: Double = 190.41,
        marginType: PerpetualMarginType = PerpetualMarginType.Cross,
        pnl: Double = 0.0,
    ): PerpetualPositionDetailsDataAggregateImpl {
        return PerpetualPositionDetailsDataAggregateImpl(
            mockPositionData(
                fundingValue = funding,
                sizeValue = sizeValue,
                entryPrice = entryPrice,
                liquidationPrice = liquidationPrice,
                takeProfit = takeProfit,
                stopLoss = stopLoss,
                marginAmount = marginAmount,
                marginType = marginType,
                pnl = pnl,
            )
        )
    }

    private fun mockPositionData(
        fundingValue: Float?,
        sizeValue: Double,
        entryPrice: Double,
        liquidationPrice: Double?,
        takeProfit: Double?,
        stopLoss: Double?,
        marginAmount: Double,
        marginType: PerpetualMarginType,
        pnl: Double,
    ): PerpetualPositionData {
        val asset = mockAsset()
        val perpetual = mockk<Perpetual> {
            every { name } returns "TON"
        }
        val position = mockk<PerpetualPosition> {
            every { id } returns "pos-ton"
            every { perpetualId } returns "TON-PERP"
            every { size } returns 0.0
            every { this@mockk.sizeValue } returns sizeValue
            every { leverage } returns 10.toUByte()
            every { this@mockk.entryPrice } returns entryPrice
            every { this@mockk.liquidationPrice } returns liquidationPrice
            every { this@mockk.marginType } returns marginType
            every { direction } returns PerpetualDirection.Long
            every { this@mockk.marginAmount } returns marginAmount
            every { this@mockk.pnl } returns pnl
            every { this@mockk.takeProfit } returns takeProfit?.let(::triggerOrder)
            every { this@mockk.stopLoss } returns stopLoss?.let(::triggerOrder)
            every { funding } returns fundingValue
        }
        return mockk {
            every { this@mockk.perpetual } returns perpetual
            every { this@mockk.asset } returns asset
            every { this@mockk.position } returns position
        }
    }

    private fun triggerOrder(price: Double): PerpetualTriggerOrder {
        return mockk {
            every { this@mockk.price } returns price
        }
    }
}
