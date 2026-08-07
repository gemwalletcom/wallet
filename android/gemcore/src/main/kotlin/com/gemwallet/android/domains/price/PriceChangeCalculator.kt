package com.gemwallet.android.domains.price

import uniffi.gemstone.PriceChangeCalculator as GemPriceChangeCalculator

object PriceChangeCalculator {
    private val calculator = GemPriceChangeCalculator()

    fun percentage(from: Double, to: Double): Double = calculator.percentage(from, to)

    fun amount(percentage: Double, value: Double): Double = calculator.amount(percentage, value)
}
