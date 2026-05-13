package com.gemwallet.android.domains.perpetual

import com.wallet.core.primitives.PerpetualDirection
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualOrderFactoryTest {

    @Test
    fun calculateSlippagePrice_appliesCorrectMultiplierPerDirectionAndAction() {
        val marketPrice = 100.0
        val slippagePercent = 2.0
        val cases = listOf(
            Triple(PerpetualDirection.Long, PerpetualOrderFactory.OrderAction.Open, 102.0),
            Triple(PerpetualDirection.Short, PerpetualOrderFactory.OrderAction.Open, 98.0),
            Triple(PerpetualDirection.Long, PerpetualOrderFactory.OrderAction.Close, 98.0),
            Triple(PerpetualDirection.Short, PerpetualOrderFactory.OrderAction.Close, 102.0),
        )
        cases.forEach { (direction, action, expected) ->
            val actual = PerpetualOrderFactory.calculateSlippagePrice(
                marketPrice = marketPrice,
                direction = direction,
                action = action,
                slippage = slippagePercent,
            )
            assertEquals("$direction + $action", expected, actual, 1e-9)
        }
    }

    @Test
    fun calculateSlippagePrice_zeroSlippageReturnsMarketPrice() {
        PerpetualDirection.entries.forEach { direction ->
            PerpetualOrderFactory.OrderAction.entries.forEach { action ->
                assertEquals(
                    "$direction + $action",
                    100.0,
                    PerpetualOrderFactory.calculateSlippagePrice(100.0, direction, action, 0.0),
                    1e-9,
                )
            }
        }
    }
}
