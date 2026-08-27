package com.gemwallet.android.ext

import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.priceAlertId

val PriceAlert.id: String
    get() = priceAlertId(toJson())

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
