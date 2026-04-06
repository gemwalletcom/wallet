package com.gemwallet.android.ext

import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import java.math.BigDecimal

val PriceAlert.id: String
    get() {
        if (price == null && pricePercentChange == null && priceDirection == null) {
            return assetId.toIdentifier()
        }
        return buildList {
            add(assetId.toIdentifier())
            add(currency)
            price?.let { add(formatIdentifierValue(it)) }
            pricePercentChange?.let { add(formatIdentifierValue(it)) }
            priceDirection?.let { add(it.string) }
        }.joinToString("_")
    }

val PriceAlert.type: PriceAlertNotificationType
    get() = when {
        priceDirection == null && price == null && pricePercentChange == null -> PriceAlertNotificationType.Auto
        priceDirection != null && price != null && pricePercentChange == null -> PriceAlertNotificationType.Price
        priceDirection != null && price == null && pricePercentChange != null -> PriceAlertNotificationType.PricePercentChange
        else -> PriceAlertNotificationType.Auto
    }

val PriceAlert.shouldDisplay: Boolean
    get() = when (type) {
        PriceAlertNotificationType.Auto -> true
        PriceAlertNotificationType.Price,
        PriceAlertNotificationType.PricePercentChange -> lastNotifiedAt == null
    }

private fun formatIdentifierValue(value: Double): String {
    return BigDecimal.valueOf(value)
        .stripTrailingZeros()
        .toPlainString()
}
