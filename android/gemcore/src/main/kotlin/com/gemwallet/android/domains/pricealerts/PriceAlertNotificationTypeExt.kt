package com.gemwallet.android.domains.pricealerts

import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

fun PriceAlertNotificationType.direction(
    currentPrice: Double,
    inputValue: Double,
    selectedDirection: PriceAlertDirection,
): PriceAlertDirection? = when (this) {
    PriceAlertNotificationType.Price -> when {
        currentPrice <= 0.0 -> null
        inputValue > currentPrice -> PriceAlertDirection.Up
        inputValue < currentPrice -> PriceAlertDirection.Down
        else -> null
    }
    PriceAlertNotificationType.PricePercentChange -> selectedDirection
    PriceAlertNotificationType.Auto -> null
}

fun PriceAlertNotificationType.formatAmount(inputValue: Double, currency: Currency): String = when (this) {
    PriceAlertNotificationType.Price -> CurrencyFormatter(currency = currency).string(inputValue)
    PriceAlertNotificationType.PricePercentChange -> "$inputValue%"
    PriceAlertNotificationType.Auto -> ""
}
