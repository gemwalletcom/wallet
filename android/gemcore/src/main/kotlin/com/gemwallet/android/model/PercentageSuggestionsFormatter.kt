package com.gemwallet.android.model

object PercentageSuggestionsFormatter {

    fun suggestions(price: Double): List<Int> {
        val base = when {
            price < 100 -> 5
            price < 10_000 -> 3
            else -> 2
        }
        return listOf(base, base * 2, base * 3)
    }
}
