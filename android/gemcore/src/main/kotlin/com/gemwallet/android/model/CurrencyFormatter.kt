package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency
import java.math.BigDecimal
import java.util.Locale

class CurrencyFormatter(val style: GemCurrencyStyle = GemCurrencyStyle.CURRENCY, val currencyCode: String, val locale: Locale = Locale.getDefault()) {
    constructor(
        style: GemCurrencyStyle = GemCurrencyStyle.CURRENCY,
        currency: Currency,
        locale: Locale = Locale.getDefault(),
    ) : this(style, currency.string, locale)

    fun string(value: Double): String = formattedCurrency(value, currencyCode, style).text(locale)

    fun string(value: BigDecimal): String = string(value.toDouble())
}
