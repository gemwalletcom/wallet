package com.gemwallet.android.model

import com.gemwallet.android.domains.price.PriceChangeCalculator
import kotlin.math.abs

class PriceChangeFormatter(private val currencyFormatter: CurrencyFormatter) {
    fun string(value: Double): String = PriceChangeCalculator.sign(value).format(currencyFormatter.string(abs(value)))
}
