package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.math.parseNumber

class AmountValidator(
    private val minValue: Double,
    private val maxValue: Double = MAX_FIAT_AMOUNT,
) {
    var error: BuyError? = null
        private set

    fun validate(input: String): Boolean {
        error = null
        val value = try {
            input.ifEmpty { "0.0" }.parseNumber().toDouble()
        } catch (_: Throwable) {
            error = BuyError.ValueIncorrect
            return false
        }
        if (value < minValue) {
            error = BuyError.MinimumAmount
            return false
        }
        if (value == 0.0) {
            error = BuyError.EmptyAmount
            return false
        }
        if (value > maxValue) {
            error = BuyError.MaximumAmount
            return false
        }
        return true
    }

    companion object {
        const val MAX_FIAT_AMOUNT = 10000.0
    }
}