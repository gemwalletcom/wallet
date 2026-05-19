package com.gemwallet.android.domains.price.values

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency

interface EquivalentValue {
    val currency: Currency
    val value: Double?
    val changePercentage: Double?

    val valueFormatted: String get() {
        val priceValue = value
        return if (priceValue == null || !priceValue.isFinite()) {
            ""
        } else {
            CurrencyFormatter(currency = currency).string(priceValue)
        }
    }

    val changePercentageFormatted: String
        get() = changePercentage.formatAsPercentage()

    val state: ValueDirection
        get() = changePercentage.toValueDirection()
}
