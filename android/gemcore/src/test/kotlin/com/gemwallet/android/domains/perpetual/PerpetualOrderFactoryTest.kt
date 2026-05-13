package com.gemwallet.android.domains.perpetual

import com.wallet.core.primitives.PerpetualDirection
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualOrderFactoryTest {

    private data class SlippageCase(
        val direction: PerpetualDirection,
        val action: PerpetualOrderFactory.OrderAction,
        val slippagePercent: Double,
        val expectedPrice: Double,
    )

    @Test
    fun calculateSlippagePrice_appliesCorrectMultiplier() {
        val marketPrice = 100.0
        val cases = listOf(
            SlippageCase(PerpetualDirection.Long, PerpetualOrderFactory.OrderAction.Open, 2.0, 102.0),
            SlippageCase(PerpetualDirection.Short, PerpetualOrderFactory.OrderAction.Open, 2.0, 98.0),
            SlippageCase(PerpetualDirection.Long, PerpetualOrderFactory.OrderAction.Close, 2.0, 98.0),
            SlippageCase(PerpetualDirection.Short, PerpetualOrderFactory.OrderAction.Close, 2.0, 102.0),
            SlippageCase(PerpetualDirection.Long, PerpetualOrderFactory.OrderAction.Open, 0.0, 100.0),
            SlippageCase(PerpetualDirection.Short, PerpetualOrderFactory.OrderAction.Close, 0.0, 100.0),
        )
        cases.forEach { case ->
            val actual = PerpetualOrderFactory.calculateSlippagePrice(
                marketPrice = marketPrice,
                direction = case.direction,
                action = case.action,
                slippage = case.slippagePercent,
            )
            assertEquals(case.toString(), case.expectedPrice, actual, 1e-9)
        }
    }
}
