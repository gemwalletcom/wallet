package com.gemwallet.android.domains.price.values

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemValueStyle

class RowFormatters {
    val value = ValueFormatter(style = GemValueStyle.SHORT)
    private val currencies = mutableMapOf<Currency, CurrencyFormatter>()

    fun currency(currency: Currency): CurrencyFormatter = currencies.getOrPut(currency) { CurrencyFormatter(currency = currency) }

    fun price(currency: Currency, value: Double?, changePercentage: Double?): PriceValue {
        val priceValue = value?.takeIf(Double::isFinite)
        val change = changePercentage?.takeIf(Double::isFinite)
        return PriceValue(
            currency = currency,
            value = priceValue,
            changePercentage = change,
            valueFormatted = priceValue?.let { currency(currency).string(it) }.orEmpty(),
            changePercentageFormatted = change.formatAsPercentage(),
            state = change.toValueDirection(),
        )
    }
}
