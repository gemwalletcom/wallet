package com.gemwallet.android.model

import java.math.RoundingMode

object PriceSuggestionsFormatter {

    fun roundedValues(price: Double, byPercent: Double = 5.0): List<Double> {
        if (price < 0.01 || byPercent <= 0) return emptyList()

        val lowerTarget = price * (1 - byPercent / 100)
        val upperTarget = price * (1 + byPercent / 100)

        val step = step(lowerTarget)

        val lower = Math.floor(lowerTarget / step) * step
        val upper = ceil(upperTarget / step, if (step > 1) RoundingMode.HALF_UP else RoundingMode.UP) * step

        return listOf(lower, upper)
    }

    private fun step(value: Double): Double = when {
        value < 1 -> 0.01
        value < 100 -> 1.0
        value < 500 -> 10.0
        value < 10_000 -> 50.0
        else -> 1000.0
    }

    private fun ceil(value: Double, roundingMode: RoundingMode): Double {
        return value.toBigDecimal().setScale(0, roundingMode).toDouble()
    }
}
