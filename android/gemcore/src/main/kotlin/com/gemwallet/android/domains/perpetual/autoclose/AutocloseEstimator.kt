package com.gemwallet.android.domains.perpetual.autoclose

import com.gemwallet.android.domains.price.PriceChange
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import kotlin.math.abs

class AutocloseEstimator(
    val entryPrice: Double,
    val positionSize: Double,
    val direction: PerpetualDirection,
    val leverage: UByte,
) {
    val hasSize: Boolean get() = positionSize != 0.0

    val percentSuggestions: List<Int>
        get() = when {
            leverage <= 3u -> listOf(5, 10, 15)
            leverage <= 5u -> listOf(10, 15, 25)
            leverage <= 10u -> listOf(15, 25, 50)
            else -> listOf(25, 50, 100)
        }

    fun pnl(price: Double): Double {
        val absSize = abs(positionSize)
        val delta = price - entryPrice
        return when (direction) {
            PerpetualDirection.Long -> delta * absSize
            PerpetualDirection.Short -> -delta * absSize
        }
    }

    private fun priceChangePercent(price: Double): Double {
        val raw = PriceChange.percentage(from = entryPrice, to = price)
        return if (direction == PerpetualDirection.Short) -raw else raw
    }

    fun roe(price: Double): Double = priceChangePercent(price) * leverage.toInt()

    fun targetPriceFromRoe(roePercent: Int, type: TpslType): Double {
        val leverageInt = leverage.toInt().coerceAtLeast(1)
        val fraction = roePercent.toDouble() / leverageInt.toDouble() / 100.0
        val sign = when (type) {
            TpslType.TakeProfit -> when (direction) {
                PerpetualDirection.Long -> 1.0
                PerpetualDirection.Short -> -1.0
            }
            TpslType.StopLoss -> when (direction) {
                PerpetualDirection.Long -> -1.0
                PerpetualDirection.Short -> 1.0
            }
        }
        return entryPrice * (1.0 + sign * fraction)
    }
}
