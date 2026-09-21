package com.gemwallet.android.model

import uniffi.gemstone.formattedSignedCurrency

class PriceChangeFormatter(private val currencyFormatter: CurrencyFormatter) {
    fun string(value: Double): String = formattedSignedCurrency(value, currencyFormatter.currencyCode, currencyFormatter.style).text(currencyFormatter.locale)
}
