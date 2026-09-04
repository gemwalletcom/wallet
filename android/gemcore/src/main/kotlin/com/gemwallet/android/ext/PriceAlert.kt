package com.gemwallet.android.ext

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import uniffi.gemstone.PriceAlertFormatter

private val priceAlertFormatter = PriceAlertFormatter()

val PriceAlert.id: String
    get() = priceAlertFormatter.alertId(toJson())

val PriceAlert.type: PriceAlertNotificationType
    get() = priceAlertFormatter.notificationType(toJson()).toPrimitives()
