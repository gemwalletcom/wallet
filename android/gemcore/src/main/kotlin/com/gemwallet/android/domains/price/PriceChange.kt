package com.gemwallet.android.domains.price

object PriceChange {
    fun percentage(from: Double, to: Double): Double {
        if (from == 0.0) return 0.0
        return (to - from) / from * 100.0
    }

    fun amount(percentage: Double, value: Double): Double {
        val denominator = 100.0 + percentage
        if (denominator == 0.0) return 0.0
        return value * percentage / denominator
    }
}
