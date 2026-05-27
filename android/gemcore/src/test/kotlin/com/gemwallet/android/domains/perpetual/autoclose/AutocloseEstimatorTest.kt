package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import org.junit.Assert.assertEquals
import org.junit.Test

class AutocloseEstimatorTest {

    @Test
    fun pnlLong() {
        val estimator = AutocloseEstimator(entryPrice = 100.0, positionSize = 10.0, direction = PerpetualDirection.Long, leverage = 5u)
        assertEquals(100.0, estimator.pnl(price = 110.0), DELTA)
        assertEquals(-100.0, estimator.pnl(price = 90.0), DELTA)
    }

    @Test
    fun pnlShort() {
        val estimator = AutocloseEstimator(entryPrice = 100.0, positionSize = -10.0, direction = PerpetualDirection.Short, leverage = 5u)
        assertEquals(100.0, estimator.pnl(price = 90.0), DELTA)
        assertEquals(-100.0, estimator.pnl(price = 110.0), DELTA)
    }

    @Test
    fun targetPriceFromRoeLong() {
        val estimator = AutocloseEstimator(entryPrice = 100.0, positionSize = 10.0, direction = PerpetualDirection.Long, leverage = 5u)
        assertEquals(110.0, estimator.targetPriceFromRoe(roePercent = 50, type = TpslType.TakeProfit), DELTA)
        assertEquals(90.0, estimator.targetPriceFromRoe(roePercent = 50, type = TpslType.StopLoss), DELTA)
    }

    @Test
    fun targetPriceFromRoeShort() {
        val estimator = AutocloseEstimator(entryPrice = 100.0, positionSize = -10.0, direction = PerpetualDirection.Short, leverage = 5u)
        assertEquals(90.0, estimator.targetPriceFromRoe(roePercent = 50, type = TpslType.TakeProfit), DELTA)
        assertEquals(110.0, estimator.targetPriceFromRoe(roePercent = 50, type = TpslType.StopLoss), DELTA)
    }

    @Test
    fun percentSuggestionsByLeverageTier() {
        assertEquals(listOf(5, 10, 15), estimator(leverage = 1u).percentSuggestions)
        assertEquals(listOf(10, 15, 25), estimator(leverage = 5u).percentSuggestions)
        assertEquals(listOf(15, 25, 50), estimator(leverage = 10u).percentSuggestions)
        assertEquals(listOf(25, 50, 100), estimator(leverage = 20u).percentSuggestions)
    }

    private fun estimator(leverage: UByte) = AutocloseEstimator(
        entryPrice = 100.0,
        positionSize = 10.0,
        direction = PerpetualDirection.Long,
        leverage = leverage,
    )

    private companion object {
        const val DELTA = 1e-9
    }
}
