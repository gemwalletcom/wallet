package com.gemwallet.android.ui.models

import com.gemwallet.android.model.CurrencyFormatter

interface FiatFormattedUIModel {
    val currency: com.wallet.core.primitives.Currency

    val fiat: Double?

    val fiatFormatted: String
        get() = fiat?.let { CurrencyFormatter(currency = currency).string(it) } ?: ""
}
