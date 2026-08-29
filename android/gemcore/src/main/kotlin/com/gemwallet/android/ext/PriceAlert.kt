package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import uniffi.gemstone.GemPriceAlertRulesService

private val priceAlertRules = GemPriceAlertRulesService()

val PriceAlert.id: String
    get() = priceAlertRules.alertId(toJson())

val PriceAlert.type: PriceAlertNotificationType
    get() = priceAlertRules.notificationType(toJson()).decodeJson()

val PriceAlert.shouldDisplay: Boolean
    get() = priceAlertRules.shouldDisplay(toJson())
