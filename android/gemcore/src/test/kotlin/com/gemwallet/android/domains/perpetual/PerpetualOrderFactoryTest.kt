package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.testkit.mockPerpetualTransferData
import com.wallet.core.primitives.PerpetualDirection
import java.math.BigInteger
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

    @Test
    fun orderActionFor_mapsPositionActionToOrderAction() {
        val transferData = mockPerpetualTransferData(direction = PerpetualDirection.Long)
        val cases = listOf(
            PerpetualPositionAction.Open(transferData) to PerpetualOrderFactory.OrderAction.Open,
            PerpetualPositionAction.Increase(transferData) to PerpetualOrderFactory.OrderAction.Open,
            PerpetualPositionAction.Reduce(
                data = transferData,
                available = BigInteger.ZERO,
                positionDirection = PerpetualDirection.Long,
            ) to PerpetualOrderFactory.OrderAction.Close,
        )
        cases.forEach { (action, expected) ->
            assertEquals(action::class.simpleName, expected, PerpetualOrderFactory.orderActionFor(action))
        }
    }
}
