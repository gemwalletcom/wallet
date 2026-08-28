package com.gemwallet.android.ext

import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.priceAlertId
import uniffi.gemstone.priceAlertNotificationType
import uniffi.gemstone.priceAlertShouldDisplay

val PriceAlert.id: String
    get() = priceAlertId(toJson())

val PriceAlert.type: PriceAlertNotificationType
    get() = priceAlertNotificationType(toJson()).decodeJson()

val PriceAlert.shouldDisplay: Boolean
    get() = priceAlertShouldDisplay(toJson())
