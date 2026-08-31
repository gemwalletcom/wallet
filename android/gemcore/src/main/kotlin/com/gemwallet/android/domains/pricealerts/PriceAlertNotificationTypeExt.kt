package com.gemwallet.android.domains.pricealerts

import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertNotificationType

fun PriceAlertNotificationType.formatAmount(inputValue: Double, currency: Currency): String = when (this) {
    PriceAlertNotificationType.Price -> CurrencyFormatter(currency = currency).string(inputValue)
    PriceAlertNotificationType.PricePercentChange -> "$inputValue%"
    PriceAlertNotificationType.Auto -> ""
}
