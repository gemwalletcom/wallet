package com.gemwallet.android.domains.pricealerts

import com.gemwallet.android.model.format
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

fun PriceAlertNotificationType.direction(
    currentPrice: Double,
    inputValue: Double,
    selectedDirection: PriceAlertDirection,
): PriceAlertDirection? = when (this) {
    PriceAlertNotificationType.Price ->
        if (currentPrice > inputValue) PriceAlertDirection.Down else PriceAlertDirection.Up
    PriceAlertNotificationType.PricePercentChange -> selectedDirection
    PriceAlertNotificationType.Auto -> null
}

fun PriceAlertNotificationType.formatAmount(inputValue: Double, currency: Currency): String = when (this) {
    PriceAlertNotificationType.Price -> currency.format(inputValue)
    PriceAlertNotificationType.PricePercentChange -> "$inputValue%"
    PriceAlertNotificationType.Auto -> ""
}
