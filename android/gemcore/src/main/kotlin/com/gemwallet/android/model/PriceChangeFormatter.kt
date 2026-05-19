package com.gemwallet.android.model

import kotlin.math.abs

class PriceChangeFormatter(private val currencyFormatter: CurrencyFormatter) {
    fun string(value: Double): String {
        val formatted = currencyFormatter.string(abs(value))
        return if (value >= 0) "+$formatted" else "-$formatted"
    }
}
